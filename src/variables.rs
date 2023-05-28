use lazy_static::lazy_static;
use std::collections::HashMap;


use regex::Regex;

pub fn resolve_variables(text: &str, vars: &HashMap<String, String>) -> String {
    lazy_static! {
        static ref RE_STRING: Regex = Regex::new(r"\{([[:alpha:]_]*)\}").unwrap();
        static ref RE_DEFAULT: Regex = Regex::new(r"\{([[:alpha:]_]*)\|(.*)\}").unwrap();
    }

    let mut resolved_text: String = String::from(text);
    for cap in RE_STRING.captures_iter(text) {
        if let Some(value) = vars.get(&cap[1]) {
            resolved_text = resolved_text.replace(&cap[0], value);
        }
    }

    for cap in RE_DEFAULT.captures_iter(text) {
        if let Some(value) = vars.get(&cap[1]) {
            resolved_text = resolved_text.replace(&cap[0], value);
        } else {
            resolved_text = resolved_text.replace(&cap[0], &cap[2]);
        }
    }

    resolved_text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_variables() {
        let text: &str = "Hi, {name} {surname} \nHow are you {name}";

        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("name".to_string(), "Foo".to_string());
        vars.insert("surname".to_string(), "Bar".to_string());

        let result = resolve_variables(text, &vars);
        let expected = "Hi, Foo Bar \nHow are you Foo".to_string();

        assert_eq!(expected, result)
    }

    #[test]
    fn test_resolve_variables_no_vars() {
        let text: &str = "Hi, {name} {surname} \nHow are you {name}";

        let result = resolve_variables(text, &HashMap::new());

        assert_eq!(text, result)
    }

    #[test]
    fn test_resolve_variables_missing_var() {
        let text: &str = "Hi, {name} {surname} \nHow are you {name}";

        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("name".to_string(), "Foo".to_string());

        let result = resolve_variables(text, &vars);
        let expected = "Hi, Foo {surname} \nHow are you Foo".to_string();

        assert_eq!(expected, result)
    }


    #[test]
    fn test_resolve_variables_with_underscore() {
        let text: &str = "Hi, {first_name}";

        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("first_name".to_string(), "Foo".to_string());

        let result = resolve_variables(text, &vars);
        let expected = "Hi, Foo".to_string();

        assert_eq!(expected, result)
    }

    #[test]
    fn test_resolve_variables_with_default() {
        let text: &str = "Hi, {first_name|Foo}";

        let vars: HashMap<String, String> = HashMap::new();

        let result = resolve_variables(text, &vars);
        let expected = "Hi, Foo".to_string();

        assert_eq!(expected, result)
    }

    #[test]
    fn test_resolve_variables_with_multiple_default() {
        let text: &str = "Hi, {first_name|Foo} {surname}\nHow are you {first_name|Foo}\nNo default {first_name}\n{age}";

        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("surname".to_string(), "Bar".to_string());

        let result = resolve_variables(text, &vars);
        let expected = "Hi, Foo Bar\nHow are you Foo\nNo default {first_name}\n{age}".to_string();

        assert_eq!(expected, result)
    }
    
}
