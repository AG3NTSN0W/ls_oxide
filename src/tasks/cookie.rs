use std::{collections::HashMap, time::Instant};

use async_trait::async_trait;
use serde_yaml::{Mapping, Value};
use thirtyfour::cookie::SameSite;

use crate::{
    executor::ExecuteResult, structs::{task_err::TaskErr, task_ok::TaskOk}, variables::resolve_variables, web_driver_session::WebDriverSession
};

use super::{get_task, get_task_name, Task, TaskResult, TaskTypes};

const TASK_TYPE: &str = "cookie";

#[derive(PartialEq, Eq, Debug)]
pub struct Cookie {
    _task_types: TaskTypes,
    name: String,
    domain: String,
    path: String,
    cookie_key: String,
    cookie_value: String,
}

#[async_trait]
impl Task for Cookie {
    fn new(task: &HashMap<String, Value>) -> TaskResult<Cookie> {
        let name = get_task_name(task)?;
        if !task.contains_key(TASK_TYPE) {
            return Err(TaskErr::new(
                String::from("Malformed Task"),
                Some(TaskTypes::COOKIE),
                Some(task.clone()),
            ));
        }

        let cookie_key = get_cookie_value(task, "cookie_key")?;
        let cookie_value = get_cookie_value(task, "cookie_value")?;
        let domain = get_cookie_value(task, "domain")?;
        let path = get_cookie_value(task, "path")?;

        Ok(Cookie {
            name,
            _task_types: TaskTypes::COOKIE,
            cookie_key,
            cookie_value,
            domain,
            path,
        })
    }

    async fn execute(&self, web_driver_session: &mut WebDriverSession) -> ExecuteResult {
        let start = Instant::now();
        let name = self.name.clone();

        
        let cookie_key = resolve_variables(&self.cookie_key, &web_driver_session.variables);
        let cookie_value = resolve_variables(&self.cookie_value, &web_driver_session.variables);
        let domain = resolve_variables(&self.domain, &web_driver_session.variables);
        let path = resolve_variables(&self.path, &web_driver_session.variables);

        let mut cookie = thirtyfour::Cookie::new(cookie_key, cookie_value);
        cookie.set_same_site(SameSite::Lax);
        cookie.set_domain(domain);
        cookie.set_path(path);
        

        let _ = match web_driver_session.driver.add_cookie(cookie.clone()).await {
            Ok(n) => n,
            Err(e) => {
                return Err(TaskErr::new(
                    format!("Unable to add cookie: {}", e),
                    Some(TaskTypes::COOKIE),
                    None,
                ))
            }
        };

        return Ok(TaskOk {
            name,
            task_type: TaskTypes::COOKIE,
            duration: start.elapsed().as_secs(),
            result: None,
        });
    }
}

fn get_cookie_value(task: &HashMap<String, Value>, key: &str) -> TaskResult<String> {
    let cookie_task: &Mapping = get_task(task, TASK_TYPE)?;
    let value = match cookie_task.get(key) {
        Some(url) => url,
        None => {
            return Err(TaskErr::new(
                format!("{key} field not found"),
                Some(TaskTypes::COOKIE),
                Some(task.clone()),
            ))
        }
    };
    let value = match value.as_str() {
        Some(url) => String::from(url),
        None => {
            return Err(TaskErr::new(
                format!("{key} is not a string"),
                Some(TaskTypes::COOKIE),
                Some(task.clone()),
            ))
        }
    };

    if value.is_empty() {
        return Err(TaskErr::new(
            format!("{key} is empty"),
            Some(TaskTypes::COOKIE),
            Some(task.clone()),
        ));
    }
    Ok(value)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_task() {
        let cookie = HashMap::new();
        let result = Cookie::new(&cookie);
        let expected = Err(TaskErr::new(
            String::from("Malformed Task"),
            None,
            Some(cookie),
        ));
        assert_eq!(expected, result)
    }

    #[test]
    fn test_missing_task_data() {
        let yaml = "  
                name: 'Add cookie'
              ";

        let cookie = serde_yaml::from_str(yaml).unwrap();
        let result = Cookie::new(&cookie);
        let expected = Err(TaskErr::new(
            String::from("Malformed Task"),
            Some(TaskTypes::COOKIE),
            Some(cookie),
        ));
        assert_eq!(expected, result)
    }

    #[test]
    fn test_missing_task_key() {
        let yaml = "  
                name: 'Add cookie'
                cookie: 
                    value: 'fuu'
              ";

        let cookie = serde_yaml::from_str(yaml).unwrap();
        let result = Cookie::new(&cookie);
        let expected = Err(TaskErr::new(
            String::from("cookie_key field not found"),
            Some(TaskTypes::COOKIE),
            Some(cookie),
        ));
        assert_eq!(expected, result)
    }

    #[test]
    fn test_missing_task_value() {
        let yaml = "  
                name: 'Add cookie'
                cookie: 
                    cookie_key: 'foo'
              ";

        let cookie = serde_yaml::from_str(yaml).unwrap();
        let result = Cookie::new(&cookie);
        let expected = Err(TaskErr::new(
            String::from("cookie_value field not found"),
            Some(TaskTypes::COOKIE),
            Some(cookie),
        ));
        assert_eq!(expected, result)
    }

    #[test]
    fn test_task_success() {
        let yaml = "  
                name: 'Add cookie'
                cookie: 
                    cookie_key: 'foo'
                    cookie_value: 'bar'
                    path: 'bar/bar'
                    domain: 'boo.bar'
              ";

        let cookie = serde_yaml::from_str(yaml).unwrap();
        let result = Cookie::new(&cookie);
        let expected = Ok(Cookie {
            _task_types: TaskTypes::COOKIE,
            name: "Add cookie".to_owned(),
            cookie_key: "foo".to_owned(),
            cookie_value: "bar".to_owned(),
            domain: "boo.bar".to_owned(),
            path: "bar/bar".to_owned()
        });
        assert_eq!(expected, result)
    }
}