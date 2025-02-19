use crate::{
    error::{Error, Result},
    eval::Context,
    parser::Value,
};

pub fn get_options(options: Vec<Value>, context: &mut Context) -> Result<Vec<String>> {
    let mut opts = Vec::new();
    for option in options {
        match option {
            Value::String(s) => opts.push(s),
            Value::Number(n) => opts.push(n.to_string()),
            Value::Boolean(b) => opts.push(b.to_string()),
            Value::Symbol(s) => opts.push(s),
            Value::Array(a) => {
                opts.extend(get_options(a, context)?);
            }
            _ => {
                return Err(Error::TypeError(format!(
                    "Invalid option type: {:?}",
                    option
                )))
            }
        }
    }

    Ok(opts)
}
