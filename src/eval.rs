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

    pub fn remove_variable(&mut self, name: String) {
        self.variables.remove(&name);
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
                BinOp::Index => match (&left_val, &right_val) {
                    (Value::Array(arr), Value::Number(index)) => {
                        let index = (*index) as usize;
                        arr.get(index).cloned().ok_or_else(|| {
                            Error::IndexOutOfBounds(format!("Index {} is out of bounds", index))
                        })
                    }
                    (Value::String(s), Value::Number(index)) => {
                        let index = (*index) as usize;
                        let c = s.chars().nth(index).ok_or_else(|| {
                            Error::IndexOutOfBounds(format!("Index {} is out of bounds", index))
                        })?;
                        Ok(Value::String(c.to_string()))
                    }
                    (Value::Object(obj), Value::String(field)) => {
                        obj.get(field).cloned().ok_or_else(|| {
                            Error::TypeError(format!("Field '{}' not found in object", field))
                        })
                    }
                    _ => Err(Error::TypeError(
                        "Index operator requires an array and a number".to_string(),
                    )),
                },
            }
        },

        Expr::Function { name, args } => {
            let mut arg_vals = Vec::new();
            for arg in args {
                let arg_val = eval(&arg, context)?;
                arg_vals.push(arg_val);
            }

            match name.as_str() {
                "keys" => {
                    if let Some(Value::Object(obj)) = arg_vals.get(0) {
                        Ok(Value::Array(
                            obj.keys().map(|k| Value::String(k.clone())).collect(),
                        ))
                    } else {
                        Err(Error::TypeError("keys function requires an object".to_string()))
                    }
                }
                "len" => {
                    if let Some(Value::String(s)) = arg_vals.get(0) {
                        Ok(Value::Number(s.len() as f64))
                    } else if let Some(Value::Array(arr)) = arg_vals.get(0) {
                        Ok(Value::Number(arr.len() as f64))
                    } else {
                        Err(Error::TypeError(
                            "len function requires a string or array".to_string(),
                        ))
                    }
                }
                "split" => {
                    match (arg_vals.get(0), arg_vals.get(1)) {
                        (Some(Value::String(s)), Some(Value::String(sep))) => {
                            Ok(Value::Array(s.split(sep).map(|s| Value::String(s.to_string())).collect()))
                        }
                        _ => Err(Error::TypeError("split function requires a string and a separator".to_string())),
                    }
                }
                _ => Err(Error::TypeError(format!(
                    "Unknown function: {}",
                    name
                ))),
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

    #[test]
    fn test_eval_index() {
        let mut context = Context::new(HashMap::new());
        context.set_variable("x".to_string(), Value::Array(vec![
            Value::String("Hello".to_string()),
            Value::String("World".to_string()),
        ]));

        let expr = parse("$x[0]").unwrap();
        assert_eq!(eval(&expr, &context).unwrap(), Value::String("Hello".to_string()));
    }

    #[test]
    fn test_eval_index_out_of_bounds() {
        let mut context = Context::new(HashMap::new());
        context.set_variable("x".to_string(), Value::Array(vec![]));

        let expr = parse("$x[0]").unwrap();
        assert_eq!(eval(&expr, &context).unwrap_err(), Error::IndexOutOfBounds("Index 0 is out of bounds".to_string()));
    }

    #[test]
    fn test_eval_index_string() {
        let mut context = Context::new(HashMap::new());
        context.set_variable("x".to_string(), Value::String("Hello".to_string()));

        let expr = parse("$x[0]").unwrap();
        assert_eq!(eval(&expr, &context).unwrap(), Value::String("H".to_string()));
    }

    #[test]
    fn test_eval_function_keys() {
        let mut context = Context::new(HashMap::new());
        context.set_variable("obj".to_string(), Value::Object(HashMap::from([
            ("key1".to_string(), Value::String("value1".to_string())),
            ("key2".to_string(), Value::String("value2".to_string())),
        ])));

        let expr = parse("keys($obj)").unwrap();
        assert_eq!(eval(&expr, &context).unwrap(), Value::Array(vec![
            Value::String("key1".to_string()),
            Value::String("key2".to_string()),
        ]));
    }

    #[test]
    fn test_eval_function_len() {
        let mut context = Context::new(HashMap::new());
        context.set_variable("arr".to_string(), Value::Array(vec![
            Value::String("Hello".to_string()),
            Value::String("World".to_string()),
        ]));

        let expr = parse("len($arr)").unwrap();
        assert_eq!(eval(&expr, &context).unwrap(), Value::Number(2.0));
    }

    #[test]
    fn test_eval_function_split() {
        let mut context = Context::new(HashMap::new());
        context.set_variable("str".to_string(), Value::String("Hello,World".to_string()));
        
        let expr = parse("split($str, ',')").unwrap();
        assert_eq!(eval(&expr, &context).unwrap(), Value::Array(vec![
            Value::String("Hello".to_string()),
            Value::String("World".to_string()),
        ]));
    }
}
