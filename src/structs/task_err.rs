use std::{collections::HashMap, fmt};

use serde_yaml::Value;

use crate::tasks::TaskTypes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskErr {
    message: String,
    task_type: Option<TaskTypes>,
    task: Option<HashMap<String, Value>>,
}

impl TaskErr {
    pub fn new(
        message: String,
        task_type: Option<TaskTypes>,
        task: Option<HashMap<String, Value>>,
    ) -> TaskErr {
        TaskErr {
            message,
            task_type,
            task,
        }
    }


    pub fn get_message(&self) -> &str {
        &self.message
    }

    pub fn set_task(&mut self, task:Option<HashMap<String, Value>>) {
        self.task = task;
    }

 
}

impl fmt::Display for TaskErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let task = self.task.clone().unwrap_or_default();

        write!(
            f,
            "{:?}: {}\n{:#?}",
            self.task_type.unwrap_or_default(),
            self.message,
            task
        )
    }
}