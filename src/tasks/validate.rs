use std::collections::HashMap;

use async_trait::async_trait;
use serde_yaml::{Mapping, Value};
use std::time::Instant;

use crate::{
    element::Element,
    executor::{ExecuteResult, WebDriverSession},
};

use super::{get_task, get_task_name, Task, TaskErr, TaskOk, TaskResult, TaskTypes};

const TASK_TYPE: &str = "validate";
#[derive(PartialEq, Eq, Debug)]
pub enum ValidateTypes {
    Text(String),
    Value(String),
    InnerHtml(String),
    Css(HashMap<String, String>),
    Property(HashMap<String, String>),
}

#[derive(PartialEq, Eq, Debug)]
pub struct Validate {
    _task_types: TaskTypes,
    name: String,
    element: Element,
    expect: Vec<ValidateTypes>,
}

#[async_trait]
impl Task for Validate {
    fn new(task: &HashMap<String, Value>) -> TaskResult<Validate> {
        let name = get_task_name(task)?;
        let validate = get_task(task, TASK_TYPE)?;

        let element = match Element::new(validate) {
            Ok(element) => element,
            Err(err) => {
                return Err(TaskErr {
                    message: err,
                    task: Some(task.clone()),
                    task_type: Some(TaskTypes::CLICK),
                })
            }
        };

        let expect = get_expect(task)?;

        Ok(Validate {
            _task_types: TaskTypes::VALIDATE,
            name,
            element,
            expect,
        })
    }

    async fn execute(&self, web_driver_session: WebDriverSession) -> ExecuteResult {
        let start = Instant::now();

        let name = self.name.clone();
        return Ok((
            web_driver_session,
            TaskOk {
                name,
                task_type: TaskTypes::VALIDATE,
                duration: start.elapsed().as_secs(),
            },
        ));
    }
}

fn get_expect(task: &HashMap<String, Value>) -> TaskResult<Vec<ValidateTypes>> {
    let task_data = match task.get(TASK_TYPE) {
        Some(task_data) => task_data.as_mapping(),
        None => {
            return Err(TaskErr {
                message: String::from("Validate Task is no a map"),
                task: Some(task.clone()),
                task_type: Some(TaskTypes::VALIDATE),
            })
        }
    };

    let task_mapping = match task_data {
        Some(t) => t,
        None => {
            return Err(TaskErr {
                message: String::from("Validate Task is is Malformed"),
                task: Some(task.clone()),
                task_type: Some(TaskTypes::VALIDATE),
            })
        }
    };

    if task_mapping.is_empty() {
        return Err(TaskErr {
            message: String::from("Validate Task is empty"),
            task: Some(task.clone()),
            task_type: Some(TaskTypes::VALIDATE),
        });
    }

    validate_task(task_mapping)
}

fn validate_task(task_mapping: &Mapping) -> TaskResult<Vec<ValidateTypes>> {
    let mut to_validate: Vec<ValidateTypes> = Vec::new();

    if let Some(v) = validate_text_data(task_mapping) {
        to_validate.push(v);
    }

    if let Some(v) = validate_value_data(task_mapping) {
        to_validate.push(v);
    }

    if let Some(v) = validate_inner_html_data(task_mapping) {
        to_validate.push(v);
    }

    if let Some(v) = validate_css_data(task_mapping) {
        to_validate.push(v);
    }

    if let Some(v) = validate_property_data(task_mapping) {
        to_validate.push(v);
    }

    Ok(to_validate)
}

fn validate_text_data(task_mapping: &Mapping) -> Option<ValidateTypes> {
    let task_data = task_mapping.get("text")?.as_str()?;

    if task_data.is_empty() {
        return None;
    }

    Some(ValidateTypes::Text(String::from(task_data)))
}

fn validate_value_data(task_mapping: &Mapping) -> Option<ValidateTypes> {
    let task_data = task_mapping.get("value")?.as_str()?;

    if task_data.is_empty() {
        return None;
    }

    Some(ValidateTypes::Value(String::from(task_data)))
}

fn validate_inner_html_data(task_mapping: &Mapping) -> Option<ValidateTypes> {
    let task_data = task_mapping.get("innerHtml")?.as_str()?;

    if task_data.is_empty() {
        return None;
    }

    Some(ValidateTypes::InnerHtml(String::from(task_data)))
}

fn validate_css_data(task_mapping: &Mapping) -> Option<ValidateTypes> {
    let task_data = task_mapping.get("css")?.as_mapping()?;

    if task_data.is_empty() {
        return None;
    }

    let task_data = to_hash(task_data);

    if task_data.is_empty() {
        return None;
    }

    Some(ValidateTypes::Css(task_data))
}

fn validate_property_data(task_mapping: &Mapping) -> Option<ValidateTypes> {
    let task_data = task_mapping.get("property")?.as_mapping()?;

    if task_data.is_empty() {
        return None;
    }

    let task_data = to_hash(task_data);

    if task_data.is_empty() {
        return None;
    }

    Some(ValidateTypes::Property(task_data))
}

fn to_hash(task_data: &Mapping) -> HashMap<String, String> {
    let mut task_hash: HashMap<String, String> = HashMap::new();

    for (key, value) in task_data {
        let key = match key.as_str() {
            None => continue,
            Some(k) => k.to_owned(),
        };

        let value = match value.as_str() {
            None => continue,
            Some(v) => v.to_owned(),
        };

        task_hash.insert(key, value);
    }

    task_hash
}

#[cfg(test)]
mod tests {
    

    use super::*;

    #[test]
    fn test_empty_task() {
        let click_map_empty = HashMap::new();
        let result = Validate::new(&click_map_empty);
        let expected = Err(TaskErr {
            message: String::from("Malformed Task"),
            task: Some(click_map_empty),
            task_type: None,
        });
        assert_eq!(expected, result)
    }
}