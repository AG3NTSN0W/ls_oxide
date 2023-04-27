
mod click;
mod close;
mod link;
mod send_key;
mod wait;
mod screenshot;
mod validate;

use crate::executor::{ExecuteResult, WebDriverSession};
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::{fs, path::PathBuf};

use self::click::Click;
use self::close::Close;
use self::link::Link;
use self::screenshot::Screenshot;
use self::send_key::SendKey;
use self::wait::Wait;
use async_trait::async_trait;

pub type Tasks = Vec<Box<dyn Task>>;
pub type TaskResult<T> = std::result::Result<T, TaskErr>;

const NAME: &'static str = "name";

#[async_trait]
pub trait Task {
    async fn execute(&self, web_driver_session: WebDriverSession) -> ExecuteResult;
    fn new(task: &HashMap<String, Value>) -> TaskResult<Self>
    where
        Self: Sized;
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TaskData {
    pub meta_data: HashMap<String, Value>,
    pub tasks: Vec<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
pub enum TaskTypes {
    CLICK,
    SENDKEY,
    LINK,
    CLOSE,
    WAIT,
    SCREENSHOT,
    VALIDATE,
    #[default]
    NONE,
}

impl FromStr for TaskTypes {
    type Err = TaskErr;

    fn from_str(input: &str) -> Result<TaskTypes, Self::Err> {
        match input {
            "click" => Ok(TaskTypes::CLICK),
            "send_key" => Ok(TaskTypes::SENDKEY),
            "link" => Ok(TaskTypes::LINK),
            "close" => Ok(TaskTypes::CLOSE),
            "wait" => Ok(TaskTypes::WAIT),
            "screenshot" => Ok(TaskTypes::SCREENSHOT),
            _ => Err(TaskErr {
                message: format!("Unknow Task Type: {:#?}", input),
                task: None,
                task_type: None,
            }),
        }
    }
}

pub fn to_task(path: PathBuf) -> TaskResult<Tasks> {
    let mut tasks: Vec<Box<dyn Task>> = vec![];
    let task_data = get_task_data(path)?;
    for task_data in task_data.tasks.iter() {
        tasks.push(data_to_task(&task_data)?);
    }
    validate_first_task(&task_data)?;
    if !is_last_task_close(&task_data)? {
        let mut close: HashMap<String, Value> = HashMap::new();
        close.insert(
            String::from(NAME),
            Value::String(String::from("closing web driver session")),
        );
        close.insert(String::from("close"), Value::Bool(true));
        tasks.push(data_to_task(&close)?);
    }
    Ok(tasks)
}

fn data_to_task(task_data: &HashMap<String, Value> ) -> TaskResult<Box<dyn Task>> {
    let task_type = get_task_type(&task_data)?;
    let task: Box<dyn Task> = match task_type {
        TaskTypes::SENDKEY => Box::new(<SendKey as Task>::new(task_data)?),
        TaskTypes::CLICK => Box::new(<Click as Task>::new(task_data)?),
        TaskTypes::CLOSE => Box::new(<Close as Task>::new(task_data)?),
        TaskTypes::LINK => Box::new(<Link as Task>::new(task_data)?),
        TaskTypes::WAIT => Box::new(<Wait as Task>::new(task_data)?),
        TaskTypes::SCREENSHOT => Box::new(<Screenshot as Task>::new(task_data)?),
        _ => {
            return Err(TaskErr {
                message: format!("Invalid Task Type"),
                task: Some(task_data.clone()),
                task_type: Some(TaskTypes::NONE),
            })
        }
    };
    Ok(task)
}

fn get_task_data(path: PathBuf) -> TaskResult<TaskData> {
    let yaml = match fs::read_to_string(path) {
        Ok(data) => data,
        Err(_) => {
            return Err(TaskErr {
                message: String::from("Unable to read File"),
                task: None,
                task_type: None,
            })
        }
    };
    match serde_yaml::from_str(&yaml) {
        Ok(data) => Ok(data),
        Err(_) => Err(TaskErr {
            message: String::from("Unable to deserialize file"),
            task: None,
            task_type: None,
        }),
    }
}

fn get_task_type(task: &HashMap<String, Value>) -> TaskResult<TaskTypes> {
    if task.len() == 2 {
        let key: &String = task
            .keys()
            .filter(|element| !element.contains(NAME))
            .collect::<Vec<&String>>()[0];
        return match TaskTypes::from_str(&key) {
            Ok(k) => Ok(k),
            Err(e) => return Err(TaskErr {
                message: e.message,
                task: Some(task.clone()),
                task_type: None,
            })
        }
    }
    Err(TaskErr {
        message: String::from("Task data is Malformed"),
        task: Some(task.clone()),
        task_type: None,
    })
}

fn validate_first_task(task_data: &TaskData) -> TaskResult<()> {
    let first_task = task_data.tasks.first();
    if first_task.is_some() {
        let key: Vec<&String> = first_task
            .unwrap()
            .keys()
            .filter(|element| !element.contains(NAME))
            .collect::<Vec<&String>>();

        // if key.len() < 1 {
        //     return Err(format!(
        //         "Task Malformed:\n{:#?}",
        //         task_data.tasks.first().unwrap()
        //     ));
        // }

        let key = TaskTypes::from_str(&key[0])?;
        if key == TaskTypes::LINK {
            return Ok(());
        }
        return Err(TaskErr {
            message: String::from("First Task should be a Link"),
            task: None,
            task_type: None,
        });
    }

    Err(TaskErr {
        message: String::from("First Task not found"),
        task: None,
        task_type: None,
    })
}

fn is_last_task_close(task_data: &TaskData) -> TaskResult<bool> {
    let last_task = task_data.tasks.last();
    if last_task.is_some() {
        let key: Vec<&String> = last_task
            .unwrap()
            .keys()
            .filter(|element| !element.contains(NAME))
            .collect::<Vec<&String>>();

        // if key.len() < 1 {
        //     return Err(format!(
        //         "Task Malformed:\n{:#?}",
        //         task_data.tasks.last().unwrap()
        //     ));
        // }

        let key = TaskTypes::from_str(&key[0])?;
        if key == TaskTypes::CLOSE {
            return Ok(true);
        }
    }
    return Ok(false);
}

pub fn get_task<'a>(task: &'a HashMap<String, Value>, key: &str) -> TaskResult<&'a Mapping> {
    let task_data = match task.get(key) {
        Some(task_data) => task_data.as_mapping(),
        None => {
            return Err(TaskErr {
                message: String::from("Malformed Task"),
                task: Some(task.clone()),
                task_type: None,
            })
        }
    };

    let task_mapping = match task_data {
        Some(t) => t,
        None => {
            return Err(TaskErr {
                message: String::from("Task data is Malformed"),
                task: Some(task.clone()),
                task_type: None,
            })
        }
    };

    if task_mapping.is_empty() {
        return Err(TaskErr {
            message: String::from("Task data is empty"),
            task: Some(task.clone()),
            task_type: None,
        });
    }

    Ok(task_mapping)
}

pub fn get_task_name(task: &HashMap<String, Value>) -> TaskResult<String> {
    let task_value = match task.get(NAME) {
        Some(task_value) => task_value,
        None => {
            return Err(TaskErr {
                message: String::from("Malformed Task"),
                task: Some(task.clone()),
                task_type: None,
            })
        }
    };

    let name = match task_value.as_str() {
        Some(name) => name,
        None => {
            return Err(TaskErr {
                message: String::from("Task name is not a string"),
                task: Some(task.clone()),
                task_type: None,
            })
        }
    };

    if name.is_empty() {
        return Err(TaskErr {
            message: String::from("Task name can`t be empty"),
            task: Some(task.clone()),
            task_type: None,
        });
    }

    Ok(String::from(name))
}

#[derive(Debug, Clone)]
pub struct TaskOk {
    name: String,
    task_type: TaskTypes,
    duration: u64,
}

impl fmt::Display for TaskOk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?}: {} [{:#?}s]",
            self.task_type, self.name, self.duration
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskErr {
    message: String,
    task_type: Option<TaskTypes>,
    task: Option<HashMap<String, Value>>,
}

impl fmt::Display for TaskErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let task = self.task.clone().unwrap_or_default();

        write!(
            f,
            "{:?}: {}\n{:#?}",
            self.task_type.unwrap_or_default(),
            self.message,
            task
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_task_name_error() {
        let task: HashMap<String, Value> = HashMap::new();

        let result = get_task_name(&task).map_err(|e| e);
        let expected = Err(TaskErr {
            message: String::from("Malformed Task"),
            task: Some(task),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_get_task_name_empty() {
        let mut task: HashMap<String, Value> = HashMap::new();
        task.insert(String::from("name"), Value::from(""));

        let result = get_task_name(&task).map_err(|e| e);
        let expected = Err(TaskErr {
            message: String::from("Task name can`t be empty"),
            task: Some(task),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_get_task_name_not_string() {
        let mut task: HashMap<String, Value> = HashMap::new();
        task.insert(String::from("name"), Value::from(0));

        let result = get_task_name(&task).map_err(|e| e);
        let expected = Err(TaskErr {
            message: String::from("Task name is not a string"),
            task: Some(task),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_get_task_error() {
        let mut task: HashMap<String, Value> = HashMap::new();
        task.insert(String::from("name"), Value::from(0));

        let result = get_task(&task, "").map_err(|e| e);
        let expected = Err(TaskErr {
            message: String::from("Malformed Task"),
            task: Some(task.clone()),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_get_task_not_mapping() {
        let mut task: HashMap<String, Value> = HashMap::new();
        task.insert(String::from("name"), Value::from("foo"));
        task.insert(String::from("send_key"), Value::from(""));

        let result = get_task(&task, "send_key").map_err(|e| e);
        let expected = Err(TaskErr {
            message: String::from("Task data is Malformed"),
            task: Some(task.clone()),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_get_task_empty() {
        let mut task: HashMap<String, Value> = HashMap::new();
        task.insert(String::from("name"), Value::from("foo"));
        task.insert(String::from("send_key"), Value::from(Mapping::new()));

        let result = get_task(&task, "send_key").map_err(|e| e);
        let expected = Err(TaskErr {
            message: String::from("Task data is empty"),
            task: Some(task.clone()),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_is_last_task_close_true() {
        let yaml = "  
meta_data: {}
tasks:
  - name: 'Open link'
    link:
      url: 'link'  

  - name: 'enter rust in search'
    send_key:
      input: Rust
      element:
        xPath: 'foo'

  - name: 'press enter key'
    send_key:
      input: '\u{E007}'
      element:
        xPath: 'bar'

  - name: 'closing web driver session' 
    close: True
        ";

        let task: TaskData = serde_yaml::from_str(yaml).unwrap();
        let is_last = is_last_task_close(&task).unwrap();
        assert!(is_last);
    }

    #[test]
    fn test_is_last_task_close_false() {
        let yaml = "  
        meta_data: {}
        tasks:
          - name: 'Open link'
            link:
              url: 'link'  
        
          - name: 'enter rust in search'
            send_key:
              input: Rust
              element:
                xPath: 'foo'
        
          - name: 'press enter key'
            send_key:
              input: '\u{E007}'
              element:
                xPath: 'bar'
                ";

        let task: TaskData = serde_yaml::from_str(yaml).unwrap();
        let is_last = is_last_task_close(&task).unwrap();
        assert!(!is_last);
    }

    #[test]
    fn test_validate_first_task() {
        let yaml = "  
        meta_data: {}
        tasks:
          - name: 'Open link'
            link:
              url: 'link'  
        
          - name: 'enter rust in search'
            send_key:
              input: Rust
              element:
                xPath: 'foo'
        
          - name: 'press enter key'
            send_key:
              input: '\u{E007}'
              element:
                xPath: 'bar'
                ";

        let task: TaskData = serde_yaml::from_str(yaml).unwrap();
        let is_valid = validate_first_task(&task);
        assert!(is_valid.is_ok())
    }

    #[test]
    fn test_validate_first_task_error() {
        let yaml = "  
        meta_data: {}
        tasks:
          - name: 'enter rust in search'
            send_key:
              input: Rust
              element:
                xPath: 'foo'
        
          - name: 'press enter key'
            send_key:
              input: '\u{E007}'
              element:
                xPath: 'bar'
                ";

        let task: TaskData = serde_yaml::from_str(yaml).unwrap();
        let result = validate_first_task(&task).map_err(|e| e);
        let expected = Err(TaskErr {
            message: String::from("First Task should be a Link"),
            task: None,
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_get_task_type_send_key() {
        let mut task: HashMap<String, Value> = HashMap::new();
        task.insert(String::from("name"), Value::from("foo"));
        task.insert(String::from("send_key"), Value::from(Mapping::new()));

        let task_type = get_task_type(&task).unwrap();
        assert_eq!(TaskTypes::SENDKEY, task_type)
    }

    #[test]
    fn test_get_task_type_click() {
        let mut task: HashMap<String, Value> = HashMap::new();
        task.insert(String::from("name"), Value::from("foo"));
        task.insert(String::from("click"), Value::from(Mapping::new()));

        let task_type = get_task_type(&task).unwrap();
        assert_eq!(TaskTypes::CLICK, task_type)
    }

    #[test]
    fn test_get_task_type_unknow() {
        let mut task: HashMap<String, Value> = HashMap::new();
        task.insert(String::from("name"), Value::from("foo"));
        task.insert(String::from("foo"), Value::from(Mapping::new()));

        let result = get_task_type(&task).map_err(|e| e);
        let expected = Err(TaskErr {
            message: String::from("Unknow Task Type: \"foo\""),
            task: Some(task.clone()),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_get_task_type_error() {
        let mut task: HashMap<String, Value> = HashMap::new();
        task.insert(String::from("name"), Value::from("foo"));

        let result = get_task_type(&task).map_err(|e| e);
        let expected = Err(TaskErr {
            message: String::from("Task data is Malformed"),
            task: Some(task.clone()),
            task_type: None,
        });
        assert_eq!(expected, result)
    }
}
