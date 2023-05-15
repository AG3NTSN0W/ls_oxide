use async_trait::async_trait;
use serde_yaml::Value;
use std::collections::HashMap;
use std::time::Instant;

use crate::{executor::{ExecuteResult, WebDriverSession}, variables::resolve_variables};

use super::{
    get_task, get_task_name, Task, TaskErr, TaskOk, TaskResult, TaskTypes,
};

const TASK_TYPE: &str = "link";

#[derive(PartialEq, Eq, Debug)]
pub struct Link {
    _task_types: TaskTypes,
    name: String,
    url: String,
}

#[async_trait]
impl Task for Link {
    fn new(task: &HashMap<String, Value>) -> TaskResult<Link> {
        let name = get_task_name(task)?;
        let url = get_url(task)?;
        Ok(Link {
            name,
            _task_types: TaskTypes::LINK,
            url,
        })
    }

    async fn execute(&self, web_driver_session: WebDriverSession) -> ExecuteResult {
        let start = Instant::now();
        // println!(
        //     "Taske Type: {:#?}\nName: {:#?}\nUrl: {:#?}",
        //     self._task_types, self.name, self.url
        // );

        let url = resolve_variables(&self.url, &web_driver_session.variables);

        let link = web_driver_session.driver.goto(url).await;
        let name = self.name.clone();

        match link {
            Ok(_) => Ok((
                web_driver_session,
                TaskOk {
                    name,
                    task_type: TaskTypes::LINK,
                    duration: start.elapsed().as_secs(),
                    result: None
                },
            )),
            Err(_) => {
                return Err((
                    web_driver_session,
                    TaskErr {
                        message: "Unable to open link".to_string(),
                        task: None,
                        task_type: Some(TaskTypes::LINK),
                    },
                ))
            }
        }
    }
}

fn get_url(task: &HashMap<String, Value>) -> TaskResult<String> {
    let link = get_task(task, TASK_TYPE)?;
    let url = match link.get("url") {
        Some(url) => url,
        None => {
            return Err(TaskErr {
                message: "url field not found".to_string(),
                task: Some(task.clone()),
                task_type: Some(TaskTypes::LINK),
            })
        }
    };
    let url = match url.as_str() {
        Some(url) => String::from(url),
        None => {
            return Err(TaskErr {
                message: "Url is not a string".to_string(),
                task: Some(task.clone()),
                task_type: Some(TaskTypes::LINK),
            })
        }
    };

    if url.is_empty() {
        return Err(TaskErr {
            message: "Url is empty".to_string(),
            task: Some(task.clone()),
            task_type: Some(TaskTypes::LINK),
        });
    }

    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_task() {
        let link = HashMap::new();
        let result = Link::new(&link);
        let expected = Err(TaskErr {
            message: String::from("Malformed Task"),
            task: Some(link),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_missing_task_data() {
        let yaml = "  
                name: 'Open wikipedia'
              ";

        let link = serde_yaml::from_str(yaml).unwrap();
        let result = Link::new(&link);
        let expected = Err(TaskErr {
            message: String::from("Malformed Task"),
            task: Some(link),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_missing_url() {
        let yaml = "  
                name: 'Open wikipedia'            
                link: 
                    foo: ''
              ";

        let link = serde_yaml::from_str(yaml).unwrap();
        let result = Link::new(&link);
        let expected = Err(TaskErr {
            message: String::from("url field not found"),
            task: Some(link),
            task_type: Some(TaskTypes::LINK),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_invaild_link_task() {
        let yaml = "  
                name: 'Open wikipedia'            
                link: 'foo'
              ";

        let link = serde_yaml::from_str(yaml).unwrap();
        let result = Link::new(&link);
        let expected = Err(TaskErr {
            message: String::from("Task data is Malformed"),
            task: Some(link),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_empty_url() {
        let yaml = "  
                name: 'Open wikipedia'            
                link:
                    url: '' 
              ";

        let link = serde_yaml::from_str(yaml).unwrap();
        let result = Link::new(&link);
        let expected = Err(TaskErr {
            message: String::from("Url is empty"),
            task: Some(link),
            task_type: Some(TaskTypes::LINK),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_invalid_url() {
        let yaml = "  
                name: 'Open wikipedia'            
                link:
                    url: 2 
              ";

        let link = serde_yaml::from_str(yaml).unwrap();
        let result = Link::new(&link);
        let expected = Err(TaskErr {
            message: String::from("Url is not a string"),
            task: Some(link),
            task_type: Some(TaskTypes::LINK),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_empty_name() {
        let yaml = "  
            name: ''
            link:
                url: 'https://wikipedia.org'  
              ";
        let link = serde_yaml::from_str(yaml).unwrap();

        let result = Link::new(&link);
        let expected = Err(TaskErr {
            message: String::from("Task name can`t be empty"),
            task: Some(link),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task() {
        let yaml = "  
            name: 'Open wikipedia'
            link:
                url: 'https://wikipedia.org'  
              ";

        let close = serde_yaml::from_str(yaml).unwrap();
        let result = Link::new(&close);
        let expected = Ok(Link {
            _task_types: TaskTypes::LINK,
            name: "Open wikipedia".to_owned(),
            url: "https://wikipedia.org".to_owned(),
        });
        assert_eq!(expected, result)
    }
}
