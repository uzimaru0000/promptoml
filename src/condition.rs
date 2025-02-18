use std::collections::HashMap;

use crate::{
    error::{Error, Result},
    eval::{eval, Context},
    parser::{Expr, Value},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    pub condition: Expr,
    pub branches: HashMap<String, Expr>,
}

impl Condition {
    pub fn eval(&self, context: &Context) -> Result<String> {
        let cond = eval(&self.condition, context)?;

        let result = match cond {
            Value::Boolean(b) => {
                if b {
                    Ok(self
                        .branches
                        .get("true")
                        .ok_or(Error::MissingBranch("true".to_string()))?)
                } else {
                    Ok(self
                        .branches
                        .get("false")
                        .ok_or(Error::MissingBranch("false".to_string()))?)
                }
            }
            _ => Err(Error::TypeError(
                "Condition must evaluate to a boolean".to_string(),
            )),
        }?;

        let result = eval(&result, context)?;
        match result {
            Value::String(s) => Ok(s),
            Value::Symbol(s) => Ok(s),
            Value::Number(_) => Err(Error::TypeError(
                "Condition must evaluate to a string".to_string(),
            )),
            Value::Boolean(_) => Err(Error::TypeError(
                "Condition must evaluate to a string".to_string(),
            )),
            Value::Object(_) => Err(Error::TypeError(
                "Condition must evaluate to a string".to_string(),
            )),
            Value::Array(_) => Err(Error::TypeError(
                "Condition must evaluate to a string".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_eval() {
        let cond = Condition {
            condition: Expr::Value(Value::Boolean(true)),
            branches: HashMap::from([
                (
                    "true".to_string(),
                    Expr::Value(Value::String("true".to_string())),
                ),
                (
                    "false".to_string(),
                    Expr::Value(Value::String("false".to_string())),
                ),
            ]),
        };

        let context = Context::new(HashMap::new());
        let result = cond.eval(&context).unwrap();
        assert_eq!(result, "true");
    }

    #[test]
    fn test_condition_eval_false() {
        let cond = Condition {
            condition: Expr::Value(Value::Boolean(false)),
            branches: HashMap::from([
                (
                    "true".to_string(),
                    Expr::Value(Value::String("true".to_string())),
                ),
                (
                    "false".to_string(),
                    Expr::Value(Value::String("false".to_string())),
                ),
            ]),
        };

        let context = Context::new(HashMap::new());
        let result = cond.eval(&context).unwrap();
        assert_eq!(result, "false");
    }

    #[test]
    fn test_condition_eval_type_error() {
        let cond = Condition {
            condition: Expr::Value(Value::String("true".to_string())),
            branches: HashMap::new(),
        };

        let context = Context::new(HashMap::new());
        let result = cond.eval(&context);
        assert_eq!(
            result,
            Err(Error::TypeError(
                "Condition must evaluate to a boolean".to_string()
            ))
        );
    }

    #[test]
    fn test_condition_eval_missing_branch() {
        let cond = Condition {
            condition: Expr::Value(Value::Boolean(true)),
            branches: HashMap::new(),
        };

        let context = Context::new(HashMap::new());
        let result = cond.eval(&context);
        assert_eq!(result, Err(Error::MissingBranch("true".to_string())));
    }
}
