mod common;

mod integration_test {

    use ls_oxide::tasks::TaskTypes;
    use serial_test::serial;

    use super::*;

    #[tokio::test]
    #[serial]
    async fn test() {
        let data = "
        meta_data: {}
        tasks:

          - name: 'Open wikipedia'
            link:
              url: 'https://wikipedia.org'
        
          - name: 'set vars'
            set_vars:
              input: 'Rust'
              enterKey: '\u{E007}'
            
          - name: 'enter rust in search'
            send_key:
              input: '{input}'
              element:
                xPath: '//*[@id=\"searchInput\"]'

          - name: 'Validate search input'
            validate:
              element:
                xPath: '//*[@id=\"searchInput\"]'
              expect:
                property:
                  value: '{input}'              
        
          - name: 'Click search button'
            click:
              element:
                xPath: '//*[@id=\"search-form\"]/fieldset/button'
        
          - name: 'Validate Title'
            validate:
              element:
                xPath: '//*[@id=\"firstHeading\"]/span' 
              expect:
                text: '{input}'
        
          - name: 'search for rust (programming language)'
            send_key:
              input: '{input} (programming language) {enterKey}'
              element:
                id: 'searchInput'

          - name: 'wait 1 sec'
            wait: 1000      
                   
          - name: 'Validate 2nd page Title'
            validate:
              element:
                id: 'firstHeading' 
              expect:
                text: 'Rust (programming language)'      

          - name: 'wait 1 sec'
            wait: 1000                    
        
          - name: 'closing web driver session'
            close: True
        ";

        let result = executor_ok!(data);

        assert_eq!(result.len(), 11);

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
        assert_eq!(task.name, "Validate search input");
        assert_eq!(task.task_type, TaskTypes::VALIDATE);
        common::validate_first_result(task, "Pass: property value is Rust");

        let task = result.get(4).unwrap();
        assert_eq!(task.name, "Click search button");
        assert_eq!(task.task_type, TaskTypes::CLICK);
        assert_eq!(task.result, None);

        let task = result.get(5).unwrap();
        assert_eq!(task.name, "Validate Title");
        assert_eq!(task.task_type, TaskTypes::VALIDATE);
        common::validate_first_result(task, "Pass: Text is Rust");

        let task = result.get(6).unwrap();
        assert_eq!(task.name, "search for rust (programming language)");
        assert_eq!(task.task_type, TaskTypes::SENDKEY);
        assert_eq!(task.result, None);

        let task = result.get(7).unwrap();
        assert_eq!(task.name, "wait 1 sec");
        assert_eq!(task.task_type, TaskTypes::WAIT);
        assert_eq!(task.result, None);
        assert_eq!(task.duration, 1);

        let task = result.get(8).unwrap();
        assert_eq!(task.name, "Validate 2nd page Title");
        assert_eq!(task.task_type, TaskTypes::VALIDATE);
        common::validate_first_result(task, "Pass: Text is Rust (programming language)");

        let task = result.get(10).unwrap();
        assert_eq!(task.name, "closing web driver session");
        assert_eq!(task.task_type, TaskTypes::CLOSE);
        assert_eq!(task.result, None);

    }
}
