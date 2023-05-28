use std::{collections::HashMap, path::Path};

use async_trait::async_trait;
use serde_yaml::{Mapping, Value};
use std::time::Instant;
use thirtyfour::{prelude::WebDriverError, By};

use crate::{
    element::Element,
    executor::{ExecuteResult, WebDriverSession}, variables::resolve_variables,
};

use super::{get_task, get_task_name, Task, TaskErr, TaskOk, TaskResult, TaskTypes};

const TASK_TYPE: &str = "screenshot";

#[derive(PartialEq, Eq, Debug)]
pub struct Screenshot {
    _task_types: crate::tasks::TaskTypes,
    name: String,
    path: String,
    element: Option<Element>,
}

#[async_trait]
impl Task for Screenshot {
    fn new(task: &HashMap<String, Value>) -> TaskResult<Screenshot> {
        let name = get_task_name(task)?;
        let screenshot = get_task(task, TASK_TYPE)?;
        let path = match get_path(screenshot) {
            Ok(p) => p,
            Err(message) => {
                return Err(TaskErr {
                    message,
                    task: Some(task.clone()),
                    task_type: Some(TaskTypes::SCREENSHOT),
                })
            }
        };

        let element = match Element::new(screenshot) {
            Ok(element) => Some(element),
            Err(_) => None,
        };

        Ok(Screenshot {
            _task_types: TaskTypes::SCREENSHOT,
            name,
            path,
            element,
        })
    }

    async fn execute(&self, web_driver_session: WebDriverSession) -> ExecuteResult {
        let start = Instant::now();

        if let Some(element) = &self.element {
            let by: By = Element::find_by_resolve(element, &web_driver_session.variables);
            let element = match web_driver_session.driver.find(by).await {
                Ok(element) => element,
                Err(e) => {
                    return Err((
                        web_driver_session,
                        TaskErr {
                            message: format!("{}", e),
                            task: None,
                            task_type: Some(TaskTypes::SCREENSHOT),
                        },
                    ));
                }
            };

            let path = resolve_variables(&self.path, &web_driver_session.variables);

            let screenshot = element.screenshot(Path::new(&path)).await;
            return screenshot_result(screenshot, web_driver_session, &self.name, start);
        }

        let screenshot = web_driver_session
            .driver
            .screenshot(Path::new(&self.path))
            .await;

        screenshot_result(screenshot, web_driver_session, &self.name, start)
    }
}

fn screenshot_result(
    screenshot: Result<(), WebDriverError>,
    web_driver_session: WebDriverSession,
    name: &str,
    start: Instant,
) -> ExecuteResult {
    match screenshot {
        Ok(_) => Ok((
            web_driver_session,
            TaskOk {
                name: name.to_string(),
                task_type: TaskTypes::SCREENSHOT,
                duration: start.elapsed().as_secs(),
                result: None,
            },
        )),
        Err(e) => {
            Err((
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

fn get_path(screenshot: &Mapping) -> Result<String, String> {
    let screenshot_path = match screenshot.get("path") {
        Some(screenshot_path) => screenshot_path,
        None => {
            return Err("path field not found".to_string());
        }
    };

    match screenshot_path.as_str() {
        Some(screenshot_path) => Ok(String::from(screenshot_path)),
        None => Err("path field is not a string".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use crate::element::ElementType;

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
            message: String::from("Malformed Task"),
            task: Some(screenshot),
            task_type: None,
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
    fn test_task_success_no_element() {
        let yaml = "  
                name: 'Take a screenshot'
                screenshot: 
                    path: '/tmp/screenshot.png'
              ";

        let screenshot = serde_yaml::from_str(yaml).unwrap();
        let result = Screenshot::new(&screenshot);
        let expected = Ok(Screenshot {
            _task_types: TaskTypes::SCREENSHOT,
            name: "Take a screenshot".to_owned(),
            path: "/tmp/screenshot.png".to_owned(),
            element: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_success_element() {
        let yaml = "  
                name: 'Take a screenshot'
                screenshot: 
                    path: '/tmp/screenshot.png'
                    element:
                        xPath: 'search-form'
              ";

        let screenshot = serde_yaml::from_str(yaml).unwrap();
        let result = Screenshot::new(&screenshot);
        let expected = Ok(Screenshot {
            _task_types: TaskTypes::SCREENSHOT,
            name: "Take a screenshot".to_owned(),
            path: "/tmp/screenshot.png".to_owned(),
            element: Some(Element {
                element_type: ElementType::XPATH,
                value: "search-form".to_owned(),
            }),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_malformed_task() {
        let yaml = "  
                name: 'Take a screenshot'
                screenshot: true
              ";

        let screenshot = serde_yaml::from_str(yaml).unwrap();
        let result = Screenshot::new(&screenshot);
        let expected = Err(TaskErr {
            message: String::from("Task data is Malformed"),
            task: Some(screenshot),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_path_not_string() {
        let yaml = "  
                name: 'Take a screenshot'
                screenshot: 
                    path: 22
                    element:
                        xPath: 'search-form'
              ";

        let screenshot = serde_yaml::from_str(yaml).unwrap();
        let result = Screenshot::new(&screenshot);
        let expected = Err(TaskErr {
            message: String::from("path field is not a string"),
            task: Some(screenshot),
            task_type: Some(TaskTypes::SCREENSHOT),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_invalid_element() {
        let yaml = "  
                name: 'Take a screenshot'
                screenshot: 
                    path: '/tmp/screenshot.png'
                    element:
                        foo: 'search-form'
              ";

        let screenshot = serde_yaml::from_str(yaml).unwrap();
        let result = Screenshot::new(&screenshot);
        let expected = Ok(Screenshot {
            _task_types: TaskTypes::SCREENSHOT,
            name: "Take a screenshot".to_owned(),
            path: "/tmp/screenshot.png".to_owned(),
            element: None,
        });
        assert_eq!(expected, result)
    }
}
