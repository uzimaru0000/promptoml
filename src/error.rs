use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Type error: {0}")]
    TypeError(String),
    
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Unexpected trailing input: {0}")]
    TrailingInput(String),

    #[error("Failed to create prompt: {0}")]
    FailedToCreatePrompt(String),

    #[error("Failed to run prompt: {0}")]
    FailedToRunPrompt(String),

    #[error("Missing branch: {0}")]
    MissingBranch(String),

    #[error("Invalid transition: {0}")]
    InvalidTransition(String),
}

pub type Result<T> = std::result::Result<T, Error>; 
