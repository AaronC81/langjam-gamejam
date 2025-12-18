use std::{ops::ControlFlow, rc::Rc};

use crate::{EntityId, EntityKind, Frame, FunctionDeclaration, Interpreter, InterpreterResult, RuntimeError, Sprite, Tone};


/// Some generic object which can be passed around the interpreter.
#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Null,
    Number(f64),
    Boolean(bool),
    Entity(EntityId),
    EntityKind(Rc<EntityKind>),
    Sprite(Sprite),
    Sound(Tone),
    Array(Vec<Object>),

    InputSingleton,
    DisplaySingleton,
    MathSingleton,
}

impl Object {
    pub fn call_function(&self, interpreter: &mut Interpreter, name: &str, arguments: Vec<Object>) -> InterpreterResult<Object> {
        match self {
            Object::Entity(entity_id) => {
                let entity_kind = interpreter.entities[&entity_id].kind.clone();
                let Some(FunctionDeclaration { parameters, body, .. }) = entity_kind.functions.get(name) else {
                    return Err(RuntimeError::new(format!("entity declaration `{}` has no function named `{}`", entity_kind.name, name)));
                };

                if parameters.len() != arguments.len() {
                    Self::incorrect_arity(name, parameters.len(), arguments.len())?;
                }

                let mut frame = Frame {
                    entity: Some(*entity_id),
                    locals: parameters.iter().cloned().zip(arguments).collect(),
                };

                let retval = match interpreter.execute_statement_body(&body, &mut frame)? {
                    ControlFlow::Break(obj) => obj,
                    ControlFlow::Continue(_) => Object::Null,
                };
                Ok(retval)
            },

            Object::EntityKind(kind) => {
                // All `EntityKind` functions take no parameters
                if arguments.len() != 0 {
                    Self::incorrect_arity(name, 0, arguments.len())?;
                }

                match name {
                    "all" => {
                        let entities_of_kind = interpreter.entities.iter()
                            .filter_map(|(id, e)|
                                if e.kind == *kind {
                                    Some(Object::Entity(*id))
                                } else {
                                    None
                                }
                            )
                            .collect::<Vec<_>>();

                        Ok(Object::Array(entities_of_kind))
                    },

                    _ => Err(RuntimeError::new(format!("`{}` has no function named `{}`", self.describe(interpreter), name))),
                }
            },

            Object::Sprite(sprite) => {
                // All `Sprite` functions take no parameters
                if arguments.len() != 0 {
                    Self::incorrect_arity(name, 0, arguments.len())?;
                }

                match name {
                    "width" => Ok(Object::Number(sprite.width as f64)),
                    "height" => Ok(Object::Number(sprite.height as f64)),

                    _ => Err(RuntimeError::new(format!("sprite has no function named `{}`", name))),
                }
            }

            Object::Sound(sound) => {
                // All `Sound` functions take no parameters
                if arguments.len() != 0 {
                    Self::incorrect_arity(name, 0, arguments.len())?;
                }

                match name {
                    "play" => {
                        interpreter.pending_sounds.push(sound.clone());
                        Ok(Object::Null)
                    }

                    _ => Err(RuntimeError::new(format!("sound has no function named `{}`", name))),
                }
            }

            Object::InputSingleton => {
                // All `Input` functions take no parameters
                if arguments.len() != 0 {
                    Self::incorrect_arity(name, 0, arguments.len())?;
                }

                match name {
                    "up_pressed" => Ok(Object::Boolean(interpreter.input_report.up)),
                    "down_pressed" => Ok(Object::Boolean(interpreter.input_report.down)),
                    "left_pressed" => Ok(Object::Boolean(interpreter.input_report.left)),
                    "right_pressed" => Ok(Object::Boolean(interpreter.input_report.right)),
                    "x_pressed" => Ok(Object::Boolean(interpreter.input_report.x)),
                    "z_pressed" => Ok(Object::Boolean(interpreter.input_report.z)),

                    _ => Err(RuntimeError::new(format!("`Input` has no function named `{}`", name))),
                }
            }

            Object::DisplaySingleton => {
                // All `Display` functions take no parameters
                if arguments.len() != 0 {
                    Self::incorrect_arity(name, 0, arguments.len())?;
                }

                match name {
                    "width" => Ok(Object::Number(interpreter.display_config.width as f64)),
                    "height" => Ok(Object::Number(interpreter.display_config.height as f64)),

                    _ => Err(RuntimeError::new(format!("`Display` has no function named `{}`", name))),
                }
            }

            Object::MathSingleton => {
                match name {
                    // `random_int(start, end)` returns a random integer between `start` and `end`
                    // (inclusive on both sides)
                    "random_int" => {
                        let [start, end] = arguments.as_slice() else {
                            Self::incorrect_arity(name, 2, arguments.len())?;
                        };
                        let (Object::Number(start), Object::Number(end)) = (start, end) else {
                            return Err(RuntimeError::new("arguments to `Math.random_int` must be numbers"));
                        };

                        let value = rand::random_range((start.round() as i64)..=(end.round() as i64)) as f64;
                        Ok(Object::Number(value))
                    },

                    _ => Err(RuntimeError::new(format!("`Math` has no function named `{}`", name))),
                }
            }

            _ => Err(RuntimeError::new(format!("cannot call function `{name}` on an object that doesn't have functions"))),
        }
    }

    fn incorrect_arity(name: &str, expected: usize, actual: usize) -> Result<!, RuntimeError> {
        Err(RuntimeError::new(format!("function declaration for `{}` has {} parameters, but {} arguments were provided", name, expected, actual)))
    }

    pub fn describe(&self, interpreter: &Interpreter) -> String {
        match self {
            Object::Null => "null".to_owned(),
            Object::Number(n) => n.to_string(),
            Object::Boolean(b) => b.to_string(),
            Object::Entity(entity_id) => {
                if let Some(entity) = interpreter.entities.get(&entity_id) {
                    let ivars = entity.ivars.iter()
                        .map(|(k, v)| format!("{}={}", k, v.describe(interpreter)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("Entity {} ({})", entity.kind.name, ivars)
                } else {
                    "destroyed entity".to_owned()
                }
            },
            Object::EntityKind(kind) => {
                format!("Entity Declaration {}", kind.name)
            },
            Object::Sprite(sprite) =>
                format!("sprite ({}x{})", sprite.width, sprite.height),
            Object::Sound(tone) =>
                format!("sound: {tone:?}"),
            Object::Array(items) => {
                if items.is_empty() {
                    "[ ]".to_string()
                } else {
                    format!("[ {} ]", items.iter().map(|i| i.describe(interpreter)).collect::<Vec<_>>().join(", "))
                }
            },
            
            Object::InputSingleton => "Input".to_owned(),
            Object::DisplaySingleton => "Display".to_owned(),
            Object::MathSingleton => "Math".to_owned(),
        }
    }
}
