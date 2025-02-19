use std::collections::HashMap;

use serde::Deserialize;
use toml::Table;

use crate::{
    condition::Condition,
    error::{Error, Result},
    parser::parse,
    prompt::{
        ConfirmPrompt, FuzzySelectPrompt, MultiSelectPrompt, PasswordPrompt, PromptType,
        SelectPrompt, TextPrompt,
    },
    state::{Node, State},
};

#[derive(Debug, Deserialize)]
struct Config {
    start: String,
    state: HashMap<String, StateConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum StateConfig {
    #[serde(rename = "text")]
    Text {
        name: String,
        message: String,
        to: String,
    },
    #[serde(rename = "confirm")]
    Confirm {
        name: String,
        message: String,
        to: String,
    },
    #[serde(rename = "password")]
    Password {
        name: String,
        message: String,
        to: String,
    },
    #[serde(rename = "select")]
    Select {
        name: String,
        message: String,
        options: Vec<String>,
        to: String,
    },
    #[serde(rename = "multi_select")]
    MultiSelect {
        name: String,
        message: String,
        options: Vec<String>,
        to: String,
    },
    #[serde(rename = "fuzzy_select")]
    FuzzySelect {
        name: String,
        message: String,
        options: Vec<String>,
        to: String,
    },
    #[serde(rename = "condition")]
    Condition {
        name: String,
        condition: String,
        branches: Table,
    },
    #[serde(rename = "done")]
    Done,
}

pub fn load(content: &str) -> Result<(String, HashMap<String, Node>)> {
    let config: Config = toml::from_str(content).map_err(|e| Error::ParseError(e.to_string()))?;

    let mut nodes = HashMap::new();
    for (key, state_config) in config.state {
        let (state, name) = match state_config {
            StateConfig::Text { name, message, to } => (
                State::Prompt(PromptType::Text(TextPrompt { message }), to),
                name,
            ),
            StateConfig::Confirm { name, message, to } => (
                State::Prompt(PromptType::Confirm(ConfirmPrompt { message }), to),
                name,
            ),
            StateConfig::Password { name, message, to } => (
                State::Prompt(PromptType::Password(PasswordPrompt { message }), to),
                name,
            ),
            StateConfig::Select {
                name,
                message,
                options,
                to,
            } => {
                let options = options
                    .iter()
                    .map(|s| parse(s))
                    .collect::<Result<Vec<_>>>()?;

                (
                    State::Prompt(PromptType::Select(SelectPrompt { message, options }), to),
                    name,
                )
            }
            StateConfig::MultiSelect {
                name,
                message,
                options,
                to,
            } => {
                let options = options
                    .iter()
                    .map(|s| parse(s))
                    .collect::<Result<Vec<_>>>()?;

                (
                    State::Prompt(
                        PromptType::MultiSelect(MultiSelectPrompt { message, options }),
                        to,
                    ),
                    name,
                )
            }
            StateConfig::FuzzySelect {
                name,
                message,
                options,
                to,
            } => {
                let options = options
                    .iter()
                    .map(|s| parse(s))
                    .collect::<Result<Vec<_>>>()?;

                (
                    State::Prompt(
                        PromptType::FuzzySelect(FuzzySelectPrompt { message, options }),
                        to,
                    ),
                    name,
                )
            }
            StateConfig::Condition {
                name,
                condition,
                branches,
            } => {
                let branches: HashMap<String, String> = branches
                    .into_iter()
                    .map(|(k, v)| {
                        let branch = match v.as_str() {
                            Some(s) => s.to_string(),
                            None => {
                                return Err(Error::ParseError(
                                    "Branch must be a string".to_string(),
                                ))
                            }
                        };

                        Ok((k, branch))
                    })
                    .collect::<Result<_>>()?;

                let condition = parse(&condition)?;
                let mut parsed_branches = HashMap::new();
                for (k, v) in branches {
                    parsed_branches.insert(k, parse(&v)?);
                }

                (
                    State::Condition(Condition {
                        condition,
                        branches: parsed_branches,
                    }),
                    name,
                )
            }
            StateConfig::Done => (State::Done, key.clone()),
        };

        nodes.insert(key.clone(), Node { name, state });
    }

    Ok((config.start, nodes))
}

#[cfg(test)]
mod tests {
    use crate::parser::{Expr, Value};

    use super::*;

    #[test]
    fn test_load_config() {
        let content = r#"
            start = "name"

            [state.name]
            type = "text"
            name = "name"
            message = "What is your name?"
            to = "age"

            [state.age]
            type = "text"
            name = "age"
            message = "What is your age?"
            to = "age_condition"

            [state.age_condition]
            type = "condition"
            name = "age_condition"
            condition = "$age > 18"
            branches = { true = "gender", false = "end" }

            [state.gender]
            type = "select"
            name = "gender"
            message = "What is your gender?"
            options = ["Male", "Female", "Other"]
            to = "end"

            [state.end]
            type = "done"
        "#;

        let (start, nodes) = load(content).unwrap();
        assert_eq!(start, "name");
        assert_eq!(nodes.len(), 5);

        // nameノードのチェック
        let name_node = nodes.get("name").unwrap();
        match &name_node.state {
            State::Prompt(PromptType::Text(_), to) => assert_eq!(to, "age"),
            _ => panic!("Expected text prompt"),
        }

        // age_conditionノードのチェック
        let condition_node = nodes.get("age_condition").unwrap();
        match &condition_node.state {
            State::Condition(condition) => {
                assert_eq!(
                    condition.condition,
                    Expr::BinaryOp {
                        op: crate::parser::BinOp::Gt,
                        left: Box::new(Expr::UnaryOp {
                            op: crate::parser::UnaryOp::Dollar,
                            expr: Box::new(Expr::Value(Value::Symbol("age".to_string()))),
                        }),
                        right: Box::new(Expr::Value(Value::Number(18.0))),
                    }
                );
                assert_eq!(
                    condition.branches.get("true").unwrap(),
                    &Expr::Value(Value::Symbol("gender".to_string()))
                );
                assert_eq!(
                    condition.branches.get("false").unwrap(),
                    &Expr::Value(Value::Symbol("end".to_string()))
                );
            }
            _ => panic!("Expected condition"),
        }

        // endノードのチェック
        let end_node = nodes.get("end").unwrap();
        assert!(matches!(end_node.state, State::Done));
    }
}
