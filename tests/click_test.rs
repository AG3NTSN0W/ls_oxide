mod common;

use ls_oxide::tasks::{TaskErr, TaskTypes};

mod click {

    use serial_test::serial;

    use super::*;

    #[tokio::test]
    async fn test_click_malformed() {
        let data = "
        meta_data: {}
        tasks:
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
        
          - name: 'Click search button'
            click: 'foo'
        ";

        let yaml = "  
            name: 'Click search button'            
            click: 'foo'
        ";

        let click = serde_yaml::from_str(yaml).unwrap();

        let expected: TaskErr =
            TaskErr::new(String::from("Task data is Malformed"), None, Some(click));

        executor_err!(data, expected);
    }

    #[tokio::test]
    async fn test_click_invalid_element() {
        let data = "
        meta_data: {}
        tasks:
        
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
        
          - name: 'Click search button'
            click:
              element:
                name: '//*[@id=\"search-form\"]/fieldset/button'

        ";

        let yaml = "
            name: 'Click search button'
            click:
                element:
                    name: '//*[@id=\"search-form\"]/fieldset/button'
        ";

        let click = serde_yaml::from_str(yaml).unwrap();

        let expected: TaskErr = TaskErr::new(
            String::from("Unknow Element Type: \"name\""),
            Some(TaskTypes::CLICK),
            Some(click),
        );

        executor_err!(data, expected);
    }

    #[tokio::test]
    async fn test_click_missing_element() {
        let data = "
        meta_data: {}
        tasks:
        
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
        
          - name: 'Click search button'
            click:
              foo: 'bar'
        ";

        let yaml = "
            name: 'Click search button'
            click:
              foo: 'bar'
        ";

        let click = serde_yaml::from_str(yaml).unwrap();

        let expected: TaskErr = TaskErr::new(
          String::from("No element found"),
            Some(TaskTypes::CLICK),
            Some(click),
        );

        executor_err!(data, expected);
    }

    #[tokio::test]
    async fn test_click_element_no_str() {
      let data = "
      meta_data: {}
      tasks:
      
        - name: 'Open wikipedia'
          link:
            url: 'https://wikipedia.org'
      
        - name: 'Click search button'
          click:
            element:
              id: true

      ";

        let yaml = "
        name: 'Click search button'
        click:
            element:
                id: true
        ";

        let click = serde_yaml::from_str(yaml).unwrap();

        let expected: TaskErr = TaskErr::new(
            String::from("Element: Value is not a string"),
            Some(TaskTypes::CLICK),
            Some(click),
        );

        executor_err!(data, expected);
    }

    #[tokio::test]
    async fn test_click_name_not_str() {

        let data = "
        meta_data: {}
        tasks:
        
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
        
          - name: 42
            click:
              element:
                xPath: '//*[@id=\"search-form\"]/fieldset/button'

        ";

        let yaml = "
        name: 42
        click:
            element:
                xPath: '//*[@id=\"search-form\"]/fieldset/button'
        ";

        let click = serde_yaml::from_str(yaml).unwrap();

        let expected: TaskErr = TaskErr::new(
            String::from("Task name is not a string"),
            None,
            Some(click),
        );

        executor_err!(data, expected);
    }

    #[tokio::test]
    async fn test_click_name_empty() {

        let data = "
        meta_data: {}
        tasks:
        
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
        
          - name: ''
            click:
              element:
                xPath: '//*[@id=\"search-form\"]/fieldset/button'
        ";

        let yaml = "
        name: ''
        click:
            element:
                xPath: '//*[@id=\"search-form\"]/fieldset/button'
        ";

        let click = serde_yaml::from_str(yaml).unwrap();

        let expected: TaskErr = TaskErr::new(
            String::from("Task name can`t be empty"),
            None,
            Some(click),
        );

        executor_err!(data, expected);
    }

    #[tokio::test]
    #[serial]
    async fn test_click_with_var() {
        let data = "
        meta_data: {}
        tasks:
        
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'

          - name: 'set vars'
            set_vars:
              xPath: '//*[@id=\"search-form\"]/fieldset/button'
        
          - name: 'Click search button'
            click:
              element:
                xPath: '{xPath}'

          - name: 'Validate Title'
            validate:
              element:
                id: 'firstHeading' 
              expect:
                text: 'Search'      
        ";

        let result = executor_ok!(data);

        assert_eq!(result.len(), 5);

        let task = result.get(0).unwrap();
        assert_eq!(task.name, "Open wikipedia");
        assert_eq!(task.task_type, TaskTypes::LINK);
        assert_eq!(task.result, None);

        let task = result.get(1).unwrap();
        assert_eq!(task.name, "set vars");
        assert_eq!(task.task_type, TaskTypes::SETVARIABLE);
        assert_eq!(task.result, None);

        let task = result.get(2).unwrap();
        assert_eq!(task.name, "Click search button");
        assert_eq!(task.task_type, TaskTypes::CLICK);
        assert_eq!(task.result, None);

        let task = result.get(3).unwrap();
        assert_eq!(task.name, "Validate Title");
        assert_eq!(task.task_type, TaskTypes::VALIDATE);
        common::validate_first_result(task, "Pass: Text is Search");
        
        let task = result.get(4).unwrap();
        assert_eq!(task.name, "closing web driver session");
        assert_eq!(task.task_type, TaskTypes::CLOSE);
        assert_eq!(task.result, None);
    }

    #[tokio::test]
    #[serial]
    async fn test_click() {
        let data = "
        meta_data: {}
        tasks:
        
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
        
          - name: 'Click search button'
            click:
              element:
                xPath: '//*[@id=\"search-form\"]/fieldset/button'

          - name: 'Validate Title'
            validate:
              element:
                id: 'firstHeading' 
              expect:
                text: 'Search'                    
        ";

        let result = executor_ok!(data);

        assert_eq!(result.len(), 4);

        let task = result.get(0).unwrap();
        assert_eq!(task.name, "Open wikipedia");
        assert_eq!(task.task_type, TaskTypes::LINK);
        assert_eq!(task.result, None);

        let task = result.get(1).unwrap();
        assert_eq!(task.name, "Click search button");
        assert_eq!(task.task_type, TaskTypes::CLICK);
        assert_eq!(task.result, None);

        let task = result.get(2).unwrap();
        assert_eq!(task.name, "Validate Title");
        assert_eq!(task.task_type, TaskTypes::VALIDATE);
        common::validate_first_result(task, "Pass: Text is Search");

        let task = result.get(3).unwrap();

        assert_eq!(task.name, "closing web driver session");
        assert_eq!(task.task_type, TaskTypes::CLOSE);
        assert_eq!(task.result, None);
    }

}
