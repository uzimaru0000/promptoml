use crate::eval::Context;
use crate::parser::Value;
use crate::{eval::eval, parser::Expr};
use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct Goto {
    pub target: Expr,
}

impl Goto {
    pub fn new(target: Expr) -> Self {
        Self { target }
    }

    pub fn eval(&self, context: &Context) -> Result<String> {
        let target = eval(&self.target, context)?;

        match target {
            Value::String(s) => Ok(s),
            _ => Err(Error::InvalidTransition(format!(
                "Goto target must be a string, got {:?}",
                target
            ))),
        }
    }
}
