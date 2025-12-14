use std::{collections::HashMap, error::Error, fmt::Display, ops::ControlFlow, rc::Rc};

use crate::{BinaryOperator, Declaration, Expression, Statement};

pub struct Interpreter {
    game_init: Vec<Statement>,

    entities: HashMap<EntityId, Entity>,
    next_entity_id: usize,

    entity_kinds: HashMap<String, Rc<EntityKind>>,
}

pub type InterpreterResult<T = ()> = Result<T, RuntimeError>;

impl Interpreter {
    pub fn new() -> Self {
        Self {
            game_init: vec![],
            entities: HashMap::new(),
            next_entity_id: 1,
            entity_kinds: HashMap::new(),
        }
    }

    pub fn execute_init(&mut self) -> InterpreterResult {
        let mut frame = Frame {
            entity: None,
            locals: HashMap::new(),
        };

        self.execute_statement_body(&self.game_init.clone(), &mut frame)?;
        Ok(())
    }

    fn execute_statement_body(&mut self, body: &[Statement], frame: &mut Frame) -> InterpreterResult<Object> {
        for stmt in body {
            match self.interpret_statement(stmt, frame)? {
                ControlFlow::Break(retval) => return Ok(retval),
                ControlFlow::Continue(_) => {},
            }
        }

        Ok(Object::Null)
    }

    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }

    pub fn interpret_declaration(&mut self, decl: &Declaration, target: Option<&mut EntityKind>) -> InterpreterResult {
        match decl {
            Declaration::GameInitDeclaration { body } => {
                if !self.game_init.is_empty() {
                    return Err(RuntimeError::new("game initialiser already defined"));
                }
                self.game_init = body.clone();
                Ok(())
            }

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
                    ivars: vec![],
                };

                for subdecl in body {
                    self.interpret_declaration(subdecl, Some(&mut new_entity_kind))?;
                }

                self.entity_kinds.insert(name.to_owned(), Rc::new(new_entity_kind));
                Ok(())
            }

            Declaration::ConstructorDeclaration { body } => {
                let Some(target) = target else {
                    return Err(RuntimeError::new("constructor declarations cannot appear outside of an entity"));
                };
                if target.constructor.is_some() {
                    return Err(RuntimeError::new(format!("constructor is already declared")));
                }

                target.constructor = Some(body.clone());
                Ok(())
            }

            Declaration::InstanceVarDeclaration { name } => {
                let Some(target) = target else {
                    return Err(RuntimeError::new("instance variable declarations cannot appear outside of an entity"));
                };
                if target.ivars.contains(name) {
                    return Err(RuntimeError::new(format!("instance variable `{name}` is already declared")));
                }

                target.ivars.push(name.to_owned());
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
            Statement::Assignment { target, value } => {
                let value = self.interpret_expression(value, frame)?.read()?;
                self.interpret_expression(target, frame)?.write(value)?;
                Ok(ControlFlow::Continue(()))
            }
            Statement::Return(expr) => {
                let retval = self.interpret_expression(expr, frame)?.read()?;
                Ok(ControlFlow::Break(retval))
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

            Expression::Identifier(id) => {
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
                    Ok(Value::WriteOnly {
                        write: Box::new(move |o| {
                            let entity = &mut self.entities.get_mut(&entity_id).unwrap();
                            entity.ivars.insert(id.to_owned(), o);
                            Ok(())
                        }),
                        error_on_read: RuntimeError::new(format!("undefined instance variable `{id}`"))
                    })
                }
                
            }

            Expression::FunctionCall { target, name, arguments } => {
                let target = self.interpret_expression(&target, frame)?.read()?;
                let Object::Entity(entity_id) = target else {
                    return Err(RuntimeError::new(format!("function calls can only target entities")));
                };

                let entity_kind = self.entities[&entity_id].kind.clone();
                let Some(FunctionDeclaration { parameters, body, .. }) = entity_kind.functions.get(name) else {
                    return Err(RuntimeError::new(format!("entity declaration `{}` has no function named `{}`", entity_kind.name, name)));
                };

                if parameters.len() != arguments.len() {
                    return Err(RuntimeError::new(format!("function declaration for `{}` has {} parameters, but {} arguments were provided", name, parameters.len(), arguments.len())));
                }

                let arguments = arguments.iter()
                    .map(|arg| self.interpret_expression(arg, frame).map(|v| v.read()).flatten())
                    .collect::<Result<Vec<_>, _>>()?;

                let mut frame = Frame {
                    entity: Some(entity_id),
                    locals: parameters.iter().cloned().zip(arguments).collect(),
                };

                Ok(Value::ReadOnly(self.execute_statement_body(&body, &mut frame)?))
            }

            Expression::BinaryOperation { left, right, operator } => {
                let left = self.interpret_expression(&left, frame)?.read()?;
                let right = self.interpret_expression(&right, frame)?.read()?;

                let (Object::Number(left), Object::Number(right)) = (left, right) else {
                    return Err(RuntimeError::new(format!("both sides of binary operator must be numbers")));
                };

                Ok(Value::ReadOnly(Object::Number(
                    match operator {
                        BinaryOperator::Add => left + right,
                        BinaryOperator::Subtract => left - right,
                        BinaryOperator::Multiply => left * right,
                        BinaryOperator::Divide => left / right,
                    }
                )))
            }

            Expression::AddEntity { name } => {
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
        }
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

/// Some generic object which can be passed around the interpreter.
#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Null,
    Number(f64),
    Entity(EntityId),
}

/// Uniquely refers to an entity. Allows entities to be passed around like objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(usize);

/// A specific instance of an entity.
pub struct Entity {
    pub kind: Rc<EntityKind>,
    pub ivars: HashMap<String, Object>,
}

/// An entity definition which can be instantiated.
pub struct EntityKind {
    name: String,
    functions: HashMap<String, FunctionDeclaration>,
    constructor: Option<Vec<Statement>>,
    ivars: Vec<String>,
}

pub struct FunctionDeclaration {
    name: String,
    parameters: Vec<String>,
    body: Vec<Statement>,
}

pub struct Frame {
    /// Local variable definitions
    locals: HashMap<String, Object>,

    /// The current entity, for instance variable lookup
    entity: Option<EntityId>,
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
