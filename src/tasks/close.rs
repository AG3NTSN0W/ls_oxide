use async_trait::async_trait;
use serde_yaml::Value;
use std::collections::HashMap;
use std::time::Instant;

use crate::{executor::ExecuteResult, web_driver_session::WebDriverSession, structs::task_ok::TaskOk};

use super::{get_task_name, Task, TaskErr, TaskResult, TaskTypes};
const TASK_TYPE: &str = "close";

#[derive(PartialEq, Eq, Debug)]
pub struct Close {
    _task_types: TaskTypes,
    name: String,
}

#[async_trait]
impl Task for Close {
    fn new(task: &HashMap<String, Value>) -> TaskResult<Close> {
        let name = get_task_name(task)?;
        if !task.contains_key(TASK_TYPE) {
            return Err(TaskErr::new(
                String::from("Malformed Task"),
                Some(TaskTypes::CLOSE),
                Some(task.clone()),
            ));
        }
        Ok(Close {
            name,
            _task_types: TaskTypes::CLOSE,
        })
    }

    async fn execute(&self, web_driver_session: WebDriverSession) -> ExecuteResult {
        let start = Instant::now();
        // println!(
        //     "Taske Type: {:#?}\nName: {:#?}",
        //     self._task_types, self.name
        // );

        let close = web_driver_session.clone().driver.quit().await;
        let name = self.name.clone();

        match close {
            Ok(_) => {
                return Ok((
                    web_driver_session,
                    TaskOk {
                        name,
                        task_type: TaskTypes::CLOSE,
                        duration: start.elapsed().as_secs(),
                        result: None,
                    },
                ));
            }

            Err(e) => Err((
                web_driver_session,
                TaskErr::new(
                    format!("Unable to close webdriver: {}", e),
                    Some(TaskTypes::CLOSE),
                    None,
                ),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_task() {
        let close = HashMap::new();
        let result = Close::new(&close);
        let expected = Err(TaskErr::new(
            String::from("Malformed Task"),
            None,
            Some(close),
        ));
        assert_eq!(expected, result)
    }

    #[test]
    fn test_missing_task_data() {
        let yaml = "  
                name: 'closing web driver session'
              ";

        let close = serde_yaml::from_str(yaml).unwrap();
        let result = Close::new(&close);
        let expected = Err(TaskErr::new(
            String::from("Malformed Task"),
            Some(TaskTypes::CLOSE),
            Some(close),
        ));
        assert_eq!(expected, result)
    }

    #[test]
    fn test_empty_name() {
        let yaml = "  
                name: ''
                close: True
              ";
        let close = serde_yaml::from_str(yaml).unwrap();

        let result = Close::new(&close);
        let expected = Err(TaskErr::new(
            String::from("Task name can`t be empty"),
            None,
            Some(close),
        ));
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_success() {
        let yaml = "  
                name: 'closing web driver session'
                close: True
              ";

        let close = serde_yaml::from_str(yaml).unwrap();
        let result = Close::new(&close);
        let expected = Ok(Close {
            _task_types: TaskTypes::CLOSE,
            name: "closing web driver session".to_owned(),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_string() {
        let yaml = "  
                name: 'closing web driver session'
                close: 'does not matter what it is'
              ";

        let close = serde_yaml::from_str(yaml).unwrap();
        let result = Close::new(&close);
        let expected = Ok(Close {
            _task_types: TaskTypes::CLOSE,
            name: "closing web driver session".to_owned(),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_mapping() {
        let yaml = "  
                name: 'closing web driver session'
                close: {}
              ";

        let close = serde_yaml::from_str(yaml).unwrap();
        let result = Close::new(&close);
        let expected = Ok(Close {
            _task_types: TaskTypes::CLOSE,
            name: "closing web driver session".to_owned(),
        });
        assert_eq!(expected, result)
    }
}
