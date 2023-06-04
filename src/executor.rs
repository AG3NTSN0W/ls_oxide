use serde::{Deserialize, Serialize};
use thirtyfour::{Capabilities, ChromeCapabilities, DesiredCapabilities, WebDriver};

use crate::{
    structs::{task_err::TaskErr, task_ok::TaskOk, task_results::ResultsType},
    tasks::{to_task, TaskResult, Tasks},
};
use std::{collections::HashMap, fs, path::PathBuf, str::FromStr};

pub type ExecuteResult =
    std::result::Result<(WebDriverSession, TaskOk), (WebDriverSession, TaskErr)>;

pub struct Executor {
    pub task_type: ResultsType,
    pub results: Vec<TaskOk>,
    pub tasks: Tasks,
    pub config_path: Option<PathBuf>,
}

impl Executor {
    pub fn new(task_path: PathBuf, config_path: Option<PathBuf>) -> TaskResult<Self> {
        let (tasks_to_execut, task_type) = to_task(task_path)?;

        Ok(Executor {
            results: vec![],
            tasks: tasks_to_execut,
            config_path,
            task_type,
        })
    }

    pub async fn execute(
        &mut self,
        vars: Option<Vec<(String, String)>>,
    ) -> Result<&Vec<TaskOk>, String> {
        let mut web_driver: WebDriverSession = WebDriverSession::new(&self.config_path).await?;

        if let Some(vars) = vars {
            vars.iter()
                .for_each(|(key, value)| web_driver.add_variable(key, value));
        }

        for task in self.tasks.iter() {
            let execute = task.execute(web_driver).await;
            match execute {
                Ok((driver, task_ok)) => {
                    web_driver = driver;
                    self.results.push(task_ok);
                }
                Err((web_driver, e)) => {
                    web_driver.driver.quit().await.unwrap();
                    println!("{e}");
                    break;
                }
            }
        }

        Ok(&self.results)
    }

    pub async fn execute_filter(
        &mut self,
        vars: Option<Vec<(String, String)>>,
    ) -> Result<Vec<HashMap<String, String>>, String> {
        let task_type = self.task_type.clone();
        let results = match self.execute(vars).await {
            Ok(r) => r,
            Err(err) => return Err(err),
        };

        Ok(Executor::filter_results(task_type, results.to_vec()))
    }

    pub async fn run(
        task_path: PathBuf,
        config_path: Option<PathBuf>,
        vars: Option<Vec<(String, String)>>,
    ) -> Result<Vec<HashMap<String, String>>, String> {
        let mut executor = match Executor::new(task_path, config_path) {
            Ok(data) => data,
            Err(e) => return Err(e.to_string()),
        };

        match executor.execute_filter(vars).await {
            Ok(result) => Ok(result),
            Err(e) => Err(e),
        }
    }

    fn filter_results(
        task_type: ResultsType,
        results: Vec<TaskOk>,
    ) -> Vec<HashMap<String, String>> {
        let mut filtered_results: Vec<HashMap<String, String>> = vec![];

        if task_type == ResultsType::TASk {
            for task_results in results {
                let mut results_map: HashMap<String, String> = HashMap::new();
                results_map.insert("name".to_string(), task_results.name);
                results_map.insert("duration".to_string(), task_results.duration.to_string());
                results_map.insert("taskType".to_string(), task_results.task_type.to_string());
                filtered_results.push(results_map);
            }

            return filtered_results;
        }

        if task_type == ResultsType::VALIDATE {
            for task_results in results {
                if let Some(results) = task_results.result {
                    for result in results {
                        if result.result_type != ResultsType::VALIDATE {
                            continue;
                        }
                        filtered_results.push(result.results);
                    }
                }
            }
            return filtered_results;
        }

        vec![]
    }
}

#[derive(Clone)]
pub struct WebDriverSession {
    pub driver: WebDriver,
    pub variables: HashMap<String, String>,
}

impl WebDriverSession {
    pub async fn new(config_path: &Option<PathBuf>) -> Result<WebDriverSession, String> {
        let config = WebDriverConfig::new(config_path)?;

        let driver = match WebDriver::new(&config.server_url, config.capabilities).await {
            Ok(d) => d,
            Err(e) => return Err(e.to_string()),
        };

        Ok(WebDriverSession {
            driver,
            variables: HashMap::new(),
        })
    }

    pub fn add_variable(&mut self, key: &String, value: &String) {
        self.variables.insert(key.to_string(), value.to_string());
    }
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
pub enum Browser {
    CHROME,
    #[default]
    FIREFOX,
}

impl FromStr for Browser {
    type Err = String;

    fn from_str(input: &str) -> Result<Browser, String> {
        match input {
            "chrome" => Ok(Browser::CHROME),
            "firefox" => Ok(Browser::FIREFOX),
            _ => Err(String::from("Browser not Supported")),
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
struct DriverConfig {
    browser: String,
    server_url: String,
}
impl DriverConfig {
    fn default() -> DriverConfig {
        DriverConfig {
            browser: String::from("firefox"),
            server_url: String::from("http://localhost:4444"),
        }
    }
}

struct WebDriverConfig {
    capabilities: Capabilities,
    server_url: String,
}

impl WebDriverConfig {
    fn new(path: &Option<PathBuf>) -> Result<WebDriverConfig, String> {
        let config: DriverConfig = Self::get_config(path)?;
        let browser = Browser::from_str(&config.browser)?;
        let server_url = config.server_url;

        let capabilities = match browser {
            Browser::CHROME => Capabilities::from(Self::get_google_capabilities()),
            Browser::FIREFOX => Capabilities::from(DesiredCapabilities::firefox()),
        };

        Ok(WebDriverConfig {
            capabilities,
            server_url,
        })
    }

    fn get_google_capabilities() -> ChromeCapabilities {
        let mut capabilities = DesiredCapabilities::chrome();
        capabilities.add_chrome_arg("--enable-automation").unwrap();
        capabilities
    }

    fn get_config(config_path: &Option<PathBuf>) -> Result<DriverConfig, String> {
        let path = match config_path {
            Some(p) => p,
            None => return Ok(DriverConfig::default()),
        };

        if path.as_os_str().is_empty() {
            return Ok(DriverConfig::default());
        }

        let yaml = match fs::read_to_string(path) {
            Ok(data) => data,
            Err(e) => {
                println!("{:#?}", e.to_string());
                return Ok(DriverConfig::default());
            }
        };
        match serde_yaml::from_str(&yaml) {
            Ok(data) => Ok(data),
            Err(_) => Err(String::from("Unable to deserialize file")),
        }
    }
}
