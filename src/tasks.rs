mod click;
mod close;
mod link;
mod screenshot;
mod send_key;
mod set_variable;
mod validate;
mod wait;

use crate::executor::{ExecuteResult, WebDriverSession};
use crate::structs::task_results::ResultsType;
use crate::structs::{task_data::TaskData, task_err::TaskErr, task_ok::TaskOk};

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
use self::set_variable::SetVars;
use self::validate::Validate;
use self::wait::Wait;
use async_trait::async_trait;
use core::fmt::Debug;

pub type Tasks = Vec<Box<dyn Task>>;
pub type TaskResult<T> = std::result::Result<T, TaskErr>;
pub type TaskToExecute = Vec<HashMap<String, Value>>;

const NAME: &str = "name";


#[async_trait]
pub trait Task {
    async fn execute(&self, web_driver_session: WebDriverSession) -> ExecuteResult;
    fn new(task: &HashMap<String, Value>) -> TaskResult<Self>
    where
        Self: Sized;
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
    SETVARIABLE,
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
            "validate" => Ok(TaskTypes::VALIDATE),
            "set_vars" => Ok(TaskTypes::SETVARIABLE),
            _ => Err(TaskErr::new(
                format!("Unknow Task Type: {:#?}", input),
                None,
                None,
            )),
        }
    }
}

impl fmt::Display for TaskTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TaskTypes::CLICK => write!(f, "click"),
            TaskTypes::SENDKEY => write!(f, "send_key"),
            TaskTypes::LINK => write!(f, "link"),
            TaskTypes::CLOSE => write!(f, "close"),
            TaskTypes::WAIT => write!(f, "wait"),
            TaskTypes::SCREENSHOT => write!(f, "screenshot"),
            TaskTypes::VALIDATE => write!(f, "validate"),
            TaskTypes::SETVARIABLE => write!(f, "set_vars"),
            TaskTypes::NONE => write!(f, "none"),
        }
    }
}


pub fn to_task(path: PathBuf) -> TaskResult<(Tasks, ResultsType)> {

    let (task_data, task_type) = get_task_data(path)?;

    let mut tasks: Vec<Box<dyn Task>> = vec![];
    for task_data in task_data.iter() {
        tasks.push(data_to_task(task_data)?);
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
    Ok((tasks, task_type))
}

fn data_to_task(task_data: &HashMap<String, Value>) -> TaskResult<Box<dyn Task>> {
    let task_type = get_task_type(task_data)?;
    let task: Box<dyn Task> = match task_type {
        TaskTypes::SENDKEY => Box::new(<SendKey as Task>::new(task_data)?),
        TaskTypes::CLICK => Box::new(<Click as Task>::new(task_data)?),
        TaskTypes::CLOSE => Box::new(<Close as Task>::new(task_data)?),
        TaskTypes::LINK => Box::new(<Link as Task>::new(task_data)?),
        TaskTypes::WAIT => Box::new(<Wait as Task>::new(task_data)?),
        TaskTypes::SCREENSHOT => Box::new(<Screenshot as Task>::new(task_data)?),
        TaskTypes::VALIDATE => Box::new(<Validate as Task>::new(task_data)?),
        TaskTypes::SETVARIABLE => Box::new(<SetVars as Task>::new(task_data)?),
        _ => {
            return Err(TaskErr::new(
                "Invalid Task Type".to_string(),
                Some(TaskTypes::NONE),
                Some(task_data.clone()),
            ))
        }
    };
    Ok(task)
}

pub fn get_task_data(path: PathBuf) -> TaskResult<(TaskToExecute, ResultsType)> {
    let yaml = match fs::read_to_string(path) {
        Ok(data) => data,
        Err(_) => {
            return Err(TaskErr::new(
                String::from("Unable to read File"),
                None,
                None,
            ))
        }
    };

    let task_data: TaskData = match serde_yaml::from_str(&yaml) {
        Ok(tasks) => tasks,
        Err(_) => return  Err(TaskErr::new(
            String::from("Unable to deserialize file"),
            None,
            None,
        )),
    };

    if let Some(data) = task_data.tasks {
        return Ok((data, ResultsType::TASk));
    } 

    if let Some(data) = task_data.validate {
        return Ok((data, ResultsType::VALIDATE));
    }

   Err(TaskErr::new(
        String::from("Unable to deserialize file"),
        None,
        None,
    ))
}

fn get_task_type(task: &HashMap<String, Value>) -> TaskResult<TaskTypes> {
    if task.len() == 2 {
        let key: &String = task
            .keys()
            .filter(|element| !element.contains(NAME))
            .collect::<Vec<&String>>()[0];
        return match TaskTypes::from_str(key) {
            Ok(k) => Ok(k),
            Err(e) => {
                let mut task_err = e;
                task_err.set_task(Some(task.clone()));
                return Err(task_err);
            }
        };
    }
    Err(TaskErr::new(
        String::from("Task data is Malformed"),
        None,
        Some(task.clone()),
    ))
}

fn validate_first_task(task_data: &TaskToExecute) -> TaskResult<()> {
    let first_task = task_data.first();
    if let Some(first) = first_task {
        let key: Vec<&String> = first
            .keys()
            .filter(|element| !element.contains(NAME))
            .collect::<Vec<&String>>();

        let key = TaskTypes::from_str(key[0])?;
        if key == TaskTypes::LINK {
            return Ok(());
        }
        return Err(TaskErr::new(
            String::from("First Task should be a Link"),
            None,
            None,
        ));
    }

    Err(TaskErr::new(
        String::from("First Task not found"),
        None,
        None,
    ))
}

fn is_last_task_close(task_data: &TaskToExecute) -> TaskResult<bool> {
    let last_task = task_data.last();
    if let Some(last) = last_task {
        let key: Vec<&String> = last
            .keys()
            .filter(|element| !element.contains(NAME))
            .collect::<Vec<&String>>();

        let key = TaskTypes::from_str(key[0])?;
        if key == TaskTypes::CLOSE {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn get_task<'a>(task: &'a HashMap<String, Value>, key: &str) -> TaskResult<&'a Mapping> {
    let task_data = match task.get(key) {
        Some(task_data) => task_data.as_mapping(),
        None => {
            return Err(TaskErr::new(
                String::from("Malformed Task"),
                None,
                Some(task.clone()),
            ))
        }
    };

    let task_mapping = match task_data {
        Some(t) => t,
        None => {
            return Err(TaskErr::new(
                String::from("Task data is Malformed"),
                None,
                Some(task.clone()),
            ))
        }
    };

    if task_mapping.is_empty() {
        return Err(TaskErr::new(
            String::from("Task data is empty"),
            None,
            Some(task.clone()),
        ));
    }

    Ok(task_mapping)
}

pub fn get_task_name(task: &HashMap<String, Value>) -> TaskResult<String> {
    let task_value = match task.get(NAME) {
        Some(task_value) => task_value,
        None => {
            return Err(TaskErr::new(
                String::from("Malformed Task"),
                None,
                Some(task.clone()),
            ))
        }
    };

    let name = match task_value.as_str() {
        Some(name) => name,
        None => {
            return Err(TaskErr::new(
                String::from("Task name is not a string"),
                None,
                Some(task.clone()),
            ))
        }
    };

    if name.is_empty() {
        return Err(TaskErr::new(
            String::from("Task name can`t be empty"),
            None,
            Some(task.clone()),
        ));
    }

    Ok(String::from(name))
}

fn to_hash(task_data: &Mapping) -> Result<HashMap<String, String>, String> {
    let mut task_hash: HashMap<String, String> = HashMap::new();

    for (key, value) in task_data {
        let key = match key.as_str() {
            None => return Err(format!("Key: {:?} is not a string", key)),
            Some(k) => k.to_owned(),
        };

        let value = match value.as_str() {
            None => return Err(format!("Value: {:?} is not a string", value)),
            Some(v) => v.to_owned(),
        };

        task_hash.insert(key, value);
    }

    Ok(task_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_task_name_error() {
        let task: HashMap<String, Value> = HashMap::new();

        let result = get_task_name(&task);
        let expected = Err(TaskErr::new(
            String::from("Malformed Task"),
            None,
            Some(task),
        ));
        assert_eq!(expected, result)
    }

    #[test]
    fn test_get_task_name_empty() {
        let mut task: HashMap<String, Value> = HashMap::new();
        task.insert(String::from("name"), Value::from(""));

        let result = get_task_name(&task);
        let expected = Err(TaskErr::new(
            String::from("Task name can`t be empty"),
            None,
            Some(task),
        ));
        assert_eq!(expected, result)
    }

    #[test]
    fn test_get_task_name_not_string() {
        let mut task: HashMap<String, Value> = HashMap::new();
        task.insert(String::from("name"), Value::from(0));

        let result = get_task_name(&task);
        let expected = Err(TaskErr::new(
            String::from("Task name is not a string"),
            None,
            Some(task),
        ));
        assert_eq!(expected, result)
    }

    #[test]
    fn test_get_task_error() {
        let mut task: HashMap<String, Value> = HashMap::new();
        task.insert(String::from("name"), Value::from(0));

        let result = get_task(&task, "");
        let expected = Err(TaskErr::new(
            String::from("Malformed Task"),
            None,
            Some(task.clone()),
        ));
        assert_eq!(expected, result)
    }

    #[test]
    fn test_get_task_not_mapping() {
        let mut task: HashMap<String, Value> = HashMap::new();
        task.insert(String::from("name"), Value::from("foo"));
        task.insert(String::from("send_key"), Value::from(""));

        let result = get_task(&task, "send_key");
        let expected = Err(TaskErr::new(
            String::from("Task data is Malformed"),
            None,
            Some(task.clone()),
        ));
        assert_eq!(expected, result)
    }

    #[test]
    fn test_get_task_empty() {
        let mut task: HashMap<String, Value> = HashMap::new();
        task.insert(String::from("name"), Value::from("foo"));
        task.insert(String::from("send_key"), Value::from(Mapping::new()));

        let result = get_task(&task, "send_key");
        let expected = Err(TaskErr::new(
            String::from("Task data is empty"),
            None,
            Some(task.clone()),
        ));
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
        let task = task.tasks.unwrap();
        
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
        let task = task.tasks.unwrap();
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
        let task = task.tasks.unwrap();
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
        let task = task.tasks.unwrap();
        let result = validate_first_task(&task);
        let expected = Err(TaskErr::new(
            String::from("First Task should be a Link"),
            None,
            None,
        ));
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

        let result = get_task_type(&task);
        let expected = Err(TaskErr::new(
            String::from("Unknow Task Type: \"foo\""),
            None,
            Some(task.clone()),
        ));
        assert_eq!(expected, result)
    }

    #[test]
    fn test_get_task_type_error() {
        let mut task: HashMap<String, Value> = HashMap::new();
        task.insert(String::from("name"), Value::from("foo"));

        let result = get_task_type(&task);
        let expected = Err(TaskErr::new(
            String::from("Task data is Malformed"),
            None,
            Some(task.clone()),
        ));
        assert_eq!(expected, result)
    }
}
