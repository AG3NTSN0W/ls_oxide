mod common;



mod close {
    use serial_test::serial;
    use ls_oxide::{tasks::TaskTypes, structs::task_err::TaskErr};
    use super::*;

    #[tokio::test]
    async fn test_missing_task_data() {
        let data = "
        meta_data: {}
        tasks:
        
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
        
          - name: 'closing web driver session'

        ";

        let yaml = "  
                name: 'closing web driver session'
              ";

        let close = serde_yaml::from_str(yaml).unwrap();

        let expected: TaskErr =
            TaskErr::new(String::from("Task data is Malformed"), None, Some(close));

        executor_err!(data, expected);
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
            close: True

        ";

        let yaml = "  
                name: ''
                close: True
              ";
        let close = serde_yaml::from_str(yaml).unwrap();

        let expected = TaskErr::new(String::from("Task name can`t be empty"), None, Some(close));

        executor_err!(data, expected);
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
        
          - name: 'closing web driver session'
            close: True

        ";

        let result = executor_ok!(data);

        assert_eq!(result.len(), 2);

        let task = result.get(0).unwrap();
        assert_eq!(task.name, "Open wikipedia");
        assert_eq!(task.task_type, TaskTypes::LINK);
        assert_eq!(task.result, None);

        let task = result.get(1).unwrap();
        assert_eq!(task.name, "closing web driver session");
        assert_eq!(task.task_type, TaskTypes::CLOSE);
        assert_eq!(task.result, None);
    }

    #[tokio::test]
    #[serial]
    async fn test_task_string() {
        let data = "
        meta_data: {}
        tasks:
        
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
        
          - name: 'closing web driver session'
            close: 'does not matter what it is'

        ";

        let result = executor_ok!(data);

        assert_eq!(result.len(), 2);

        let task = result.get(0).unwrap();
        assert_eq!(task.name, "Open wikipedia");
        assert_eq!(task.task_type, TaskTypes::LINK);
        assert_eq!(task.result, None);

        let task = result.get(1).unwrap();
        assert_eq!(task.name, "closing web driver session");
        assert_eq!(task.task_type, TaskTypes::CLOSE);
        assert_eq!(task.result, None);
    }

    #[tokio::test]
    #[serial]
    async fn test_task_mapping() {
        let data = "
        meta_data: {}
        tasks:
        
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
        
          - name: 'closing web driver session'
            close: {}

        ";

        let result = executor_ok!(data);

        assert_eq!(result.len(), 2);

        let task = result.get(0).unwrap();
        assert_eq!(task.name, "Open wikipedia");
        assert_eq!(task.task_type, TaskTypes::LINK);
        assert_eq!(task.result, None);

        let task = result.get(1).unwrap();
        assert_eq!(task.name, "closing web driver session");
        assert_eq!(task.task_type, TaskTypes::CLOSE);
        assert_eq!(task.result, None);
    }
}
