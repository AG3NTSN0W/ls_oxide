use async_trait::async_trait;
use serde_yaml::Value;
use std::collections::HashMap;
use std::time::Instant;

use crate::{element::Element, executor::{ExecuteResult, WebDriverSession}, variables::resolve_variables};

use super::{
    get_task, get_task_name, Task, TaskErr, TaskOk, TaskResult, TaskTypes,
};

const TASK_TYPE: &str = "send_key";

#[derive(PartialEq, Eq, Debug)]
pub struct SendKey {
    _task_types: TaskTypes,
    name: String,
    element: Element,
    input: String,
}

#[async_trait]
impl Task for SendKey {
    fn new(task: &HashMap<String, Value>) -> TaskResult<SendKey> {
        let name = get_task_name(task)?;
        let send_key = get_task(task, TASK_TYPE)?;
        let input = get_input(task)?;

        let element = match Element::new(send_key) {
            Ok(element) => element,
            Err(e) => {
                return Err(TaskErr {
                    message: e,
                    task: Some(task.clone()),
                    task_type: Some(TaskTypes::SENDKEY),
                });
            }
        };

        Ok(SendKey {
            name,
            _task_types: TaskTypes::SENDKEY,
            element,
            input,
        })
    }

    async fn execute(&self, web_driver_session: WebDriverSession) -> ExecuteResult {
        let start = Instant::now();
        // println!(
        //     "Taske Type: {:#?}\nName: {:#?}\nelement Type: {:#?},\nValue: {}",
        //     self._task_types, self.name, self.element.element_type, self.element.value
        // );
        let by = Element::find_by_resolve(&self.element, &web_driver_session.variables);

        let element = match web_driver_session.driver.find(by).await {
            Ok(element) => element,
            Err(_) => {
                return Err((
                    web_driver_session,
                    TaskErr {
                        message: format!(
                            "Unable to find element - Type: {:?}, Value: {}",
                            self.element.element_type, self.element.value
                        ),
                        task: None,
                        task_type: Some(TaskTypes::SENDKEY),
                    },
                ))
            }
        };

        let input = resolve_variables(&self.input, &web_driver_session.variables);
        let send_key = element.send_keys(input).await;
        let name = self.name.clone();

        match send_key {
            Ok(_) => Ok((
                web_driver_session,
                TaskOk {
                    name,
                    task_type: TaskTypes::SENDKEY,
                    duration: start.elapsed().as_secs(),
                    result: None
                },
            )),
            Err(_) => {
                return Err((
                    web_driver_session,
                    TaskErr {
                        message: format!(
                            "Unable to send keys - Type: {:?}, Value: {}",
                            self.element.element_type, self.element.value
                        ),
                        task: None,
                        task_type: Some(TaskTypes::SENDKEY),
                    },
                ))
            }
        }
    }
}

fn get_input(task: &HashMap<String, Value>) -> TaskResult<String> {
    let link = get_task(task, TASK_TYPE)?;
    let input = match link.get("input") {
        Some(input) => input,
        None => {
            // return Err(format!("send_key: Task is malformed:"));
            return Err(TaskErr {
                message: "input field not found".to_string(),
                task: Some(task.clone()),
                task_type: Some(TaskTypes::SENDKEY),
            });
        }
    };
    let input = match input.as_str() {
        Some(url) => String::from(url),
        None => {
            // return Err(format!("send_key: input is not a string:\n{:#?}", task));
            return Err(TaskErr {
                message: "input is not a string".to_string(),
                task: Some(task.clone()),
                task_type: Some(TaskTypes::SENDKEY),
            });
        }
    };

    if input.is_empty() {
        // return Err(format!("send_key: input is empty:\n{:#?}", task));
        return Err(TaskErr {
            message: "input is empty".to_string(),
            task: Some(task.clone()),
            task_type: Some(TaskTypes::SENDKEY),
        });
    }

    Ok(input)
}

#[cfg(test)]
mod tests {
    use crate::{element::ElementType, tasks::*};

    use super::*;

    #[test]
    fn test_empty_task() {
        let send_key_task = HashMap::new();
        let result = SendKey::new(&send_key_task);
        let expected = Err(TaskErr {
            message: String::from("Malformed Task"),
            task: Some(send_key_task),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_invalid_task_data() {
        let yaml = "  
        name: 'enter rust in search'
        send_key: 2
              ";

        let send_key_task = serde_yaml::from_str(yaml).unwrap();
        let result = SendKey::new(&send_key_task);
        let expected = Err(TaskErr {
            message: String::from("Task data is Malformed"),
            task: Some(send_key_task),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_missing_task_data() {
        let yaml = "  
        name: 'enter rust in search'
              ";

        let send_key_task = serde_yaml::from_str(yaml).unwrap();
        let result = SendKey::new(&send_key_task);
        let expected = Err(TaskErr {
            message: String::from("Malformed Task"),
            task: Some(send_key_task),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_empty_name() {
        let yaml = "  
        name: ''
        send_key:
            input: 'Rust'
            element:
                xPath: '//*[@id=\"searchInput\"]'
              ";

        let send_key_task = serde_yaml::from_str(yaml).unwrap();

        let result = SendKey::new(&send_key_task);
        let expected = Err(TaskErr {
            message: String::from("Task name can`t be empty"),
            task: Some(send_key_task),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_invalid_name() {
        let yaml = "  
        name: 2
        send_key:
            input: 'Rust'
            element:
                xPath: '//*[@id=\"searchInput\"]'
              ";

        let send_key_task = serde_yaml::from_str(yaml).unwrap();

        let result = SendKey::new(&send_key_task);
        let expected = Err(TaskErr {
            message: String::from("Task name is not a string"),
            task: Some(send_key_task),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_invalid_element() {
        let yaml = "  
        name: 'enter rust in search'
        send_key:
            input: 'Rust'
            element:
                foo: '//*[@id=\"searchInput\"]'
              ";

        let send_key_task = serde_yaml::from_str(yaml).unwrap();

        let result = SendKey::new(&send_key_task);
        let expected = Err(TaskErr {
            message: String::from("Unknow Element Type: \"foo\""),
            task: Some(send_key_task),
            task_type: Some(TaskTypes::SENDKEY),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_invalid_element_value() {
        let yaml = "  
        name: 'enter rust in search'
        send_key:
            input: 'Rust'
            element:
                xPath: 2
              ";

        let send_key_task = serde_yaml::from_str(yaml).unwrap();
        let result = SendKey::new(&send_key_task);
        let expected = Err(TaskErr {
            message: String::from("Element: Value is not a string"),
            task: Some(send_key_task),
            task_type: Some(TaskTypes::SENDKEY),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_invalid_input() {
        let yaml = "  
        name: 'enter rust in search'
        send_key:
            input: 2
            element:
                xPath: '//*[@id=\"searchInput\"]'
              ";

        let send_key_task = serde_yaml::from_str(yaml).unwrap();
        let result = SendKey::new(&send_key_task);
        let expected = Err(TaskErr {
            message: String::from("input is not a string"),
            task: Some(send_key_task),
            task_type: Some(TaskTypes::SENDKEY),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_missing_input() {
        let yaml = "  
        name: 'enter rust in search'
        send_key:
            element:
                xPath: '//*[@id=\"searchInput\"]'
              ";

        let send_key_task = serde_yaml::from_str(yaml).unwrap();
        let result = SendKey::new(&send_key_task);
        let expected = Err(TaskErr {
            message: String::from("input field not found"),
            task: Some(send_key_task),
            task_type: Some(TaskTypes::SENDKEY),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_missing_element() {
        let yaml = "  
        name: 'enter rust in search'
        send_key:
            input: 'Rust'
              ";

        let send_key_task = serde_yaml::from_str(yaml).unwrap();
        let result = SendKey::new(&send_key_task);
        let expected = Err(TaskErr {
            message: String::from("No element found"),
            task: Some(send_key_task),
            task_type: Some(TaskTypes::SENDKEY),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_success() {
        let yaml = "  
        name: 'enter rust in search'
        send_key:
            input: 'Rust'
            element:
                xPath: '//*[@id=\"searchInput\"]'
              ";

        let send_key_task = serde_yaml::from_str(yaml).unwrap();
        let result = SendKey::new(&send_key_task);
        let expected = Ok(SendKey {
            _task_types: TaskTypes::SENDKEY,
            name: "enter rust in search".to_owned(),
            input: "Rust".to_owned(),
            element: Element {
                element_type: ElementType::XPATH,
                value: "//*[@id=\"searchInput\"]".to_owned(),
            },
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task() {
        let yaml = "  
        name: 'enter rust in search'
        send_key:
            input: 'Rust'
            element:
                id: 'searchInput'
              ";

        let send_key_task = serde_yaml::from_str(yaml).unwrap();
        let result = SendKey::new(&send_key_task);
        let expected = Ok(SendKey {
            _task_types: TaskTypes::SENDKEY,
            name: "enter rust in search".to_owned(),
            input: "Rust".to_owned(),
            element: Element {
                element_type: ElementType::ID,
                value: "searchInput".to_owned(),
            },
        });
        assert_eq!(expected, result)
    }
}
