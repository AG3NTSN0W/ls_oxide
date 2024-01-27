use async_trait::async_trait;
use serde_yaml::Value;
use std::collections::HashMap;
use std::time::Instant;

use crate::{
    executor::ExecuteResult, structs::task_ok::TaskOk, variables::resolve_variables,
    web_driver_session::WebDriverSession,
};

use super::{get_task, get_task_name, Task, TaskErr, TaskResult, TaskTypes};

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

    async fn execute(&self, web_driver_session: &mut WebDriverSession) -> ExecuteResult {
        let start = Instant::now();
        // println!(
        //     "Taske Type: {:#?}\nName: {:#?}\nUrl: {:#?}",
        //     self._task_types, self.name, self.url
        // );

        let url = resolve_variables(&self.url, &web_driver_session.variables);

        let link = web_driver_session.driver.goto(url).await;
        let name = self.name.clone();

        match link {
            Ok(_) => Ok(TaskOk {
                name,
                task_type: TaskTypes::LINK,
                duration: start.elapsed().as_secs(),
                result: None,
            }),
            Err(_) => {
                return Err(TaskErr::new(
                    "Unable to open link".to_string(),
                    Some(TaskTypes::LINK),
                    None,
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
            return Err(TaskErr::new(
                "url field not found".to_string(),
                Some(TaskTypes::LINK),
                Some(task.clone()),
            ))
        }
    };
    let url = match url.as_str() {
        Some(url) => String::from(url),
        None => {
            return Err(TaskErr::new(
                "Url is not a string".to_string(),
                Some(TaskTypes::LINK),
                Some(task.clone()),
            ))
        }
    };

    if url.is_empty() {
        return Err(TaskErr::new(
            "Url is empty".to_string(),
            Some(TaskTypes::LINK),
            Some(task.clone()),
        ));
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
        let expected = Err(TaskErr::new(
            String::from("Malformed Task"),
            None,
            Some(link),
        ));
        assert_eq!(expected, result)
    }

    #[test]
    fn test_missing_task_data() {
        let yaml = "  
                name: 'Open wikipedia'
              ";

        let link = serde_yaml::from_str(yaml).unwrap();
        let result = Link::new(&link);
        let expected = Err(TaskErr::new(
            String::from("Malformed Task"),
            None,
            Some(link),
        ));
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
        let expected = Err(TaskErr::new(
            String::from("url field not found"),
            Some(TaskTypes::LINK),
            Some(link),
        ));
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
        let expected = Err(TaskErr::new(
            String::from("Task data is Malformed"),
            None,
            Some(link),
        ));
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
        let expected = Err(TaskErr::new(
            String::from("Url is empty"),
            Some(TaskTypes::LINK),
            Some(link),
        ));
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
        let expected = Err(TaskErr::new(
            String::from("Url is not a string"),
            Some(TaskTypes::LINK),
            Some(link),
        ));
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
        let expected = Err(TaskErr::new(
            String::from("Task name can`t be empty"),
            None,
            Some(link),
        ));
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
