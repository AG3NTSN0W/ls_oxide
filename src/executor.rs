use crate::{
    structs::{task_err::TaskErr, task_ok::TaskOk},
    tasks::{to_task, TaskResult, Tasks},
    web_driver_session::WebDriverSession,
};
use std::path::PathBuf;

pub type ExecuteResult = std::result::Result<TaskOk, TaskErr>;

pub struct Executor {
    pub results: Vec<TaskOk>,
    pub tasks: Tasks
}

impl Executor {
    pub fn validate_tasks(task_path: &PathBuf) -> TaskResult<Self> {
        let tasks_to_execute = to_task(task_path)?;

        Ok(Executor {
            results: vec![],
            tasks: tasks_to_execute,
        })
    }

    pub async fn execute(
        &mut self,
        vars: &Option<Vec<(String, String)>>,
        mut web_driver_session: WebDriverSession
    ) -> Result<&Vec<TaskOk>, String> {
        if let Some(vars) = vars {
            vars.iter()
                .for_each(|(key, value)| web_driver_session.add_variable(key, value));
        }

        for task in self.tasks.iter() {
            let execute = task.execute(&mut web_driver_session).await;
            match execute {
                Ok(task_ok) => self.results.push(task_ok),
                Err(e) => {
                    web_driver_session.driver.clone().quit().await.unwrap();
                    println!("{e}");
                    break;
                }
            }
        }
        Ok(&self.results)
    }
}
