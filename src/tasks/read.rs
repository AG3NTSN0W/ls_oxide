use std::{collections::HashMap, str::FromStr};

use async_trait::async_trait;
use serde_yaml::{Mapping, Value};
use std::time::Instant;

use crate::{
    element::Element,
    executor::{ExecuteResult, WebDriverSession},
};

use super::{get_task, get_task_name, Task, TaskErr, TaskOk, TaskResult, TaskTypes};

#[derive(Debug, PartialEq, Eq)]
pub enum Property {
    TEXT,
    INNERHTML,
    VALUE,
}

impl FromStr for Property {
    type Err = String;

    fn from_str(input: &str) -> Result<Property, Self::Err> {
        match input {
            "text" => Ok(Property::TEXT),
            "innerHtml" => Ok(Property::INNERHTML),
            "value" => Ok(Property::VALUE),
            _ => Err(format!("Unknow Element Type: {:#?}", input)),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
struct Fields {
    element: Element,
    key: String,
    property: Property,
}

impl Fields {
    pub fn from(mapping: &Mapping) -> Result<Fields, String> {
        let element = Element::new(mapping)?;

        let key = Fields::get_key_from_mapping(mapping)?;

        let property = Fields::get_property_from_mapping(mapping)?;

        Ok(Fields {
            element,
            key,
            property,
        })
    }

    fn get_key_from_mapping(mapping: &Mapping) -> Result<String, String> {
        let key: &Value = match mapping.get("key") {
            None => return Err("Key not found".to_string()),
            Some(key) => key,
        };

        if let Some(key) = key.as_str() {
            return Ok(key.to_string());
        }

        Err("Key is not a sting".to_string())
    }

    fn get_property_from_mapping(mapping: &Mapping) -> Result<Property, String> {
        let property: &Value = match mapping.get("property") {
            None => return Err("property not found".to_string()),
            Some(property) => property,
        };

        if let Some(property) = property.as_str() {
            return Ok(Property::from_str(property)?);
        }

        Err("property is not a sting".to_string())
    }
}

const TASK_TYPE: &str = "read";

#[derive(PartialEq, Eq, Debug)]
pub struct Read {
    _task_types: TaskTypes,
    name: String,
    element: Element,
    fields: Vec<Fields>,
}

#[async_trait]
impl Task for Read {
    fn new(task: &HashMap<String, Value>) -> TaskResult<Read> {
        let name = get_task_name(&task)?;

        let read = get_task(task, TASK_TYPE)?;

        let element = match Element::new(read) {
            Ok(element) => element,
            Err(err) => return Err(TaskErr::new(err, Some(TaskTypes::READ), Some(task.clone()))),
        };

        let fields = match get_properties(read) {
            Ok(fields) => fields,
            Err(err) => return Err(TaskErr::new(err, Some(TaskTypes::READ), Some(task.clone()))),
        };

        Ok(Read {
            _task_types: TaskTypes::READ,
            name,
            element,
            fields,
        })
    }

    async fn execute(&self, web_driver_session: WebDriverSession) -> ExecuteResult {
        let start = Instant::now();

        let name = self.name.clone();
        return Ok((
            web_driver_session,
            TaskOk {
                name,
                task_type: TaskTypes::READ,
                duration: start.elapsed().as_secs(),
                result: None,
            },
        ));
    }
}

fn get_properties(task: &Mapping) -> Result<Vec<Fields>, String> {
    let mut properies: Vec<Fields> = Vec::new();

    let fields = match task.get("fields") {
        None => return Err("Fields propery not found".to_string()),
        Some(fields) => fields,
    };

    let fields = match fields.as_sequence() {
        None => return Err("Fields propery not found".to_string()),
        Some(fields) => fields,
    };

    for field in fields {
        let mapping = match field.as_mapping() {
            None => return Err("Fields propery not found".to_string()),
            Some(map) => map,
        };

        properies.push(Fields::from(mapping)?);
    }

    Ok(properies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{element::ElementType, assert_task_ok, assert_task_err};

    #[test]
    fn test_task() {
        let yaml = "  
        name: 'read data'
        read:
          element:
            id: 'search-form'
          fields:  
            - element:
                id: 'foo'
              key: 'bar'
              property: 'text'   
            - element:
                id: 'bob'
              key: 'foo'
              property: 'text'    
        ";

        let mut fields: Vec<Fields> = Vec::new();
        fields.push(Fields {
            element: Element {
                element_type: ElementType::ID,
                value: "foo".to_string(),
            },
            key: "bar".to_string(),
            property: Property::TEXT,
        });
        fields.push(Fields {
            element: Element {
                element_type: ElementType::ID,
                value: "bob".to_string(),
            },
            key: "foo".to_string(),
            property: Property::TEXT,
        });

        let expected = Read {
            _task_types: TaskTypes::READ,
            name: "read data".to_owned(),
            element: Element {
                element_type: ElementType::ID,
                value: "search-form".to_owned(),
            },
            fields,
        };

        assert_task_ok!(yaml, Read, expected)
    }
}
