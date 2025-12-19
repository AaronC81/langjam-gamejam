use std::{collections::{HashMap, HashSet}, error::Error, fmt::Display, ops::ControlFlow, rc::Rc, time::Instant};

use crate::{BinaryOperator, Declaration, Expression, Object, Sprite, Statement, Tone};

pub struct Interpreter {
    top_level_constructor: Vec<Statement>,

    pub(crate) entities: HashMap<EntityId, Entity>,
    pub(crate) entities_by_kinds: HashMap<String, HashSet<EntityId>>,
    next_entity_id: usize,
    
    /// Entity destruction is delayed until a tick has finished, otherwise you encounter errors due
    /// to all of your instance variables disappearing underneath you!
    entities_pending_destroy: HashSet<EntityId>,

    /// Sounds that have been enqueued for play during this tick
    pub(crate) pending_sounds: Vec<Tone>,

    entity_kinds: HashMap<String, Rc<EntityKind>>,

    pub(crate) input_report: InputReport,
    pub(crate) display_config: DisplayConfig,
}

pub type InterpreterResult<T = ()> = Result<T, RuntimeError>;

impl Interpreter {
    pub fn new() -> Self {
        Self {
            top_level_constructor: vec![],
            entities: HashMap::new(),
            entities_by_kinds: HashMap::new(),

            next_entity_id: 1,
            entities_pending_destroy: HashSet::new(),
            pending_sounds: vec![],
            entity_kinds: HashMap::new(),
            input_report: Default::default(),
            display_config: Default::default(),
        }
    }

    pub fn with_declarations(declarations: &[Declaration]) -> InterpreterResult<Interpreter> {
        let mut interpreter = Self::new();
        for decl in declarations {
            interpreter.interpret_declaration(decl, None)?;
        }
        Ok(interpreter)
    }

    pub fn execute_init(&mut self) -> InterpreterResult {
        let mut frame = Frame {
            entity: None,
            locals: HashMap::new(),
        };

        let _ = self.execute_statement_body(&self.top_level_constructor.clone(), &mut frame)?;
        
        self.forbid_sound()?;
        Ok(())
    }

    pub fn update_input_report(&mut self, report: InputReport) {
        self.input_report = report;
    }

    pub fn update_display_config(&mut self, config: DisplayConfig) {
        self.display_config = config;
    }

    pub fn execute_tick(&mut self) -> InterpreterResult<Vec<Tone>> {
        self.entities_pending_destroy.clear();

        let ids_and_kinds = self.entities.iter()
            .map(|(id, entity)| (*id, entity.kind.clone()))
            .collect::<Vec<_>>();

        for (id, kind) in ids_and_kinds {
            if let Some(tick) = kind.tick_handler.as_ref() {
                let mut frame = Frame {
                    entity: Some(id),
                    locals: HashMap::new(),
                };

                let _ = self.execute_statement_body(tick, &mut frame)?;
            }
        }

        for destroyed_entity in &self.entities_pending_destroy {
            let kind = self.entities[destroyed_entity].kind.name.clone();
            self.entities.remove(&destroyed_entity);
            self.entities_by_kinds.get_mut(&kind).unwrap().remove(destroyed_entity);
        }

        let sounds = self.pending_sounds.clone();
        self.pending_sounds.clear();
        Ok(sounds)
    }

    pub fn execute_draw(&mut self) -> InterpreterResult<Vec<DrawOperation>> {
        let mut draw_ops = vec![];

        let ids_and_kinds = self.entities.iter()
            .map(|(id, entity)| (*id, entity.kind.clone()))
            .collect::<Vec<_>>();

        for (id, kind) in ids_and_kinds {
            if let Some(draw) = kind.draw_handler.as_ref() {
                let mut frame = Frame {
                    entity: Some(id),
                    locals: HashMap::new(),
                };

                match self.execute_statement_body(draw, &mut frame)? {
                    ControlFlow::Continue(_) | ControlFlow::Break(Object::Null) => {},
                    ControlFlow::Break(Object::Sprite(sprite)) => {
                        let (x, y) = self.entities[&id].draw_position_ivars()?;
                        draw_ops.push(DrawOperation { x, y, sprite })
                    },

                    _ => return Err(RuntimeError::new("if `draw` returns something, it must be a sprite")),
                }
            }
        }

        self.forbid_sound()?;
        Ok(draw_ops)
    }

    pub(crate) fn execute_statement_body(&mut self, body: &[Statement], frame: &mut Frame) -> InterpreterResult<ControlFlow<Object>> {
        for stmt in body {
            match self.interpret_statement(stmt, frame)? {
                ControlFlow::Break(retval) => return Ok(ControlFlow::Break(retval)),
                ControlFlow::Continue(_) => {},
            }
        }

        Ok(ControlFlow::Continue(()))
    }

    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }

    pub fn interpret_declaration(&mut self, decl: &Declaration, target: Option<&mut EntityKind>) -> InterpreterResult {
        match decl {
            Declaration::EntityDeclaration { name, body } => {
                if target.is_some() {
                    return Err(RuntimeError::new("cannot nest entity definitions"));
                }
                if self.entity_kinds.contains_key(name) {
                    return Err(RuntimeError::new(format!("duplicate entity declaration `{name}`")));
                }

                let mut new_entity_kind = EntityKind {
                    name: name.to_owned(),
                    functions: HashMap::new(),
                    constructor: None,
                    tick_handler: None,
                    draw_handler: None,
                    ivars: vec![],
                };

                for subdecl in body {
                    self.interpret_declaration(subdecl, Some(&mut new_entity_kind))?;
                }

                self.entity_kinds.insert(name.to_owned(), Rc::new(new_entity_kind));
                Ok(())
            }

            Declaration::ConstructorDeclaration { body } => {
                // Constructors may either apply to the current entity, or the entire program
                if let Some(target) = target {
                    if let Some(constructor) = target.constructor.as_mut() {
                        constructor.extend(body.clone());
                    } else {
                        target.constructor = Some(body.clone());
                    }
    
                    Ok(())
                } else {
                    if !self.top_level_constructor.is_empty() {
                        return Err(RuntimeError::new("top-level constructor is already declared"));
                    }
                    self.top_level_constructor = body.clone();
                    Ok(())
                }
            }
            
            Declaration::TickDeclaration { body } => {
                let Some(target) = target else {
                    return Err(RuntimeError::new("tick declarations cannot appear outside of an entity"));
                };
                
                if let Some(tick) = target.tick_handler.as_mut() {
                    tick.extend(body.clone());
                } else {
                    target.tick_handler = Some(body.clone());
                }

                Ok(())
            }

            Declaration::DrawDeclaration { body } => {
                let Some(target) = target else {
                    return Err(RuntimeError::new("draw declarations cannot appear outside of an entity"));
                };
                if target.draw_handler.is_some() {
                    return Err(RuntimeError::new(format!("draw handler is already declared")));
                }

                target.draw_handler = Some(body.clone());
                Ok(())
            }

            Declaration::InstanceVarDeclaration { names } => {
                let Some(target) = target else {
                    return Err(RuntimeError::new("instance variable declarations cannot appear outside of an entity"));
                };

                for name in names {
                    if target.ivars.contains(name) {
                        return Err(RuntimeError::new(format!("instance variable `{name}` is already declared")));
                    }

                    target.ivars.push(name.to_owned());
                }
                Ok(())
            }

            Declaration::FunctionDeclaration { name, parameters, body } => {
                let Some(target) = target else {
                    return Err(RuntimeError::new("function declarations cannot appear outside of an entity"));
                };
                if target.functions.contains_key(name) {
                    return Err(RuntimeError::new(format!("function `{name}` is already declared")));
                }

                let decl = FunctionDeclaration {
                    name: name.to_owned(),
                    parameters: parameters.clone(),
                    body: body.clone(),
                };
                target.functions.insert(name.to_owned(), decl);
                Ok(())
            }

            Declaration::UseDeclaration { name } => {
                let Some(target) = target else {
                    return Err(RuntimeError::new("use declarations cannot appear outside of an entity"));
                };
                let Some(source_entity_kind) = self.entity_kinds.get(name) else {
                    return Err(RuntimeError::new(format!("no entity declaration named `{name}`")));
                };

                // Copy the contents of that entity declaration into this one
                let EntityKind { name: _, functions, constructor, tick_handler, draw_handler, ivars } = &**source_entity_kind;

                target.functions.extend(functions.clone());
                target.ivars.extend(ivars.clone());

                if let Some(source_constructor) = constructor.as_ref() {
                    if let Some(target_constructor) = target.constructor.as_mut() {
                        target_constructor.extend_from_slice(&source_constructor);
                    } else {
                        target.constructor = Some(source_constructor.clone());
                    }
                }
                if let Some(source_tick) = tick_handler.as_ref() {
                    if let Some(target_tick) = target.tick_handler.as_mut() {
                        target_tick.extend_from_slice(&source_tick);
                    } else {
                        target.tick_handler = Some(source_tick.clone());
                    }
                }

                // Extending the `draw` handler doesn't make much sense, because it is designed to return something, so only one will ever run. Don't do that
                if target.draw_handler.is_some() && draw_handler.is_some() {
                    return Err(RuntimeError::new(format!("both used entity and target entity define `draw`, but that is not possible to merge")));
                }

                Ok(())
            }
        }
    }

    /// If this is a `return`, returns [`ControlFlow::Break`] and the returned object
    pub fn interpret_statement(&mut self, stmt: &Statement, frame: &mut Frame) -> InterpreterResult<ControlFlow<Object>> {
        match stmt {
            Statement::Expression(expr) => {
                // We should generally read from this value - even though we aren't using it - to
                // bring out any errors for the value.
                // 
                // If we didn't do this, the statement expression `foobar;` would not error even if
                // `foobar` wasn't defined as a local. (It's a nonsense expression, but still.)
                self.interpret_expression(expr, frame)?.read()?;

                Ok(ControlFlow::Continue(()))
            }
            Statement::IfConditional { condition, true_body, false_body } => {
                let condition = self.interpret_expression(condition, frame)?.read()?;
                let Object::Boolean(condition) = condition else {
                    return Err(RuntimeError::new("if-condition must be a boolean"));
                };

                if condition {
                    self.execute_statement_body(&true_body, frame)
                } else if let Some(false_body) = false_body {
                    self.execute_statement_body(&false_body, frame)
                } else {
                    Ok(ControlFlow::Continue(()))
                }
            }
            Statement::EachLoop { variable, source, body } => {
                let source = self.interpret_expression(source, frame)?.read()?;
                
                let items = match source {
                    Object::Array(items) => items,
                    Object::Number(max) => (0..(max.round() as i64))
                        .map(|n| Object::Number(n as f64))
                        .collect(),
                    _ => return Err(RuntimeError::new("loop source must be an array or integer")),
                };

                for item in items {
                    frame.locals.insert(variable.clone(), item);
                    match self.execute_statement_body(body, frame)? {
                        ControlFlow::Continue(_) => {},
                        ControlFlow::Break(retval) => {
                            return Ok(ControlFlow::Break(retval));
                        },
                    }
                }

                Ok(ControlFlow::Continue(()))
            }
            Statement::Assignment { target, value } => {
                let value = self.interpret_expression(value, frame)?.read()?;
                self.interpret_expression(target, frame)?.write(value)?;
                Ok(ControlFlow::Continue(()))
            }
            Statement::Return(expr) => {
                if let Some(expr) = expr {
                    let retval = self.interpret_expression(expr, frame)?.read()?;
                    Ok(ControlFlow::Break(retval))
                } else {
                    Ok(ControlFlow::Break(Object::Null))
                }
            }
        }
    }

    pub fn interpret_expression<'a>(&'a mut self, expr: &'a Expression, frame: &'a mut Frame) -> InterpreterResult<Value<'a>> {
        match expr {
            Expression::ThisLiteral => {
                if let Some(entity) = frame.entity {
                    Ok(Value::ReadOnly(Object::Entity(entity)))
                } else {
                    Err(RuntimeError::new("`this` is not valid here"))
                }
            },

            Expression::NullLiteral => Ok(Value::ReadOnly(Object::Null)),
            Expression::NumberLiteral(n) => Ok(Value::ReadOnly(Object::Number(*n))),
            Expression::BooleanLiteral(b) => Ok(Value::ReadOnly(Object::Boolean(*b))),

            Expression::ArrayLiteral(items) => {
                let items = items.iter()
                    .map(|e| self.interpret_expression(e, frame).map(|v| v.read()).flatten())
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Value::ReadOnly(Object::Array(items)))
            }

            Expression::Identifier(id) => {
                // Special identifiers!
                match id.as_ref() {
                    "Input" => return Ok(Value::ReadOnly(Object::InputSingleton)),
                    "Display" => return Ok(Value::ReadOnly(Object::DisplaySingleton)),
                    "Math" => return Ok(Value::ReadOnly(Object::MathSingleton)),
                    _ => {}, // Carry on
                }

                // Look for entity kinds
                if let Some(kind) = self.entity_kinds.get(id) {
                    return Ok(Value::ReadOnly(Object::EntityKind(kind.clone())))
                }

                // Finally, locals
                if let Some(obj) = frame.locals.get(id) {
                    Ok(Value::ReadWrite {
                        value: obj.clone(),
                        write: Box::new(|o| {
                            frame.locals.insert(id.to_owned(), o);
                            Ok(())
                        }),
                    })
                } else {
                    Ok(Value::WriteOnly {
                        write: Box::new(|o| {
                            frame.locals.insert(id.to_owned(), o);
                            Ok(())
                        }),
                        error_on_read: RuntimeError::new(format!("undefined identifier `{id}`"))
                    })
                }
            },
            Expression::InstanceVarIdentifier(id) => {
                let Some(entity_id) = frame.entity else {
                    return Err(RuntimeError::new(format!("cannot get instance variable `{id}` in non-entity context")))
                };

                if let Some(obj) = self.entities[&entity_id].ivars.get(id) {
                    Ok(Value::ReadWrite {
                        value: obj.clone(),
                        write: Box::new(move |o| {
                            let entity = &mut self.entities.get_mut(&entity_id).unwrap();
                            entity.ivars.insert(id.to_owned(), o);
                            Ok(())
                        }),
                    })
                } else {
                    Err(RuntimeError::new(format!("undeclared instance variable `{id}`")))
                }    
            }

            Expression::SpriteLiteral(sprite) => Ok(Value::ReadOnly(Object::Sprite(sprite.clone()))),
            Expression::SoundLiteral(tone) => Ok(Value::ReadOnly(Object::Sound(tone.clone()))),

            Expression::FunctionCall { target, name, arguments } => {
                let target = self.interpret_expression(&target, frame)?.read()?;
                let arguments = arguments.iter()
                        .map(|arg| self.interpret_expression(arg, frame).map(|v| v.read()).flatten())
                        .collect::<Result<Vec<_>, _>>()?;
                
                Ok(Value::ReadOnly(target.call_function(self, name, arguments)?))
            }

            Expression::BinaryOperation { left, right, operator } => {
                fn to_boolean(obj: Object) -> InterpreterResult<bool> {
                    if let Object::Boolean(b) = obj {
                        Ok(b)
                    } else {
                        Err(RuntimeError::new(format!("both sides of logical operator must be booleans")))
                    }
                }

                // Boolean operators are special in that they short-circuit, so handle them early
                if let BinaryOperator::And = operator {
                    let left = to_boolean(self.interpret_expression(&left, frame)?.read()?)?;
                    if left {
                        let right = to_boolean(self.interpret_expression(&right, frame)?.read()?)?;
                        return Ok(Value::ReadOnly(Object::Boolean(right)));
                    } else {
                        // If LHS is false, the expression can never be true
                        return Ok(Value::ReadOnly(Object::Boolean(false)));
                    }
                }
                if let BinaryOperator::Or = operator {
                    let left = to_boolean(self.interpret_expression(&left, frame)?.read()?)?;
                    if left {
                        // If LHS is true, the expression will always be true
                        return Ok(Value::ReadOnly(Object::Boolean(true)));
                    } else {
                        let right = to_boolean(self.interpret_expression(&right, frame)?.read()?)?;
                        return Ok(Value::ReadOnly(Object::Boolean(right)));
                    }
                }

                let left = self.interpret_expression(&left, frame)?.read()?;
                let right = self.interpret_expression(&right, frame)?.read()?;

                fn numeric(left: Object, right: Object, f: impl FnOnce(f64, f64) -> Object) -> InterpreterResult<Object> {
                    let (Object::Number(left), Object::Number(right)) = (left, right) else {
                        return Err(RuntimeError::new(format!("both sides of binary operator must be numbers")));
                    };
                    Ok(f(left, right))
                }

                Ok(Value::ReadOnly(
                    match operator {
                        BinaryOperator::Add => numeric(left, right, |l, r| Object::Number(l + r))?,
                        BinaryOperator::Subtract => numeric(left, right, |l, r| Object::Number(l - r))?,
                        BinaryOperator::Multiply => numeric(left, right, |l, r| Object::Number(l * r))?,
                        BinaryOperator::Divide => numeric(left, right, |l, r| Object::Number(l / r))?,

                        BinaryOperator::Equals => Object::Boolean(left == right),
                        BinaryOperator::NotEquals => Object::Boolean(left != right),
                        BinaryOperator::LessThan => numeric(left, right, |l, r| Object::Boolean(l < r))?,
                        BinaryOperator::GreaterThan => numeric(left, right, |l, r| Object::Boolean(l > r))?,
                        BinaryOperator::LessThanOrEquals => numeric(left, right, |l, r| Object::Boolean(l <= r))?,
                        BinaryOperator::GreaterThanOrEquals => numeric(left, right, |l, r| Object::Boolean(l >= r))?,

                        // Handled earlier
                        BinaryOperator::And | BinaryOperator::Or => unreachable!(),
                    }
                ))
            }

            Expression::SpawnEntity { name } => {
                let Some(entity_kind) = self.entity_kinds.get(name).cloned() else {
                    return Err(RuntimeError::new(format!("no entity declaration named `{name}`")))
                };

                // Build new entity with dummy ivars
                let mut new_entity = Entity {
                    kind: entity_kind.clone(),
                    ivars: HashMap::new(),
                };
                for ivar in &entity_kind.ivars {
                    new_entity.ivars.insert(ivar.to_owned(), Object::Null);
                }

                let entity_id = EntityId(self.next_entity_id);
                self.next_entity_id += 1;

                self.entities.insert(entity_id, new_entity);
                self.entities_by_kinds.entry(name.clone()).or_default().insert(entity_id);

                // Execute constructor
                if let Some(constructor) = entity_kind.constructor.as_ref() {
                    let mut constructor_frame = Frame {
                        entity: Some(entity_id),
                        locals: HashMap::new(),
                    };
                    self.execute_statement_body(&constructor, &mut constructor_frame)?;
                }

                Ok(Value::ReadOnly(Object::Entity(entity_id)))
            }

            Expression::DestroyEntity(target) => {
                let target = self.interpret_expression(target, frame)?.read()?;
                let Object::Entity(entity_id) = target else {
                    return Err(RuntimeError::new(format!("used `destroy` on non-entity object: {}", target.describe(self))));
                };

                self.entities_pending_destroy.insert(entity_id);

                Ok(Value::ReadOnly(Object::Null))
            }

            Expression::Echo(target) => {
                let target = self.interpret_expression(target, frame)?.read()?;
                println!("{}", target.describe(self));
                Ok(Value::ReadOnly(target))
            }
        }
    }

    fn forbid_sound(&self) -> InterpreterResult {
        if !self.pending_sounds.is_empty() {
            return Err(RuntimeError::new("cannot play sound from anywhere other than `tick` (or a function it calls)"))
        }

        Ok(())
    }
}


/// Generic container for some kind of lvalue/rvalue.
/// 
/// In an rvalue context, this can typically be read to produce an [`Object`].
/// In a more limited set of cases, this can be used as an lvalue to assign an [`Object`] to something.
pub enum Value<'w> {
    ReadOnly(Object),
    WriteOnly {
        write: Box<dyn FnOnce(Object) -> InterpreterResult + 'w>,
        error_on_read: RuntimeError,
    },
    ReadWrite {
        value: Object,
        write: Box<dyn FnOnce(Object) -> InterpreterResult + 'w>,
    }
}

impl<'w> Value<'w> {
    pub fn read(self) -> InterpreterResult<Object> {
        match self {
            Value::ReadOnly(object) => Ok(object),
            Value::WriteOnly { error_on_read, .. } => Err(error_on_read),
            Value::ReadWrite { value, .. } => Ok(value),
        }
    }

    pub fn write(self, value: Object) -> InterpreterResult {
        match self {
            Value::ReadOnly(_) => Err(RuntimeError::new("expression cannot be target of an assignment")),
            Value::WriteOnly { write, .. } => {
                write(value)?;
                Ok(())
            },
            Value::ReadWrite { write, .. } => {
                write(value)?;
                Ok(())
            }
        }
    }
}

/// Uniquely refers to an entity. Allows entities to be passed around like objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(usize);

/// A specific instance of an entity.
pub struct Entity {
    pub kind: Rc<EntityKind>,
    pub ivars: HashMap<String, Object>,
}

impl Entity {
    pub fn draw_position_ivars(&self) -> InterpreterResult<(f64, f64)> {
        let Some(x) = self.ivars.get("x") else {
            return Err(RuntimeError::new("instance variable `x` must be declared when drawing a sprite"));
        };
        let Some(y) = self.ivars.get("y") else {
            return Err(RuntimeError::new("instance variable `y` must be declared when drawing a sprite"));
        };

        let (Object::Number(x), Object::Number(y)) = (x, y) else {
            return Err(RuntimeError::new("instance variables `x` and `y` must both be numbers"));
        };

        Ok((*x, *y))
    }
}

/// An entity definition which can be instantiated.
#[derive(Debug, Clone)]
pub struct EntityKind {
    pub name: String,
    pub functions: HashMap<String, FunctionDeclaration>,
    pub constructor: Option<Vec<Statement>>,
    pub tick_handler: Option<Vec<Statement>>,
    pub draw_handler: Option<Vec<Statement>>,
    pub ivars: Vec<String>,
}

impl PartialEq for EntityKind {
    fn eq(&self, other: &Self) -> bool {
        // The interpreter won't permit multiple kinds with the same name to be defined
        self.name == other.name
    }
}

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<Statement>,
}

pub struct DrawOperation {
    pub sprite: Sprite,
    pub x: f64,
    pub y: f64,
}

/// State of external game inputs.
/// 
/// As a "fantasy console", only a subset of keys are supported.
#[derive(Debug, Clone, Default)]
pub struct InputReport {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,

    pub x: bool,
    pub z: bool,
}

/// State of the display which this interpreter is rendering to. 
#[derive(Debug, Clone, Default)]
pub struct DisplayConfig {
    pub width: usize,
    pub height: usize,
}

pub struct Frame {
    /// Local variable definitions
    pub locals: HashMap<String, Object>,

    /// The current entity, for instance variable lookup
    pub entity: Option<EntityId>,
}

#[derive(Debug, Clone)]
pub struct RuntimeError(String);

impl RuntimeError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "runtime error: {}", self.0)
    }
}
impl Error for RuntimeError {}
