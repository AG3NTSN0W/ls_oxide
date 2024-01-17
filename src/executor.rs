use crate::{tasks::{to_task, Tasks, TaskResult}, structs::{task_ok::TaskOk, task_err::TaskErr}, web_driver_session::WebDriverSession};
use std::path::PathBuf;

pub type ExecuteResult = std::result::Result<(WebDriverSession, TaskOk), (WebDriverSession, TaskErr)>;

pub struct Executor {
    pub results: Vec<TaskOk>,
    pub tasks: Tasks,
    pub config_path: Option<PathBuf>
}

impl Executor {
    pub fn new(task_path: PathBuf, config_path: Option<PathBuf>) -> TaskResult<Self> {
        let tasks_to_execute = to_task(task_path)?;

        Ok(Executor {
            results: vec![],
            tasks: tasks_to_execute,
            config_path
        })
    }

    pub async fn execute(&mut self, vars: Option<Vec<(String, String)>>) -> Result<&Vec<TaskOk>, String> {
        let mut web_driver: WebDriverSession = WebDriverSession::new(&self.config_path).await?;

        if let Some(vars) = vars {
            vars.iter().for_each(|(key, value)| web_driver.add_variable(key, value));
        }
  
        for task in self.tasks.iter() {
            let execute = task.execute(web_driver).await;
            match execute {
                Ok((driver, task_ok)) => {
                    web_driver = driver;
                    self.results.push(task_ok)
                }
                Err((web_driver, e)) => {
                    web_driver.driver.quit().await.unwrap();
                    println!("{e}");
                    break;
                },
            }
        }
        Ok(&self.results)
    }
}
