use regex::Error as RegexError;
use std::{borrow::Borrow, error::Error, fmt::Display};

#[derive(Debug, PartialEq, Eq)]
pub enum OrmerError {
    Regex,
    UserConfigError,
    ParsingError,
}
impl Display for OrmerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Regex")
    }
}

#[derive(Debug)]
pub struct StackError {
    pub error_type: OrmerError,
    pub file_name: &'static str,
    pub ln: u32,
    pub msg: Option<String>,
    pub lower_stack: Option<Box<dyn Error>>,
}

impl StackError {
    pub fn from_regex_error<S: Borrow<str> + Display>(
        error: RegexError,
        file_name: &'static str,
        ln: u32,
        msg: Option<S>,
    ) -> Self {
        Self::new(OrmerError::Regex, file_name, ln, msg, Some(error))
    }
    pub fn user_config_error<S: Borrow<str> + Display>(
        file_name: &'static str,
        ln: u32,
        msg: Option<S>,
    ) -> Self {
        Self::new_wo_error(OrmerError::UserConfigError, file_name, ln, msg)
    }
    pub fn new_wo_error<S: Borrow<str> + Display>(
        error_type: OrmerError,
        file_name: &'static str,
        ln: u32,
        msg: Option<S>,
    ) -> Self {
        Self {
            error_type,
            file_name,
            ln,
            msg: match msg {
                Some(msg) => Some(msg.to_string()),
                None => None,
            },
            lower_stack: None,
        }
    }
    pub fn new<T: Error + 'static, S: Borrow<str> + Display>(
        error_type: OrmerError,
        file_name: &'static str,
        ln: u32,
        msg: Option<S>,
        error: Option<T>,
    ) -> Self {
        Self {
            error_type,
            file_name,
            ln,
            msg: match msg {
                Some(msg) => Some(msg.to_string()),
                None => None,
            },
            lower_stack: match error {
                Some(error) => Some(Box::new(error)),
                None => None,
            },
        }
    }
}

impl Display for StackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = if let Some(msg) = &self.msg {
            format!(", msg: {}", msg)
        } else {
            "".to_string()
        };
        let lower_stack = if let Some(stack) = &self.lower_stack {
            format!("\n{}", stack)
        } else {
            "".to_string()
        };
        write!(
            f,
            "{}: ln: {}, row: {}{}{}",
            self.error_type, self.file_name, self.ln, msg, lower_stack
        )
    }
}

impl Error for StackError {}
