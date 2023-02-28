use std::error::Error;
use std::fmt;
use std::fmt::Display;

use log::SetLoggerError;

#[derive(Clone, Debug)]
pub enum LogError {
    E(String),
}

impl From<&str> for LogError {
    fn from(arg: &str) -> Self {
        return LogError::E(arg.to_string());
    }
}

impl From<std::string::String> for LogError {
    fn from(arg: String) -> Self {
        return LogError::E(arg);
    }
}

impl From<SetLoggerError> for LogError {
    fn from(arg: SetLoggerError) -> Self {
        LogError::E(arg.to_string())
    }
}

impl From<std::io::Error> for LogError {
    fn from(value: std::io::Error) -> Self {
        LogError::E(value.to_string())
    }
}

impl Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return match self {
            LogError::E(ref err) => {
                write!(f, "Rbatis Error: {}", err)
            }
        };
    }
}

impl Error for LogError {
    fn description(&self) -> &str {
        return match self {
            LogError::E(data) => data.as_str(),
        };
    }
}

impl Default for LogError {
    fn default() -> Self {
        LogError::E(String::new())
    }
}
