use std::collections::HashMap;

use async_trait::async_trait;
use serde_yaml::Value;
use std::time::Instant;

use crate::{executor::{ExecuteResult, WebDriverSession}};

use super::{get_task_name, Task, TaskErr, TaskOk, TaskResult, TaskTypes, get_task, to_hash};


const TASK_TYPE: &str = "set_vars";

#[derive(PartialEq, Eq, Debug)]
pub struct SetVars {
    _task_types: TaskTypes,
    name: String,
    variables: HashMap<String, String>
}

#[async_trait]
impl Task for SetVars {
    fn new(task: &HashMap<String, Value>) -> TaskResult<SetVars> {
        let name = get_task_name(task)?;
        let variables = get_task(task, TASK_TYPE)?;

        let variables: HashMap<String, String> = match to_hash(variables) {
            Ok(v) => v,
            Err(e) => {
                return Err(TaskErr {
                    message: e,
                    task: Some(task.clone()),
                    task_type: Some(TaskTypes::SETVARIABLE),
                });
            }
        };

        Ok(SetVars {
            _task_types: TaskTypes::SETVARIABLE,
            name,
            variables
        })
    }

    async fn execute(&self, mut web_driver_session: WebDriverSession) -> ExecuteResult {
        let start = Instant::now();
        
        for (key, value) in self.variables.iter() { 
            web_driver_session.add_variable(key, value);
         }

        let name = self.name.clone();
        return Ok((
            web_driver_session,
            TaskOk {
                name,
                task_type: TaskTypes::SETVARIABLE,
                duration: start.elapsed().as_secs(),
                result: None
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_task() {
        let variable = HashMap::new();
        let result = SetVars::new(&variable);
        let expected = Err(TaskErr {
            message: String::from("Malformed Task"),
            task: Some(variable),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_missing_task_data() {
        let yaml = "  
                name: 'set vars'
              ";

        let variable = serde_yaml::from_str(yaml).unwrap();
        let result = SetVars::new(&variable);
        let expected = Err(TaskErr {
            message: String::from("Malformed Task"),
            task: Some(variable),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_invalid_task_data_str() {
        let yaml = "  
                name: 'set vars'
                set_vars: 'foo'
              ";

        let variable = serde_yaml::from_str(yaml).unwrap();
        let result = SetVars::new(&variable);
        let expected = Err(TaskErr {
            message: String::from("Task data is Malformed"),
            task: Some(variable),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_invalid_task_data_int() {
        let yaml = "  
                name: 'set vars'
                set_vars: 2
              ";

        let variable = serde_yaml::from_str(yaml).unwrap();
        let result = SetVars::new(&variable);
        let expected = Err(TaskErr {
            message: String::from("Task data is Malformed"),
            task: Some(variable),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_invalid_task_data_list() {
        let yaml = "  
                name: 'set vars'
                set_vars: 
                    - 'bar'
              ";

        let variable = serde_yaml::from_str(yaml).unwrap();
        let result = SetVars::new(&variable);
        let expected = Err(TaskErr {
            message: String::from("Task data is Malformed"),
            task: Some(variable),
            task_type: None,
        });
        assert_eq!(expected, result)
    }


    #[test]
    fn test_invalid_variable_key_int() {
        let yaml = "  
                name: 'set vars'
                set_vars: 
                    2: 'foo'
              ";

        let variable = serde_yaml::from_str(yaml).unwrap();
        let result = SetVars::new(&variable);
        let expected = Err(TaskErr {
            message: String::from("Key: Number(2) is not a string"),
            task: Some(variable),
            task_type: Some(TaskTypes::SETVARIABLE),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_invalid_variable_key_bool() {
        let yaml = "  
                name: 'set vars'
                set_vars: 
                    True: 'foo'
              ";

        let variable = serde_yaml::from_str(yaml).unwrap();
        let result = SetVars::new(&variable);
        let expected = Err(TaskErr {
            message: String::from("Key: Bool(true) is not a string"),
            task: Some(variable),
            task_type: Some(TaskTypes::SETVARIABLE),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task() {
        let yaml = "  
                name: 'set vars'
                set_vars: 
                    name: 'foo'
                    age: '42'
              ";

        let mut variables: HashMap<String, String> = HashMap::new();      
        variables.insert("name".to_string(), "foo".to_string());
        variables.insert("age".to_string(), "42".to_string());


        let variable = serde_yaml::from_str(yaml).unwrap();
        let result = SetVars::new(&variable);
        let expected = Ok(SetVars {
            name: "set vars".to_string(),
            _task_types: TaskTypes::SETVARIABLE,
            variables
        });
        assert_eq!(expected, result)
    }
}
