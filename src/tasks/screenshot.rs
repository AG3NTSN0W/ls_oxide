use std::{collections::HashMap, path::Path};

use async_trait::async_trait;
use serde_yaml::Value;
use std::time::Instant;

use crate::executor::{ExecuteResult, WebDriverSession};

use super::{get_task_name, Task, TaskErr, TaskOk, TaskResult, TaskTypes};

const TASK_TYPE: &str = "screenshot";

#[derive(PartialEq, Eq, Debug)]
pub struct Screenshot {
    _task_types: crate::tasks::TaskTypes,
    name: String,
    path: String,
}

#[async_trait]
impl Task for Screenshot {
    fn new(task: &HashMap<String, Value>) -> TaskResult<Screenshot> {
        let name = get_task_name(task)?;
        let path = get_path(task)?;

        Ok(Screenshot {
            _task_types: TaskTypes::SCREENSHOT,
            name,
            path,
        })
    }

    async fn execute(&self, web_driver_session: WebDriverSession) -> ExecuteResult {
        let start = Instant::now();

        let screenshot = web_driver_session
            .driver
            .screenshot(Path::new(&self.path))
            .await;
        let name = self.name.clone();

        match screenshot {
            Ok(_) => Ok((
                web_driver_session,
                TaskOk {
                    name,
                    task_type: TaskTypes::SCREENSHOT,
                    duration: start.elapsed().as_secs(),
                    result: None
                },
            )),
            Err(e) => {
                return Err((
                    web_driver_session,
                    TaskErr {
                        message: format!("Unable to take a screenshot: {:?}", e),
                        task: None,
                        task_type: Some(TaskTypes::SCREENSHOT),
                    },
                ))
            }
        }
    }
}

fn get_path(task: &HashMap<String, Value>) -> TaskResult<String> {
    let screenshot_path = match task.get(TASK_TYPE) {
        Some(screenshot_path) => screenshot_path,
        None => {
            return Err(TaskErr {
                message: "screenshot field not found".to_string(),
                task: Some(task.clone()),
                task_type: Some(TaskTypes::SCREENSHOT),
            });
        }
    };
    let screenshot_path = match screenshot_path.as_str() {
        Some(screenshot_path) => screenshot_path,
        None => {
            // return Err(format!("send_key: input is not a string:\n{:#?}", task));
            return Err(TaskErr {
                message: "screenshot field is not a string".to_string(),
                task: Some(task.clone()),
                task_type: Some(TaskTypes::SCREENSHOT),
            });
        }
    };

    Ok(String::from(screenshot_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_task() {
        let screenshot = HashMap::new();
        let result = Screenshot::new(&screenshot);
        let expected = Err(TaskErr {
            message: String::from("Malformed Task"),
            task: Some(screenshot),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_missing_task_data() {
        let yaml = "  
                name: 'Take a screenshot'
              ";

        let screenshot = serde_yaml::from_str(yaml).unwrap();
        let result = Screenshot::new(&screenshot);
        let expected = Err(TaskErr {
            message: String::from("screenshot field not found"),
            task: Some(screenshot),
            task_type: Some(TaskTypes::SCREENSHOT),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_empty_name() {
        let yaml = "  
                name: ''
                screenshot: True
              ";
        let screenshot = serde_yaml::from_str(yaml).unwrap();

        let result = Screenshot::new(&screenshot);
        let expected = Err(TaskErr {
            message: String::from("Task name can`t be empty"),
            task: Some(screenshot),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_success() {
        let yaml = "  
                name: 'Take a screenshot'
                screenshot: '/tmp/screenshot.png'
              ";

        let screenshot = serde_yaml::from_str(yaml).unwrap();
        let result = Screenshot::new(&screenshot);
        let expected = Ok(Screenshot {
            _task_types: TaskTypes::SCREENSHOT,
            name: "Take a screenshot".to_owned(),
            path: "/tmp/screenshot.png".to_owned()
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_not_string() {
        let yaml = "  
                name: 'Take a screenshot'
                screenshot: true
              ";

        let screenshot = serde_yaml::from_str(yaml).unwrap();
        let result = Screenshot::new(&screenshot);
        let expected = Err(TaskErr {
            message: String::from("screenshot field is not a string"),
            task: Some(screenshot),
            task_type: Some(TaskTypes::SCREENSHOT),
        });
        assert_eq!(expected, result)
    }
}
