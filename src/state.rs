use std::collections::HashMap;

use crate::{
    condition::Condition,
    error::{Error, Result},
    eval::{eval, Context},
    parser::Expr,
    prompt::{Prompt, PromptType},
};

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Prompt(PromptType, String),
    Condition(Condition),
    Set(Expr, String),
    Remove(String),
    Done,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub state: State,
}

pub struct StateMachine {
    pub context: Context,
    pub nodes: HashMap<String, Node>,
}

impl StateMachine {
    pub fn new(nodes: HashMap<String, Node>, context: Context) -> Self {
        Self { nodes, context }
    }

    pub fn run(&mut self, start: String) -> Result<()> {
        let mut current_node = self
            .nodes
            .get(&start)
            .ok_or(Error::InvalidTransition(format!(
                "Invalid start node: {}",
                start
            )))?;
        while current_node.state != State::Done {
            match &current_node.state {
                State::Prompt(prompt, to) => {
                    prompt.run(&current_node.name, &mut self.context)?;
                    current_node = self.nodes.get(to).ok_or(Error::InvalidTransition(format!(
                        "Invalid transition from {} to {}",
                        current_node.name, to
                    )))?;
                }
                State::Condition(condition) => {
                    let result = condition.eval(&self.context)?;
                    current_node =
                        self.nodes
                            .get(&result)
                            .ok_or(Error::InvalidTransition(format!(
                                "Invalid transition from {} to {}",
                                current_node.name, result
                            )))?;
                }
                State::Set(expr, to) => {
                    let value = eval(expr, &self.context)?;
                    self.context.set_variable(current_node.name.clone(), value);
                    current_node = self.nodes.get(to).ok_or(Error::InvalidTransition(format!(
                        "Invalid transition from {} to {}",
                        current_node.name, to
                    )))?;
                }
                State::Remove(to) => {
                    self.context.remove_variable(current_node.name.clone());
                    current_node = self.nodes.get(to).ok_or(Error::InvalidTransition(format!(
                        "Invalid transition from {} to {}",
                        current_node.name, to
                    )))?;
                }
                State::Done => {
                    break;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::prompt::TextPrompt;

    use super::*;

    #[test]
    fn test_state_machine() {
        let nodes = vec![
            (
                "start".to_string(),
                Node {
                    name: "start".to_string(),
                    state: State::Prompt(
                        PromptType::Text(TextPrompt {
                            message: "What is your name?".to_string(),
                        }),
                        "end".to_string(),
                    ),
                },
            ),
            (
                "end".to_string(),
                Node {
                    name: "end".to_string(),
                    state: State::Done,
                },
            ),
        ]
        .into_iter()
        .collect();

        let mut sm = StateMachine::new(nodes, Context::new(HashMap::new()));
        sm.run("start".to_string()).unwrap();
    }
}
