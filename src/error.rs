use std::{fmt};
use std::error::Error;
use std::fmt::Display;

use serde::{Deserialize, Serialize};
use log::SetLoggerError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LogError {
    E(String)
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

impl From<SetLoggerError> for LogError{
    fn from(arg: SetLoggerError) -> Self {
        LogError::E(arg.to_string())
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
            LogError::E(data) => {
                data.as_str()
            }
        };
    }
}


pub trait AsStdResult<T> where T: Clone {
    fn as_std_result(&self) -> Result<T, Box<dyn std::error::Error>>;
}

impl<T> AsStdResult<T> for Result<T, LogError> where T: Clone {
    fn as_std_result(&self) -> Result<T, Box<dyn std::error::Error>> {
        return match self {
            Err(e) => {
                Err(Box::new(e.clone()))
            }
            Ok(o) => {
                Ok(o.clone())
            }
        };
    }
}


#[test]
pub fn test_error() {
    let e = e().err().unwrap();
    println!("{}", e);
}

fn e() -> Result<String, LogError> {
    return Err(LogError::from("e"));
}