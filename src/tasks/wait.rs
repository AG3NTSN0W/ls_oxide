use std::collections::HashMap;

use async_trait::async_trait;
use serde_yaml::Value;
use std::time::Instant;

use crate::executor::{ExecuteResult, WebDriverSession};

use super::{get_task_name, Task, TaskErr, TaskOk, TaskResult, TaskTypes};
use tokio::time::{sleep, Duration};

const TASK_TYPE: &str = "wait";

#[derive(PartialEq, Eq, Debug)]
pub struct Wait {
    _task_types: TaskTypes,
    name: String,
    duration_ms: Duration,
}

#[async_trait]
impl Task for Wait {
    fn new(task: &HashMap<String, Value>) -> TaskResult<Wait> {
        let name = get_task_name(task)?;
        let duration_ms = get_duration_ms(task)?;

        Ok(Wait {
            _task_types: TaskTypes::WAIT,
            name,
            duration_ms,
        })
    }

    async fn execute(&self, web_driver_session: WebDriverSession) -> ExecuteResult {
        let start = Instant::now();

        sleep(self.duration_ms).await;

        let name = self.name.clone();
        return Ok((
            web_driver_session,
            TaskOk {
                name,
                task_type: TaskTypes::WAIT,
                duration: start.elapsed().as_secs(),
            },
        ));
    }
}

fn get_duration_ms(task: &HashMap<String, Value>) -> TaskResult<Duration> {
    let duration_ms = match task.get(TASK_TYPE) {
        Some(duration_ms) => duration_ms,
        None => {
            return Err(TaskErr {
                message: "wait field not found".to_string(),
                task: Some(task.clone()),
                task_type: Some(TaskTypes::WAIT),
            });
        }
    };
    let duration_ms = match duration_ms.as_u64() {
        Some(duration_ms) => duration_ms,
        None => {
            // return Err(format!("send_key: input is not a string:\n{:#?}", task));
            return Err(TaskErr {
                message: "Wait field is not a number".to_string(),
                task: Some(task.clone()),
                task_type: Some(TaskTypes::WAIT),
            });
        }
    };

    Ok(Duration::from_millis(duration_ms))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_task() {
        let wait = HashMap::new();
        let result = Wait::new(&wait);
        let expected = Err(TaskErr {
            message: String::from("Malformed Task"),
            task: Some(wait),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_missing_task_data() {
        let yaml = "  
                name: 'wait 4 sec'
              ";

        let wait = serde_yaml::from_str(yaml).unwrap();
        let result = Wait::new(&wait);
        let expected = Err(TaskErr {
            message: String::from("wait field not found"),
            task: Some(wait),
            task_type: Some(TaskTypes::WAIT),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_empty_name() {
        let yaml = "  
                name: ''
                wait: 4
              ";
        let wait = serde_yaml::from_str(yaml).unwrap();

        let result = Wait::new(&wait);
        let expected = Err(TaskErr {
            message: String::from("Task name can`t be empty"),
            task: Some(wait),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_success() {
        let yaml = "  
                name: 'wait 4 sec'
                wait: 4
              ";

        let wait = serde_yaml::from_str(yaml).unwrap();
        let result = Wait::new(&wait);
        let expected = Ok(Wait {
            _task_types: TaskTypes::WAIT,
            name: "wait 4 sec".to_owned(),
            duration_ms: Duration::from_millis(4)
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_string() {
        let yaml = "  
                name: 'wait 4 sec'
                wait: 4
              ";

        let wait = serde_yaml::from_str(yaml).unwrap();
        let result = Wait::new(&wait);
        let expected = Ok(Wait {
            _task_types: TaskTypes::WAIT,
            name: "wait 4 sec".to_owned(),
            duration_ms: Duration::from_millis(4)
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_mapping() {
        let yaml = "  
                name: 'wait 4 sec'
                wait: 4
              ";

        let wait = serde_yaml::from_str(yaml).unwrap();
        let result = Wait::new(&wait);
        let expected = Ok(Wait {
            _task_types: TaskTypes::WAIT,
            name: "wait 4 sec".to_owned(),
            duration_ms: Duration::from_millis(4)
        });
        assert_eq!(expected, result)
    }
}
