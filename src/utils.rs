use std::fmt::Debug;
use std::{error::Error, fmt::Display};
use log::*;

#[derive(Debug)]
struct CustomError {
    message: String,
    cause: Option<Box<dyn Error>>,
    fatal: bool,
}

impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.fatal {
            write!(f, "Fatal error: {}", self.message)
        } else {
            write!(f, "Error: {}", self.message)
        }
    }
}

impl Error for CustomError {
    fn description(&self) -> &str {
        &self.message
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

#[cfg(feature = "backtrace")]
macro_rules! print_trace {
    () => {
        use backtrace::Backtrace;
        let backtrace = Backtrace::new();
        
        trace!("Backtrace:");
        // Get the Debug representation of the backtrace
        let debug = format!("{:?}", backtrace);
        // Print each line individually
        for line in debug.lines() {
            trace!("{}", line);
        }
    }
}

#[cfg(not(feature = "backtrace"))]
macro_rules! print_trace {
    () => {
        trace!("Backtrace disabled, enable with the 'backtrace' feature");
    };
}


pub fn unwrap_or_err<T>(result: Option<T>, message: &str, fatal: bool) -> Result<T, Box<dyn Error>> {
        let e = CustomError {
            message: message.to_string(),
            cause: None,
            fatal,
        };
        match result {
            Some(result) => Ok(result),
            None => {
                error!("{}", e.message);
                print_trace!();
                if e.fatal {
                    #[cfg(not(test))]
                    std::process::exit(1);
                    #[cfg(test)]
                    // Panicking can be caught by tests, exiting cannot
                    panic!("{}", e.message);
                } else {
                    Err(Box::new(e))
                }
            }
        }
}

fn print_cause(cause: &dyn Error) {
    trace!("Caused by: {}", cause);
    if let Some(cause) = cause.source() {
        print_cause(cause);
    }
}

pub fn unwrap_result_or_err<T: Debug, E>(result: Result<T, E>, message: &str, fatal: bool) -> Result<T, Box<dyn Error>>
    where E: Error + 'static {
        match result {
            Ok(result) => Ok(result),
            Err(err) => {
                let e = CustomError {
                    message: message.to_string(),
                    cause: Some(Box::new(err)),
                    fatal,
                };
                error!("{}", e.message);
                print_cause(e.cause.as_ref().unwrap().as_ref());
                print_trace!();
                if e.fatal {
                    #[cfg(not(test))]
                    std::process::exit(1);
                    #[cfg(test)]
                    // Panicking can be caught by tests, exiting cannot
                    panic!("{}", e.message);
                } else {
                    Err(Box::new(e))
                }
            }
        }
    }