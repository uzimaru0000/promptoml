use crate::error::{Error, Result};
use crate::parser::{BinOp, Expr, UnaryOp, Value};
use std::collections::HashMap;

pub struct Context {
    variables: HashMap<String, Value>,
}

impl Context {
    pub fn new(args: HashMap<String, Value>) -> Self {
        Context {
            variables: HashMap::from_iter(vec![("args".to_string(), Value::Object(args))]),
        }
    }

    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get_context(&self) -> &HashMap<String, Value> {
        &self.variables
    }
}

pub fn eval(expr: &Expr, context: &Context) -> Result<Value> {
    match expr {
        Expr::Value(value) => Ok(value.clone()),

        Expr::UnaryOp { op, expr } => match op {
            UnaryOp::Dollar => {
                if let Expr::Value(Value::Symbol(name)) = expr.as_ref() {
                    context
                        .variables
                        .get(name)
                        .cloned()
                        .ok_or_else(|| Error::UndefinedVariable(name.clone()))
                } else {
                    Err(Error::TypeError(
                        "Dollar operator requires a symbol".to_string(),
                    ))
                }
            }
            UnaryOp::Not => {
                let value = eval(expr, context)?;
                match value {
                    Value::Boolean(b) => Ok(Value::Boolean(!b)),
                    _ => Err(Error::TypeError(
                        "Not operator requires a boolean".to_string(),
                    )),
                }
            }
        },

        Expr::BinaryOp { op, left, right } => {
            let left_val = eval(left, context)?;
            let right_val = eval(right, context)?;

            match op {
                BinOp::Eq => Ok(Value::Boolean(left_val == right_val)),
                BinOp::NotEq => Ok(Value::Boolean(left_val != right_val)),
                BinOp::Gt => Ok(Value::Boolean(left_val.gt(&right_val))),
                BinOp::Ge => Ok(Value::Boolean(left_val.ge(&right_val))),
                BinOp::Lt => Ok(Value::Boolean(left_val.lt(&right_val))),
                BinOp::Le => Ok(Value::Boolean(left_val.le(&right_val))),
                BinOp::Dot => match (&left_val, &right_val) {
                    (Value::Object(obj), Value::Symbol(field)) => {
                        obj.get(field).cloned().ok_or_else(|| {
                            Error::TypeError(format!("Field '{}' not found in object", field))
                        })
                    }
                    _ => Err(Error::TypeError(
                        "Dot operator requires an object and a field name".to_string(),
                    )),
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_eval_basic() {
        let mut context = Context::new(HashMap::new());
        context.set_variable("x".to_string(), Value::Number(42.0));
        context.set_variable(
            "obj".to_string(),
            Value::Object(HashMap::from([(
                "field".to_string(),
                Value::String("Hello World".to_string()),
            )])),
        );

        // Test variable access
        let expr = parse("$x").unwrap();
        assert_eq!(eval(&expr, &context).unwrap(), Value::Number(42.0));

        // Test comparison
        let expr = parse("42 == 42").unwrap();
        assert_eq!(eval(&expr, &context).unwrap(), Value::Boolean(true));

        // Test object field access
        let expr = parse("$obj.field").unwrap();
        assert_eq!(
            eval(&expr, &context).unwrap(),
            Value::String("Hello World".to_string())
        );
    }
}
