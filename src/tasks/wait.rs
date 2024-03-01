use std::collections::HashMap;

use crate::{
    element::Element, executor::ExecuteResult, structs::task_ok::TaskOk, web_driver_session::WebDriverSession
};
use async_trait::async_trait;
use serde_yaml::Value;
use thirtyfour::{extensions::query::{ElementQueryable, ElementWaitable}, By};
use std::time::Instant;

use super::{get_task, get_task_name, Task, TaskErr, TaskResult, TaskTypes};
use tokio::time::{sleep, Duration};

const TASK_TYPE: &str = "wait";

#[derive(PartialEq, Eq, Debug)]
pub struct Wait {
    _task_types: TaskTypes,
    name: String,
    duration_ms: Option<Duration>,
    element: Option<Element>,
}

#[async_trait]
impl Task for Wait {
    fn new(task: &HashMap<String, Value>) -> TaskResult<Wait> {
        let name = get_task_name(task)?;
        let duration_ms = get_duration_ms(task)?;

        let wait_task = get_task(task, TASK_TYPE)?;
        let element = match Element::new(wait_task) {
            Ok(element) => Some(element),
            Err(_) => None
        };

        Ok(Wait {
            _task_types: TaskTypes::WAIT,
            name,
            duration_ms,
            element
        })
    }

    async fn execute(&self, web_driver_session: &mut WebDriverSession) -> ExecuteResult {
        let start = Instant::now();

        if let Some(duration_ms) = self.duration_ms {
            sleep(duration_ms).await;
        }

        if let Some(wait_element) = &self.element {
            let by: By = Element::find_by_resolve(&wait_element, &web_driver_session.variables);

            let find = web_driver_session.driver.query(by).first().await;
            let element = match find {
                Ok(element) => element,
                Err(e) => {
                    return Err(
                        TaskErr::new(format!("{}", e), Some(TaskTypes::CLICK), None),
                    );
                }
            };
            
            match element.wait_until().displayed().await {
                Ok(element) => element,
                Err(_) => {
                    return Err(TaskErr::new(
                        format!(
                            "Unable to find element - Type: {:?}, Value: {}",
                            wait_element.element_type, wait_element.value
                        ),
                        Some(TaskTypes::SENDKEY),
                        None,
                    ))
                }
            };
        }
      

        let name = self.name.clone();
        return Ok(TaskOk {
            name,
            task_type: TaskTypes::WAIT,
            duration: start.elapsed().as_secs(),
            result: None,
        });
    }
}

fn get_duration_ms(task: &HashMap<String, Value>) -> TaskResult<Option<Duration>> {
    let wait_task = get_task(task, TASK_TYPE)?;

    let duration_ms = match wait_task.get("duration_ms") {
        Some(duration) => duration,
        None => return Ok(None),
    }; 


    let duration_ms = match duration_ms.as_u64() {
        Some(duration_ms) => duration_ms,
        None => {
            // return Err(format!("send_key: input is not a string:\n{:#?}", task));
            return Err(TaskErr::new(
                "Wait field is not a number".to_string(),
                Some(TaskTypes::WAIT),
                Some(task.clone()),
            ));
        }
    };
    Ok(Some(Duration::from_millis(duration_ms)))
}

#[cfg(test)]
mod tests {
    use crate::element::ElementType;

    use super::*;

    #[test]
    fn test_empty_task() {
        let wait = HashMap::new();
        let result = Wait::new(&wait);
        let expected = Err(TaskErr::new(
            String::from("Malformed Task"),
            None,
            Some(wait),
        ));
        assert_eq!(expected, result)
    }

    #[test]
    fn test_missing_task_data() {
        let yaml = "  
                name: 'wait 4 sec'
              ";

        let wait = serde_yaml::from_str(yaml).unwrap();
        let result = Wait::new(&wait);
        let expected = Err(TaskErr::new(
            String::from("Malformed Task"),
            None,
            Some(wait),
        ));
        assert_eq!(expected, result)
    }

    #[test]
    fn test_empty_name() {
        let yaml = "  
                name: ''
                wait: 
                    duration_ms: 4
              ";
        let wait = serde_yaml::from_str(yaml).unwrap();

        let result = Wait::new(&wait);
        let expected = Err(TaskErr::new(
            String::from("Task name can`t be empty"),
            None,
            Some(wait),
        ));
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_success() {
        let yaml = "  
                name: 'wait 4 sec'
                wait: 
                    duration_ms: 4
              ";

        let wait = serde_yaml::from_str(yaml).unwrap();
        let result = Wait::new(&wait);
        let expected = Ok(Wait {
            _task_types: TaskTypes::WAIT,
            name: "wait 4 sec".to_owned(),
            duration_ms: Some(Duration::from_millis(4)),
            element: None
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_success_element_x_path() {
        let yaml = "  
                name: 'wait 4 sec'
                wait: 
                  element:
                    xPath: 'search-form'
              ";

        let wait = serde_yaml::from_str(yaml).unwrap();
        let result = Wait::new(&wait);
        let expected = Ok(Wait {
            _task_types: TaskTypes::WAIT,
            name: "wait 4 sec".to_owned(),
            duration_ms: None,
            element: Some(Element {
                element_type: ElementType::XPATH,
                value: "search-form".to_owned(),
            })
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_success_element_id() {
        let yaml = "  
                name: 'wait 4 sec'
                wait: 
                  element:
                    id: 'search-form'
              ";

        let wait = serde_yaml::from_str(yaml).unwrap();
        let result = Wait::new(&wait);
        let expected = Ok(Wait {
            _task_types: TaskTypes::WAIT,
            name: "wait 4 sec".to_owned(),
            duration_ms: None,
            element: Some(Element {
                element_type: ElementType::ID,
                value: "search-form".to_owned(),
            })
        });
        assert_eq!(expected, result)
    }
}
