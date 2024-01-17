mod common;
mod send_key {

    use super::*;
    use ls_oxide::tasks::TaskTypes;
    use serial_test::serial;

    #[tokio::test]
    async fn test_invalid_task_data() {
        let data = "
        meta_data: {}
        tasks:
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
        
          - name: 'enter rust in search'
            send_key: 2
        ";

        executor_err_message!(data, "Task data is Malformed");
    }

    #[tokio::test]
    async fn test_missing_task_data() {
        let data = "
        meta_data: {}
        tasks:
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
        
          - name: 'enter rust in search'
        ";

        executor_err_message!(data, "Task data is Malformed");
    }

    #[tokio::test]
    async fn test_empty_name() {
        let data = "
        meta_data: {}
        tasks:
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
    
          - name: ''
            send_key:
              input: 'Rust'
              element:
                xPath: '//*[@id=\"searchInput\"]'
        ";

        executor_err_message!(data, "Task name can`t be empty");
    }

    #[tokio::test]
    async fn test_invalid_name() {
        let data = "
        meta_data: {}
        tasks:
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
    
          - name: 42
            send_key:
              input: 'Rust'
              element:
                xPath: '//*[@id=\"searchInput\"]'
        ";

        executor_err_message!(data, "Task name is not a string");
    }

    #[tokio::test]
    async fn test_invalid_element() {
        let data = "
        meta_data: {}
        tasks:
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
    
          - name: 'enter rust in search'
            send_key:
              input: 'Rust'
              element:
                foo: '//*[@id=\"searchInput\"]'
        ";

        executor_err_message!(data, "Unknow Element Type: \"foo\"");
    }

    #[tokio::test]
    async fn test_invalid_element_value() {
        let data = "
        meta_data: {}
        tasks:
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
    
          - name: 'enter rust in search'
            send_key:
              input: 'Rust'
              element: 
                xPath: 42
        ";

        executor_err_message!(data, "Element: Value is not a string");
    }

    #[tokio::test]
    async fn test_invalid_input() {
        let data = "
        meta_data: {}
        tasks:
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
    
          - name: 'enter rust in search'
            send_key:
              input: 42
              element: 
                xPath: '//*[@id=\"searchInput\"]'
        ";

        executor_err_message!(data, "input is not a string");
    }

    #[tokio::test]
    async fn test_missing_input() {
        let data = "
        meta_data: {}
        tasks:
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
    
          - name: 'enter rust in search'
            send_key:
              element: 
                xPath: '//*[@id=\"searchInput\"]'
        ";

        executor_err_message!(data, "input field not found");
    }

    #[tokio::test]
    async fn test_missing_element() {
        let data = "
        meta_data: {}
        tasks:
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
    
          - name: 'enter rust in search'
            send_key:
              input: 'Rust'
        ";

        executor_err_message!(data, "No element found");
    }

    #[tokio::test]
    #[serial]
    async fn test_task_success() {
        let data = "
        meta_data: {}
        tasks:
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
    
          - name: 'enter rust in search'
            send_key:
              input: 'Rust'
              element:
                xPath: '//*[@id=\"searchInput\"]'

          - name: 'Validate Input'
            validate:
              element:
                xPath: '//*[@id=\"searchInput\"]'
              expect:
                property:
                  value: 'Rust'         
        ";

        let result = executor_ok!(data);

        assert_eq!(result.len(), 4);

        let task = result.get(0).unwrap();
        assert_eq!(task.name, "Open wikipedia");
        assert_eq!(task.task_type, TaskTypes::LINK);
        assert_eq!(task.result, None);


        let task = result.get(1).unwrap();
        assert_eq!(task.name, "enter rust in search");
        assert_eq!(task.task_type, TaskTypes::SENDKEY);
        assert_eq!(task.result, None);

        let task = result.get(2).unwrap();
        assert_eq!(task.name, "Validate Input");
        assert_eq!(task.task_type, TaskTypes::VALIDATE);
        common::validate_first_result(task, "Pass: property value is Rust");
    }

    #[tokio::test]
    #[serial]
    async fn test_task_with_var() {
        let data = "
        meta_data: {}
        tasks:
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'

          - name: 'set vars'
            set_vars:
              input: 'Rust'    
    
          - name: 'enter rust in search'
            send_key:
              input: '{input}'
              element:
                xPath: '//*[@id=\"searchInput\"]'

          - name: 'Validate Input'
            validate:
              element:
                xPath: '//*[@id=\"searchInput\"]'
              expect:
                property:
                  value: '{input}'        
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
        assert_eq!(task.name, "enter rust in search");
        assert_eq!(task.task_type, TaskTypes::SENDKEY);
        assert_eq!(task.result, None);

        let task = result.get(3).unwrap();
        assert_eq!(task.name, "Validate Input");
        assert_eq!(task.task_type, TaskTypes::VALIDATE);
        common::validate_first_result(task, "Pass: property value is Rust");
    }
}
