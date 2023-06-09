use serde::{Serialize, Deserialize};
use thirtyfour::{Capabilities, DesiredCapabilities, ChromeCapabilities, WebDriver};

use crate::tasks::{to_task, Tasks, TaskOk, TaskResult, TaskErr};
use std::{path::PathBuf, str::FromStr, fs, collections::HashMap};

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

    pub async fn execute(&mut self) -> Result<&Vec<TaskOk>, String> {
        let mut web_driver: WebDriverSession = WebDriverSession::new(&self.config_path).await?;
  
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
            Err(e) => return Err(e.to_string())
        };


        Ok(WebDriverSession { driver, variables: HashMap::new() })
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
    server_url: String
}
impl DriverConfig {
    fn default() -> DriverConfig {
        DriverConfig {
            browser: String::from("firefox"),
            server_url: String::from("http://localhost:4444")
        }
    }
}

struct WebDriverConfig {
    capabilities: Capabilities,
    server_url: String
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
            server_url
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
            None => return Ok(DriverConfig::default())
        };

        if path.as_os_str().is_empty() {
            return Ok(DriverConfig::default())
        }

        let yaml = match fs::read_to_string(path) {
            Ok(data) => data,
            Err(e) => {
                println!("{:#?}", e.to_string());
                return Ok(DriverConfig::default())
            }
        };
        match serde_yaml::from_str(&yaml) {
            Ok(data) => Ok(data),
            Err(_) => Err(String::from("Unable to deserialize file")),
        }
    }
}

