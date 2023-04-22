use serde_yaml::{Mapping, Value};
use std::str::FromStr;
use thirtyfour::By;

type ElementValue<'a> = (&'a Value, &'a Value);
type ElementStr<'a> = (&'a str, &'a str);

#[derive(Debug, PartialEq, Eq)]
pub struct Element {
    pub element_type: ElementType,
    pub value: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ElementType {
    ID,
    XPATH,
    CLASSNAME,
}

impl FromStr for ElementType {
    type Err = String;

    fn from_str(input: &str) -> Result<ElementType, Self::Err> {
        match input {
            "id" => Ok(ElementType::ID),
            "xPath" => Ok(ElementType::XPATH),
            "className" => Ok(ElementType::CLASSNAME),
            _ => Err(format!("Unknow Element Type: {:#?}", input)),
        }
    }
}

impl Element {
    pub fn new(task: &Mapping) -> Result<Self, String> {
        let element = Self::get_element(task)?;

        if element.len() == 1 {
            let element: ElementValue = element.iter().last().unwrap();
            let (element_key, element_value) = Self::get_element_value(element)?;
            return Ok(Element {
                element_type: ElementType::from_str(element_key)?,
                value: String::from(element_value),
            });
        }

        Err(format!("Multiple elements are not supported"))
    }

    fn get_element(task: &Mapping) -> Result<Mapping, String> {
        let elemnet = match task.get("element") {
            Some(x) => x.as_mapping(),
            None => return Err(format!("No element found")),
        };

        match elemnet {
            Some(e) => Ok(e.to_owned()),
            None => Err(format!("Invalid element structure")),
        }
    }

    fn get_element_value(element: ElementValue) -> Result<ElementStr, String> {
        let element_value = match element.1.as_str() {
            Some(v) => v,
            None => return Err(format!("Element: Value is not a string")),
        };

        if element_value.is_empty() {
            return Err(format!("Element: Value can`t be empty"));
        }

        let element_key = match element.0.as_str() {
            Some(v) => v,
            None => return Err(format!("Element: Key is not a string")),
        };

        if element_key.is_empty() {
            return Err(format!("Element: Key can`t be empty"));
        }
        Ok((element_key, element_value))
    }

    pub fn find_by(element: &Element) -> By {
        match element.element_type {
            ElementType::CLASSNAME => By::ClassName(&element.value),
            ElementType::ID => By::Id(&element.value),
            ElementType::XPATH => By::XPath(&element.value),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::element::{Element, ElementType};
    use serde_yaml::{Mapping, Value};

    #[test]
    fn test_invalid_element() {
        let mut element_value: Mapping = Mapping::new();
        element_value.insert(
            Value::from("foo"),
            Value::from("//*[@id=\"search-form\"]/fieldset/button"),
        );

        let mut element: Mapping = Mapping::new();
        element.insert(
            Value::from("element".to_string()),
            Value::from(element_value),
        );

        let result = Element::new(&element).map_err(|e| e);
        let expected = Err(String::from("Unknow Element Type: \"foo\""));

        assert_eq!(expected, result)
    }

    #[test]
    fn test_empty_element_value() {
        let mut element_value: Mapping = Mapping::new();
        element_value.insert(Value::from("xPath"), Value::from(""));

        let mut element: Mapping = Mapping::new();
        element.insert(
            Value::from("element".to_string()),
            Value::from(element_value),
        );

        let result = Element::new(&element).map_err(|e| e);
        let expected = Err(String::from("Element: Value can`t be empty"));

        assert_eq!(expected, result)
    }

    #[test]
    fn test_invalid_element_value() {
        let mut element_value: Mapping = Mapping::new();
        element_value.insert(Value::from("xPath"), Value::from(5));

        let mut element: Mapping = Mapping::new();
        element.insert(Value::from("element"), Value::from(element_value));

        let result = Element::new(&element).map_err(|e| e);
        let expected = Err(String::from("Element: Value is not a string"));

        assert_eq!(expected, result)
    }

    #[test]
    fn test_empty_element_key() {
        let mut element_value: Mapping = Mapping::new();
        element_value.insert(
            Value::from(""),
            Value::from("//*[@id=\"search-form\"]/fieldset/button"),
        );

        let mut element: Mapping = Mapping::new();
        element.insert(Value::from("element"), Value::from(element_value));

        let result = Element::new(&element).map_err(|e| e);
        let expected = Err(String::from("Element: Key can`t be empty"));

        assert_eq!(expected, result)
    }

    #[test]
    fn test_invalid_element_key() {
        let mut element_value: Mapping = Mapping::new();
        element_value.insert(
            Value::from(2),
            Value::from("//*[@id=\"search-form\"]/fieldset/button"),
        );

        let mut element: Mapping = Mapping::new();
        element.insert(
            Value::from("element".to_string()),
            Value::from(element_value),
        );

        let result = Element::new(&element).map_err(|e| e);
        let expected = Err(String::from("Element: Key is not a string"));

        assert_eq!(expected, result)
    }

    #[test]
    fn test_valid_element() {
        let mut element_value: Mapping = Mapping::new();
        element_value.insert(
            Value::from("xPath"),
            Value::from("//*[@id=\"search-form\"]/fieldset/button"),
        );

        let mut element: Mapping = Mapping::new();
        element.insert(
            Value::from("element".to_string()),
            Value::from(element_value),
        );

        let result = Element::new(&element).map_err(|e| e);
        let expected = Ok(Element {
            element_type: ElementType::XPATH,
            value: "//*[@id=\"search-form\"]/fieldset/button".to_owned(),
        });

        assert_eq!(expected, result)
    }
}
