use std::fmt;

use crate::tasks::{TaskTypes};

use super::task_results::TaskResults;


#[derive(Debug, Clone)]
pub struct TaskOk {
    pub name: String,
    pub task_type: TaskTypes,
    pub duration: u64,
    pub result: Option<Vec<TaskResults>>,
}

impl fmt::Display for TaskOk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?}: {} [{:#?}s]\n{:#?}",
            self.task_type, self.name, self.duration, self.result
        )
    }
}
