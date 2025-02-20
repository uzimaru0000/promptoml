use crate::{error::{Error, Result}, eval::{eval, Context}, parser::{Expr, Value}, utils::get_options};
use promkit::preset::{
    checkbox::Checkbox, confirm::Confirm, listbox::Listbox, password::Password, query_selector::QuerySelector, readline::Readline
};

pub trait Prompt {
    fn run(&self, name: &str, context: &mut Context) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum PromptType {
    Text(TextPrompt),
    Confirm(ConfirmPrompt),
    Password(PasswordPrompt),
    Select(SelectPrompt),
    MultiSelect(MultiSelectPrompt),
    FuzzySelect(FuzzySelectPrompt),
}

impl Prompt for PromptType {
    fn run(&self, name: &str, context: &mut Context) -> Result<()> {
        match self {
            PromptType::Text(prompt) => prompt.run(name, context),
            PromptType::Confirm(prompt) => prompt.run(name, context),
            PromptType::Password(prompt) => prompt.run(name, context),
            PromptType::Select(prompt) => prompt.run(name, context),
            PromptType::MultiSelect(prompt) => prompt.run(name, context),
            PromptType::FuzzySelect(prompt) => prompt.run(name, context),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextPrompt {
    pub message: String,
}

impl Prompt for TextPrompt {
    fn run(&self, name: &str, context: &mut Context) -> Result<()> {
        let mut p = Readline::default()
            .title(&self.message)
            .prompt()
            .map_err(|e| Error::FailedToCreatePrompt(e.to_string()))?;

        let result = p.run().map_err(|e| Error::FailedToRunPrompt(e.to_string()))?;
        context.set_variable(name.to_string(), Value::String(result));

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmPrompt {
    pub message: String,
}

impl Prompt for ConfirmPrompt {
    fn run(&self, name: &str, context: &mut Context) -> Result<()> {
        let mut p = Confirm::new(&self.message)
            .prompt()
            .map_err(|e| Error::FailedToCreatePrompt(e.to_string()))?;

        let result = p.run().map_err(|e| Error::FailedToRunPrompt(e.to_string()))?;
        context.set_variable(name.to_string(), Value::Boolean(match result.as_str() {
            "yes" | "y" | "true" => true,
            "no" | "n" | "false" => false,
            _ => return Err(Error::FailedToRunPrompt(format!("Invalid confirmation response: {}", result))),
        }));

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PasswordPrompt {
    pub message: String,
}

impl Prompt for PasswordPrompt {
    fn run(&self, name: &str, context: &mut Context) -> Result<()> {
        let mut p = Password::default()
            .title(&self.message)
            .prompt()
            .map_err(|e| Error::FailedToCreatePrompt(e.to_string()))?;

        let result = p.run().map_err(|e| Error::FailedToRunPrompt(e.to_string()))?;
        context.set_variable(name.to_string(), Value::String(result));

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectPrompt {
    pub message: String,
    pub options: Vec<Expr>,
}

impl Prompt for SelectPrompt {
    fn run(&self, name: &str, context: &mut Context) -> Result<()> {
        let opts = self.options.iter().map(|expr| eval(expr, context)).collect::<Result<Vec<_>>>()?;
        let opts = get_options(opts, context);

        let mut p = Listbox::new(&opts)
            .title(&self.message)
            .prompt()
            .map_err(|e| Error::FailedToCreatePrompt(e.to_string()))?;

        let result = p.run().map_err(|e| Error::FailedToRunPrompt(e.to_string()))?;
        context.set_variable(name.to_string(), Value::String(result));

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MultiSelectPrompt {
    pub message: String,
    pub options: Vec<Expr>,
}

impl Prompt for MultiSelectPrompt {
    fn run(&self, name: &str, context: &mut Context) -> Result<()> {
        let opts = self.options.iter().map(|expr| eval(expr, context)).collect::<Result<Vec<_>>>()?;
        let opts = get_options(opts, context);

        let mut p = Checkbox::new(&opts)
            .title(&self.message)
            .prompt()
            .map_err(|e| Error::FailedToCreatePrompt(e.to_string()))?;
        let result = p.run().map_err(|e| Error::FailedToRunPrompt(e.to_string()))?;
        context.set_variable(name.to_string(), Value::Array(result.into_iter().map(|s| Value::String(s)).collect()));

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuzzySelectPrompt {
    pub message: String,
    pub options: Vec<Expr>,
}

impl Prompt for FuzzySelectPrompt {
    fn run(&self, name: &str, context: &mut Context) -> Result<()> {
        let options = self.options.iter().map(|expr| eval(expr, context)).collect::<Result<Vec<_>>>()?;
        let opts = get_options(options, context);

        let mut p = QuerySelector::new(&opts, |input, opts| {
            opts.into_iter()
                .filter(|opt| opt.to_lowercase().contains(&input.to_lowercase()))
                .map(|x| x.clone())
                .collect()
        })
        .title(&self.message)
        .prompt()
        .map_err(|e| Error::FailedToCreatePrompt(e.to_string()))?;

        let result = p.run().map_err(|e| Error::FailedToRunPrompt(e.to_string()))?;
        context.set_variable(name.to_string(), Value::String(result));

        Ok(())
    }
}
