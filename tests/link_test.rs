mod common;

use ls_oxide::tasks::{TaskErr, TaskTypes};


mod link {

    use serial_test::serial;

    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_link() {
        let data = "
        meta_data: {}
        tasks:
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'

          - name: 'Validate Title'
            validate:
              element:
                xPath: '//*[@id=\"www-wikipedia-org\"]/div[1]/h1/span' 
              expect:
                text: 'Wikipedia'          

        ";

        let result = executor_ok!(data);

        assert_eq!(result.len(), 3);

        let link_task = result.get(0).unwrap();
        assert_eq!(link_task.name, "Open wikipedia");
        assert_eq!(link_task.task_type, TaskTypes::LINK);
        assert_eq!(link_task.result, None);

        let task = result.get(1).unwrap();
        assert_eq!(task.name, "Validate Title");
        assert_eq!(task.task_type, TaskTypes::VALIDATE);
        common::validate_first_result(task, "Pass: Text is Wikipedia");

        let close_task = result.get(2).unwrap();

        assert_eq!(close_task.name, "closing web driver session");
        assert_eq!(close_task.task_type, TaskTypes::CLOSE);
        assert_eq!(close_task.result, None);
    }

    #[tokio::test]
    async fn test_first_task_link() {
        let data = "
        meta_data: {}
        tasks:
          - name: 'set vars'
            set_vars:
              input: 'Rust'
              enterKey: '\u{E007}'
              color: 'rgb(0, 0, 0)'
        ";
        let expected: TaskErr =
            TaskErr::new(String::from("First Task should be a Link"), None, None);
        executor_err!(data, expected)
    }

    #[tokio::test]
    async fn test_empty_link() {
        let data: &str = "  
        meta_data: {}
        tasks:
          - name: 'Open wikipedia'
            link:
              url: ''
        ";

        let yaml = "  
        name: 'Open wikipedia'            
        link:
            url: '' 
        ";

        let link = serde_yaml::from_str(yaml).unwrap();

        let expected: TaskErr = TaskErr::new(
            String::from("Url is empty"),
            Some(TaskTypes::LINK),
            Some(link),
        );

        executor_err!(data, expected)
    }

    #[tokio::test]
    async fn test_missing_link() {
        let data: &str = "  
        meta_data: {}
        tasks:
          - name: 'Open wikipedia'
            link:
        ";

        let yaml = "
          name: 'Open wikipedia'
          link:
        ";

        let link = serde_yaml::from_str(yaml).unwrap();
        let expected: TaskErr =
            TaskErr::new(String::from("Task data is Malformed"), None, Some(link));

        executor_err!(data, expected)
    }

    #[tokio::test]
    async fn test_link_name_not_str() {
        let data = "
        meta_data: {}
        tasks:
          - name: 42
            link:
              url: 'https://wikipedia.org'

        ";

        let yaml = "  
            name: 42
            link:
                url: 'https://wikipedia.org'  
              ";

        let link = serde_yaml::from_str(yaml).unwrap();
        let expected: TaskErr =
            TaskErr::new(String::from("Task name is not a string"), None, Some(link));

        executor_err!(data, expected)
    }

    #[tokio::test]
    async fn test_link_name_empty() {
        let data = "
        meta_data: {}
        tasks:
          - name: ''
            link:
              url: 'https://wikipedia.org'

        ";

        let yaml = "  
            name: ''
            link:
                url: 'https://wikipedia.org'  
              ";

        let link = serde_yaml::from_str(yaml).unwrap();
        let expected: TaskErr =
            TaskErr::new(String::from("Task name can`t be empty"), None, Some(link));

        executor_err!(data, expected)
    }
}
