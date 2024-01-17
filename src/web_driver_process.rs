use std::process::{Command, Stdio};
use std::time::Duration;
use std::{fmt, thread};

// const PATH_TO_DRIVER: &str = "/usr/bin/geckodriver";
const PATH_TO_DRIVER: &str = "/home/jacobusferreira/Downloads/geckodriver";

#[derive(PartialEq, Debug, Clone)]
pub enum WebDriverProcessError {
    UnableToCreateProcess(String),
    ProcessNotRunning,
    ProcessNotFound,
}

impl fmt::Display for WebDriverProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WebDriverProcessError::UnableToCreateProcess(error) => {
                write!(f, "Failed to start process. --> {}", error)
            }
            WebDriverProcessError::ProcessNotFound => write!(f, "Web Driver Process wasn't found"),
            WebDriverProcessError::ProcessNotRunning => write!(f, "Process wasn't running"),
        }
    }
}

pub struct WebDriverProcess {
    process: Option<std::process::Child>,
}

impl WebDriverProcess {
    pub fn new() -> Result<Self, WebDriverProcessError> {
        let process_thread = Command::new(PATH_TO_DRIVER)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        let process = match process_thread {
            Ok(p) => p,
            Err(e) => {
                return Err(WebDriverProcessError::UnableToCreateProcess(e.to_string()));
            }
        };

        thread::sleep(Duration::from_millis(2000));

        Ok(WebDriverProcess {
            process: Some(process),
        })
    }
}

impl Drop for WebDriverProcess {
    fn drop(&mut self) {
        match self.process.take() {
            Some(mut process) => {
                process
                    .kill()
                    .unwrap_or_else(|_| panic!("{:?}", WebDriverProcessError::ProcessNotRunning));
            }
            None => println!("{}", WebDriverProcessError::ProcessNotFound),
        }
    }
}