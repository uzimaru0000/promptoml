use std::collections::HashMap;

use promptoml::{eval::Context, parser::Value, state::StateMachine, toml::load};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML error: {0}")]
    Toml(#[from] promptoml::error::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, clap::Parser)]
pub struct App {
    #[clap(short, long)]
    config: String,
    args: Option<String>
}

impl App {
    pub fn run(&self) -> Result<HashMap<String, serde_json::Value>, AppError> {
        let config = std::fs::read_to_string(&self.config).map_err(AppError::Io)?;
        let (start, nodes) = load(&config).map_err(AppError::Toml)?;

        let args = self.args.clone().unwrap_or("{}".to_string());
        let args: HashMap<String, serde_json::Value> = serde_json::from_str(&args).map_err(AppError::Json)?;
        let args = args.into_iter().map(|(k, v)| match v {
            serde_json::Value::String(s) => (k, Value::String(s)),
            serde_json::Value::Number(n) => (k, Value::Number(n.as_f64().unwrap())),
            serde_json::Value::Bool(b) => (k, Value::Boolean(b)),
            _ => (k, Value::String(v.to_string())),
        }).collect();

        let context = Context::new(args);
        let mut state = StateMachine::new(nodes, context);

        state.run(start)?;

        let mut context = state.context.get_context().clone();
        context.remove("args");

        Ok(context.into_iter().map(|(k, v)| (k, to_serde_json(v))).collect())
    }
}

fn to_serde_json(value: Value) -> serde_json::Value {
    match value {
        Value::String(s) => serde_json::Value::String(s),
        Value::Number(n) => serde_json::Value::Number(serde_json::Number::from_f64(n).unwrap()),
        Value::Boolean(b) => serde_json::Value::Bool(b),
        Value::Symbol(s) => serde_json::Value::String(s),
        Value::Object(o) => serde_json::Value::Object(o.into_iter().map(|(k, v)| (k, to_serde_json(v))).collect()),
        Value::Array(a) => serde_json::Value::Array(a.into_iter().map(|v| to_serde_json(v)).collect()),
    }
}
