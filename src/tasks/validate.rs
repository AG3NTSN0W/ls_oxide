use std::collections::HashMap;

use async_trait::async_trait;
use serde_yaml::{Mapping, Value};
use std::time::Instant;
use thirtyfour::WebElement;

use crate::{
    element::Element,
    executor::{ExecuteResult, WebDriverSession}, variables::resolve_variables,
};

use super::{get_task, get_task_name, Task, TaskErr, TaskOk, TaskResult, TaskTypes, ValidationReult, ValidationReultType, to_hash};

const TASK_TYPE: &str = "validate";
#[derive(PartialEq, Eq, Debug)]
pub enum ValidateTypes {
    Text(String),
    InnerHtml(String),
    Css(HashMap<String, String>),
    Property(HashMap<String, String>),
}

#[derive(PartialEq, Eq, Debug)]
pub struct Validate {
    _task_types: TaskTypes,
    name: String,
    element: Element,
    expects: Vec<ValidateTypes>,
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

        let expects = get_expects(task)?;

        Ok(Validate {
            _task_types: TaskTypes::VALIDATE,
            name,
            element,
            expects,
        })
    }

    async fn execute(&self, web_driver_session: WebDriverSession) -> ExecuteResult {
        let start = Instant::now();

        let web_element = match web_driver_session
            .driver
            .find(Element::find_by(&self.element))
            .await
        {
            Ok(element) => element,
            Err(e) => {
                return Err((
                    web_driver_session,
                    TaskErr {
                        message: format!("{}", e),
                        task: None,
                        task_type: Some(TaskTypes::VALIDATE),
                    },
                ));
            }
        };

        let results =
            match validate(&self.expects, web_element, &web_driver_session.variables).await {
                Ok(r) => r,
                Err(e) => {
                    return Err((
                        web_driver_session,
                        TaskErr {
                            message: e,
                            task: None,
                            task_type: Some(TaskTypes::VALIDATE),
                        },
                    ))
                }
            };

        let name = self.name.clone();
        return Ok((
            web_driver_session,
            TaskOk {
                name,
                task_type: TaskTypes::VALIDATE,
                duration: start.elapsed().as_secs(),
                result: Some(results),
            },
        ));
    }
}

async fn validate(
    expects: &Vec<ValidateTypes>,
    web_element: WebElement,
    variables: &HashMap<String, String>,
) -> Result<Vec<ValidationReult>, String> {
    let mut results: Vec<ValidationReult> = Vec::new();

    for expect in expects {
        match expect {
            ValidateTypes::Text(expect) => {
                results.push(validate_text(expect, &web_element, variables).await)
            }
            ValidateTypes::InnerHtml(expect) => {
                results.push(validate_inner_html(expect, &web_element).await)
            }
            ValidateTypes::Css(expect) => {
                results.append(&mut validate_css(expect, &web_element, variables).await);
            }
            ValidateTypes::Property(expect) => {
                results.append(&mut validate_property(expect, &web_element, variables).await)
            }
        }
    }

    Ok(results)
}

async fn validate_text(
    expect: &String,
    web_element: &WebElement,
    variables: &HashMap<String, String>,
) -> ValidationReult {
    let expect = resolve_variables(expect, variables);

    let actual: String = match web_element.text().await {
        Ok(s) => s,
        Err(e) => {
            return ValidationReult {
                validation: ValidationReultType::FAILED,
                message: e.to_string(),
            }
        }
    };

    if actual.eq(&expect) {
        return ValidationReult {
            validation: ValidationReultType::SUCCESS,
            message: format!("Pass: Text is {}", expect),
        };
    }
    ValidationReult {
        validation: ValidationReultType::FAILED,
        message: format!("Failed: Text expected: [{}], actual: [{}]", expect, actual),
    }
}

async fn validate_inner_html(expect: &String, web_element: &WebElement) -> ValidationReult {
    let actual: String = match web_element.inner_html().await {
        Ok(s) => s,
        Err(e) => {
            return ValidationReult {
                validation: ValidationReultType::FAILED,
                message: e.to_string(),
            }
        }
    };

    if actual.eq(expect) {
        return ValidationReult {
            validation: ValidationReultType::SUCCESS,
            message: format!("Pass: InnerHtml is {}", expect),
        };
    }

    ValidationReult {
        validation: ValidationReultType::FAILED,
        message: format!(
            "Failed: InnerHtml expected: [{}], actual: [{}]",
            expect, actual
        ),
    }
}

async fn validate_css(
    expected: &HashMap<String, String>,
    web_element: &WebElement,
    variables: &HashMap<String, String>,
) -> Vec<ValidationReult> {
    let mut results: Vec<ValidationReult> = Vec::new();

    for (css_value, expect) in expected {
        let actual: String = match web_element.css_value(css_value).await {
            Ok(s) => s,
            Err(e) => {
                results.push(ValidationReult {
                    validation: ValidationReultType::FAILED,
                    message: format!("Failed: Css error {}", e),
                });
                continue;
            }
        };

        let expect = resolve_variables(expect, variables);
        if actual.eq(&expect) {
            results.push(ValidationReult {
                validation: ValidationReultType::SUCCESS,
                message: format!("Pass: CSS [{}] is [{}]", css_value, expect),
            });
            continue;
        }

        results.push(ValidationReult {
            validation: ValidationReultType::FAILED,
            message: format!(
                "Failed: CSS [{}] was [{}] expect [{}]",
                css_value, actual, expect
            ),
        })
    }

    results
}

async fn validate_property(
    expected: &HashMap<String, String>,
    web_element: &WebElement,
    variables: &HashMap<String, String>,
) -> Vec<ValidationReult> {
    let mut results: Vec<ValidationReult> = Vec::new();

    for (prop, expect) in expected {
        let prop_value: Option<String> = match web_element.prop(prop).await {
            Ok(prop) => prop,
            Err(e) => {
                results.push(ValidationReult {
                    validation: ValidationReultType::FAILED,
                    message: format!("Failed: property error {}", e),
                });
                continue;
            }
        };

        let expect = resolve_variables(expect, variables);
        if let Some(actual) = prop_value {
            if actual.eq(&expect) {
                results.push(ValidationReult {
                    validation: ValidationReultType::SUCCESS,
                    message: format!("Pass: property [{}] is [{}]", actual, expect),
                });
                continue;
            }
            results.push(ValidationReult {
                validation: ValidationReultType::FAILED,
                message: format!(
                    "Failed: property [{}] was [{}] expected [{}]",
                    prop, actual, expect
                ),
            });
            continue;
        }

        results.push(ValidationReult {
            validation: ValidationReultType::FAILED,
            message: format!("Failed: property [{}] not found", prop),
        })
    }

    results
}

fn get_expects(task: &HashMap<String, Value>) -> TaskResult<Vec<ValidateTypes>> {
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

    match validate_task(task_mapping) {
        Ok(v) => Ok(v),
        Err(e) => Err(TaskErr {
            message: e,
            task: Some(task.clone()),
            task_type: Some(TaskTypes::VALIDATE),
        }),
    }
}

fn validate_task(task_mapping: &Mapping) -> Result<Vec<ValidateTypes>, String> {
    let mut to_validate: Vec<ValidateTypes> = Vec::new();

    let expect_data = match task_mapping.get("expect") {
        None => return Err("Validate Task is Malformed".to_string()),
        Some(data) => data,
    };

    let expect_data = match expect_data.as_mapping() {
        None => return Err("Validate expect is not a map".to_string()),
        Some(data) => data,
    };

    for expect in ["text", "innerHtml", "css", "property"] {
        match expect {
            "text" | "innerHtml" => {
                if let Some(v) = validate_data_string(expect_data, expect)? {
                    to_validate.push(v);
                }
            }
            "css" | "property" => {
                if let Some(v) = validate_data_mapping(expect_data, expect)? {
                    to_validate.push(v);
                }
            }
            _ => continue,
        }
    }

    Ok(to_validate)
}

fn validate_data_string(
    task_mapping: &Mapping,
    key: &str,
) -> Result<Option<ValidateTypes>, String> {
    if let Some(task_data) = task_mapping.get(key) {
        let value = match task_data.as_str() {
            None => return Err(format!("{} - value is not a string", key)),
            Some(value) => value,
        };

        if value.is_empty() {
            return Ok(None);
        }

        let validate = match key {
            "text" => Some(ValidateTypes::Text(value.to_string())),
            "innerHtml" => Some(ValidateTypes::InnerHtml(value.to_string())),
            _ => None,
        };

        return Ok(validate);
    }

    Ok(None)
}

fn validate_data_mapping(
    task_mapping: &Mapping,
    key: &str,
) -> Result<Option<ValidateTypes>, String> {
    if let Some(task_data) = task_mapping.get(key) {
        let value = match task_data.as_mapping() {
            None => return Err(format!("{} value is not a map", key)),
            Some(value) => value,
        };

        if value.is_empty() {
            return Ok(None);
        }

        let value = to_hash(value)?;

        let validate = match key {
            "css" => Some(ValidateTypes::Css(value)),
            "property" => Some(ValidateTypes::Property(value)),
            _ => None,
        };

        return Ok(validate);
    }

    Ok(None)
}

#[cfg(test)]
mod tests {

    use crate::element::ElementType;

    use super::*;

    #[test]
    fn test_empty_task() {
        let validate = HashMap::new();
        let result = Validate::new(&validate);
        let expected = Err(TaskErr {
            message: String::from("Malformed Task"),
            task: Some(validate),
            task_type: None,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_expect() {
        let yaml = "  
        name: 'validate'
        validate:
          element:
            xPath: '//*[@id=\"search-form\"]/fieldset/button'
          expect:
            css:
              color: 'rgba(0, 0, 0, 0)'
              'text-indent': '-10000px'
            property:
              name: 'foo'  
            text: 'Text'
            innerHtml: 'InnerHtml'      
              ";

        let mut expect_vec: Vec<ValidateTypes> = Vec::new();
        expect_vec.push(ValidateTypes::Text("Text".to_string()));
        expect_vec.push(ValidateTypes::InnerHtml("InnerHtml".to_string()));

        let mut css_map: HashMap<String, String> = HashMap::new();
        css_map.insert("color".to_string(), "rgba(0, 0, 0, 0)".to_string());
        css_map.insert("text-indent".to_string(), "-10000px".to_string());

        let mut property_map: HashMap<String, String> = HashMap::new();
        property_map.insert("name".to_string(), "foo".to_string());

        expect_vec.push(ValidateTypes::Css(css_map));
        expect_vec.push(ValidateTypes::Property(property_map));

        let data = serde_yaml::from_str(yaml).unwrap();
        let result = Validate::new(&data);
        let expected = Ok(Validate {
            _task_types: TaskTypes::VALIDATE,
            name: "validate".to_string(),
            element: Element {
                element_type: ElementType::XPATH,
                value: "//*[@id=\"search-form\"]/fieldset/button".to_owned(),
            },
            expects: expect_vec,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_expect_css_property() {
        let yaml = "  
        name: 'validate'
        validate:
          element:
            xPath: '//*[@id=\"search-form\"]/fieldset/button'
          expect:
            css:
              color: 'rgba(0, 0, 0, 0)'
              'text-indent': '-10000px'
            property:
              name: 'foo'  
            text: ''
            innerHtml: ''      
              ";

        let mut expect_vec: Vec<ValidateTypes> = Vec::new();

        let mut css_map: HashMap<String, String> = HashMap::new();
        css_map.insert("color".to_string(), "rgba(0, 0, 0, 0)".to_string());
        css_map.insert("text-indent".to_string(), "-10000px".to_string());

        let mut property_map: HashMap<String, String> = HashMap::new();
        property_map.insert("name".to_string(), "foo".to_string());

        expect_vec.push(ValidateTypes::Css(css_map));
        expect_vec.push(ValidateTypes::Property(property_map));

        let data = serde_yaml::from_str(yaml).unwrap();
        let result = Validate::new(&data);
        let expected = Ok(Validate {
            _task_types: TaskTypes::VALIDATE,
            name: "validate".to_string(),
            element: Element {
                element_type: ElementType::XPATH,
                value: "//*[@id=\"search-form\"]/fieldset/button".to_owned(),
            },
            expects: expect_vec,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_expect_property() {
        let yaml = "  
        name: 'validate'
        validate:
          element:
            xPath: '//*[@id=\"search-form\"]/fieldset/button'
          expect:
            property:
              name: 'foo'  
            text: ''
            innerHtml: ''      
              ";

        let mut expect_vec: Vec<ValidateTypes> = Vec::new();

        let mut property_map: HashMap<String, String> = HashMap::new();
        property_map.insert("name".to_string(), "foo".to_string());
        expect_vec.push(ValidateTypes::Property(property_map));

        let data = serde_yaml::from_str(yaml).unwrap();
        let result = Validate::new(&data);
        let expected = Ok(Validate {
            _task_types: TaskTypes::VALIDATE,
            name: "validate".to_string(),
            element: Element {
                element_type: ElementType::XPATH,
                value: "//*[@id=\"search-form\"]/fieldset/button".to_owned(),
            },
            expects: expect_vec,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_expect_css() {
        let yaml = "  
        name: 'validate'
        validate:
          element:
            xPath: '//*[@id=\"search-form\"]/fieldset/button'
          expect:
            css:
              color: 'rgba(0, 0, 0, 0)'
              'text-indent': '-10000px'
            text: ''
            innerHtml: ''      
              ";

        let mut expect_vec: Vec<ValidateTypes> = Vec::new();

        let mut css_map: HashMap<String, String> = HashMap::new();
        css_map.insert("color".to_string(), "rgba(0, 0, 0, 0)".to_string());
        css_map.insert("text-indent".to_string(), "-10000px".to_string());

        expect_vec.push(ValidateTypes::Css(css_map));

        let data = serde_yaml::from_str(yaml).unwrap();
        let result = Validate::new(&data);
        let expected = Ok(Validate {
            _task_types: TaskTypes::VALIDATE,
            name: "validate".to_string(),
            element: Element {
                element_type: ElementType::XPATH,
                value: "//*[@id=\"search-form\"]/fieldset/button".to_owned(),
            },
            expects: expect_vec,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_expect_css_empty_property() {
        let yaml = "  
        name: 'validate'
        validate:
          element:
            xPath: '//*[@id=\"search-form\"]/fieldset/button'
          expect:
            css:
              color: 'rgba(0, 0, 0, 0)'
              'text-indent': '-10000px'
            property: {}  
            text: ''
            innerHtml: ''      
              ";

        let mut expect_vec: Vec<ValidateTypes> = Vec::new();

        let mut css_map: HashMap<String, String> = HashMap::new();
        css_map.insert("color".to_string(), "rgba(0, 0, 0, 0)".to_string());
        css_map.insert("text-indent".to_string(), "-10000px".to_string());

        expect_vec.push(ValidateTypes::Css(css_map));

        let data = serde_yaml::from_str(yaml).unwrap();
        let result = Validate::new(&data);
        let expected = Ok(Validate {
            _task_types: TaskTypes::VALIDATE,
            name: "validate".to_string(),
            element: Element {
                element_type: ElementType::XPATH,
                value: "//*[@id=\"search-form\"]/fieldset/button".to_owned(),
            },
            expects: expect_vec,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_expect_property_empty_css() {
        let yaml = "  
        name: 'validate'
        validate:
          element:
            xPath: '//*[@id=\"search-form\"]/fieldset/button'
          expect:
            property:
              name: 'foo'  
            css: {}  
            text: ''
            innerHtml: ''      
              ";

        let mut expect_vec: Vec<ValidateTypes> = Vec::new();

        let mut property_map: HashMap<String, String> = HashMap::new();
        property_map.insert("name".to_string(), "foo".to_string());
        expect_vec.push(ValidateTypes::Property(property_map));

        let data = serde_yaml::from_str(yaml).unwrap();
        let result = Validate::new(&data);
        let expected = Ok(Validate {
            _task_types: TaskTypes::VALIDATE,
            name: "validate".to_string(),
            element: Element {
                element_type: ElementType::XPATH,
                value: "//*[@id=\"search-form\"]/fieldset/button".to_owned(),
            },
            expects: expect_vec,
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_expect_property_not_map() {
        let yaml = "  
        name: 'validate'
        validate:
          element:
            xPath: '//*[@id=\"search-form\"]/fieldset/button'
          expect:
            property: 2    
              ";

        let mut expect_vec: Vec<ValidateTypes> = Vec::new();

        let mut property_map: HashMap<String, String> = HashMap::new();
        property_map.insert("name".to_string(), "foo".to_string());
        expect_vec.push(ValidateTypes::Property(property_map));

        let data = serde_yaml::from_str(yaml).unwrap();
        let result = Validate::new(&data);
        let expected = Err(TaskErr {
            message: String::from("property value is not a map"),
            task: Some(data),
            task_type: Some(TaskTypes::VALIDATE),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_expect_css_not_map() {
        let yaml = "  
        name: 'validate'
        validate:
          element:
            xPath: '//*[@id=\"search-form\"]/fieldset/button'
          expect:
            css: 2    
              ";

        let mut expect_vec: Vec<ValidateTypes> = Vec::new();

        let mut property_map: HashMap<String, String> = HashMap::new();
        property_map.insert("name".to_string(), "foo".to_string());
        expect_vec.push(ValidateTypes::Property(property_map));

        let data = serde_yaml::from_str(yaml).unwrap();
        let result = Validate::new(&data);
        let expected = Err(TaskErr {
            message: String::from("css value is not a map"),
            task: Some(data),
            task_type: Some(TaskTypes::VALIDATE),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_expect_css_key_not_str() {
        let yaml = "  
        name: 'validate'
        validate:
          element:
            xPath: '//*[@id=\"search-form\"]/fieldset/button'
          expect:
            css: 
              2 : 'foo'
              ";

        let mut expect_vec: Vec<ValidateTypes> = Vec::new();

        let mut property_map: HashMap<String, String> = HashMap::new();
        property_map.insert("name".to_string(), "foo".to_string());
        expect_vec.push(ValidateTypes::Property(property_map));

        let data = serde_yaml::from_str(yaml).unwrap();
        let result = Validate::new(&data);
        let expected = Err(TaskErr {
            message: String::from("Key: Number(2) is not a string"),
            task: Some(data),
            task_type: Some(TaskTypes::VALIDATE),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_expect_css_value_not_str() {
        let yaml = "  
        name: 'validate'
        validate:
          element:
            xPath: '//*[@id=\"search-form\"]/fieldset/button'
          expect:
            css: 
              foo : 2
              ";

        let mut expect_vec: Vec<ValidateTypes> = Vec::new();

        let mut property_map: HashMap<String, String> = HashMap::new();
        property_map.insert("name".to_string(), "foo".to_string());
        expect_vec.push(ValidateTypes::Property(property_map));

        let data = serde_yaml::from_str(yaml).unwrap();
        let result = Validate::new(&data);
        let expected = Err(TaskErr {
            message: String::from("Value: Number(2) is not a string"),
            task: Some(data),
            task_type: Some(TaskTypes::VALIDATE),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_expect_text_not_str() {
        let yaml = "  
        name: 'validate'
        validate:
          element:
            xPath: '//*[@id=\"search-form\"]/fieldset/button'
          expect:
            text: 2
              ";

        let mut expect_vec: Vec<ValidateTypes> = Vec::new();

        let mut property_map: HashMap<String, String> = HashMap::new();
        property_map.insert("name".to_string(), "foo".to_string());
        expect_vec.push(ValidateTypes::Property(property_map));

        let data = serde_yaml::from_str(yaml).unwrap();
        let result = Validate::new(&data);
        let expected = Err(TaskErr {
            message: String::from("text - value is not a string"),
            task: Some(data),
            task_type: Some(TaskTypes::VALIDATE),
        });
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_expect_inner_html_not_str() {
        let yaml = "  
        name: 'validate'
        validate:
          element:
            xPath: '//*[@id=\"search-form\"]/fieldset/button'
          expect:
            innerHtml: 2      
              ";

        let mut expect_vec: Vec<ValidateTypes> = Vec::new();

        let mut property_map: HashMap<String, String> = HashMap::new();
        property_map.insert("name".to_string(), "foo".to_string());
        expect_vec.push(ValidateTypes::Property(property_map));

        let data = serde_yaml::from_str(yaml).unwrap();
        let result = Validate::new(&data);
        let expected = Err(TaskErr {
            message: String::from("innerHtml - value is not a string"),
            task: Some(data),
            task_type: Some(TaskTypes::VALIDATE),
        });
        assert_eq!(expected, result)
    }
}
