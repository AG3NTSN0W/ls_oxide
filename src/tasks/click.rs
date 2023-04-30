use async_trait::async_trait;
use serde_yaml::Value;
use std::collections::HashMap;
use std::time::Instant;
use thirtyfour::By;

use super::{get_task, get_task_name, Task, TaskErr, TaskOk, TaskResult, TaskTypes};
use crate::{
    element::Element,
    executor::{ExecuteResult, WebDriverSession},
};

const TASK_TYPE: &str = "click";

#[derive(PartialEq, Eq, Debug)]
pub struct Click {
    _task_types: TaskTypes,
    name: String,
    element: Element,
}

#[async_trait]
impl Task for Click {
    fn new(task: &HashMap<String, Value>) -> TaskResult<Click> {
        let name = get_task_name(task)?;
        let click = get_task(task, TASK_TYPE)?;

        let element = match Element::new(click) {
            Ok(element) => element,
            Err(err) => {
                return Err(TaskErr {
                    message: err,
                    task: Some(task.clone()),
                    task_type: Some(TaskTypes::CLICK),
                })
            }
        };

        Ok(Click {
            name,
            _task_types: TaskTypes::CLICK,
            element,
        })
    }

    async fn execute(&self, web_driver_session: WebDriverSession) -> ExecuteResult {
        let start = Instant::now();
        // println!(
        //     "Taske Type: {:#?}\nName: {:#?}\nelement Type: {:#?},\nValue: {}",
        //     self._task_types, self.name, self.element.element_type, self.element.value
        // );

        let by: By = Element::find_by(&self.element);

        let element = match web_driver_session.driver.find(by).await {
            Ok(element) => element,
            Err(e) => {
                return Err((
                    web_driver_session,
                    TaskErr {
                        message: format!("{}", e),
                        task: None,
                        task_type: Some(TaskTypes::CLICK),
                    },
                ));
            }
        };

        let click = element.click().await;
        let name = self.name.clone();

        match click {
            Ok(_) => Ok((
                web_driver_session,
                TaskOk {
                    name,
                    task_type: TaskTypes::CLICK,
                    duration: start.elapsed().as_secs(),
                    result: None
                },
            )),
            Err(e) => {
                return Err((
                    web_driver_session,
                    TaskErr {
                        message: format!("{}", e),
                        task: None,
                        task_type: Some(TaskTypes::CLICK),
                    },
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::element::ElementType;

    use super::*;

    #[test]
    fn test_empty_task() {
        let click_map_empty = HashMap::new();
        let result = Click::new(&click_map_empty);
        let expected = Err(TaskErr {
            message: String::from("Malformed Task"),
            task: Some(click_map_empty),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_invalid_task_data() {
        let yaml = "  
        name: 'Click search button'
        click: '2'
              ";

        let click = serde_yaml::from_str(yaml).unwrap();
        let result = Click::new(&click);
        let expected = Err(TaskErr {
            message: String::from("Task data is Malformed"),
            task: Some(click),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_missing_task_data() {
        let yaml = "  
                name: 'Click search button'
              ";

        let click = serde_yaml::from_str(yaml).unwrap();
        let result = Click::new(&click);
        let expected = Err(TaskErr {
            message: String::from("Malformed Task"),
            task: Some(click),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_empty_name() {
        let yaml = "  
                name: ''
                click:
                    element:
                        xPath: '//*[@id=\"search-form\"]/fieldset/button'
              ";

        let click = serde_yaml::from_str(yaml).unwrap();

        let result = Click::new(&click);
        let expected = Err(TaskErr {
            message: String::from("Task name can`t be empty"),
            task: Some(click),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_invalid_name() {
        let yaml = "  
                name: 2
                click:
                    element:
                        xPath: '//*[@id=\"search-form\"]/fieldset/button'
              ";

        let click = serde_yaml::from_str(yaml).unwrap();

        let result = Click::new(&click);
        let expected = Err(TaskErr {
            message: String::from("Task name is not a string"),
            task: Some(click),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_invalid_element() {
        let yaml = "  
                name: 'Click search button'
                click:
                    element:
                        foo: '//*[@id=\"search-form\"]/fieldset/button'
              ";

        let click = serde_yaml::from_str(yaml).unwrap();

        let result = Click::new(&click);
        let expected = Err(TaskErr {
            message: String::from("Unknow Element Type: \"foo\""),
            task: Some(click),
            task_type: Some(TaskTypes::CLICK),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_invalid_element_value() {
        let yaml = "  
                name: 'Click search button'
                click: 
                    element:
                        xPath: 2
              ";

        let click = serde_yaml::from_str(yaml).unwrap();
        let result = Click::new(&click);
        let expected = Err(TaskErr {
            message: String::from("Element: Value is not a string"),
            task: Some(click),
            task_type: Some(TaskTypes::CLICK),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_missing_element() {
        let yaml = "  
                name: 'Click search button'
                click: 
                    foo: 'bar'
              ";

        let click = serde_yaml::from_str(yaml).unwrap();
        let result = Click::new(&click);
        let expected = Err(TaskErr {
            message: String::from("No element found"),
            task: Some(click),
            task_type: Some(TaskTypes::CLICK),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_success() {
        let yaml = "  
        name: 'Click search button'
        click:
          element:
            xPath: '//*[@id=\"search-form\"]/fieldset/button'
              ";

        let click = serde_yaml::from_str(yaml).unwrap();
        let result = Click::new(&click);
        let expected = Ok(Click {
            _task_types: TaskTypes::CLICK,
            name: "Click search button".to_owned(),
            element: Element {
                element_type: ElementType::XPATH,
                value: "//*[@id=\"search-form\"]/fieldset/button".to_owned(),
            },
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task() {
        let yaml = "  
        name: 'Click search button'
        click:
          element:
            id: 'search-form'
              ";

        let click = serde_yaml::from_str(yaml).unwrap();
        let result = Click::new(&click);
        let expected = Ok(Click {
            _task_types: TaskTypes::CLICK,
            name: "Click search button".to_owned(),
            element: Element {
                element_type: ElementType::ID,
                value: "search-form".to_owned(),
            },
        });
        assert_eq!(expected, result)
    }
}
