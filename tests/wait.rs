mod common;

mod wait {
    use ls_oxide::tasks::TaskTypes;

    use super::*;

    #[tokio::test]
    async fn test_missing_task_data() {
        let data = "
        meta_data: {}
        tasks:
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
            
          - name: 'wait 5 sec'
            wait: true
        ";

        executor_err_message!(data, "Wait field is not a number");
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
            wait: 5000
        ";

        executor_err_message!(data, "Task name can`t be empty");
    }

    #[tokio::test]
    async fn test_task_success() {
        let data = "
        meta_data: {}
        tasks:
          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
            
          - name: 'wait 5 sec'
            wait: 5000
        ";

        let result = executor_ok!(data);

        let task = result.get(0).unwrap();
        assert_eq!(task.name, "Open wikipedia");
        assert_eq!(task.task_type, TaskTypes::LINK);
        assert_eq!(task.result, None);

        let task = result.get(1).unwrap();
        assert_eq!(task.name, "wait 5 sec");
        assert_eq!(task.task_type, TaskTypes::WAIT);
        assert_eq!(task.result, None);
        assert_eq!(task.duration, 5);
    }

}
