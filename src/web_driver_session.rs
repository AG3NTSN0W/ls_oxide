use serde::{Deserialize, Serialize};
use thirtyfour::{Capabilities, ChromeCapabilities, DesiredCapabilities, WebDriver};

use std::{
    collections::HashMap,
    fs,
    path::{self, PathBuf},
    str::FromStr,
};

use crate::args::Args;

#[derive(Clone)]
pub struct WebDriverSession {
    pub driver: WebDriver,
    pub variables: HashMap<String, String>,
}

impl WebDriverSession {
    pub async fn new(web_driver_config: WebDriverConfig) -> Result<WebDriverSession, String> {
        let driver = match WebDriver::new(
            &web_driver_config.server_url,
            web_driver_config.capabilities,
        )
        .await
        {
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
    browser: Option<String>,
    server_url: Option<String>,
    webdriver_path: Option<String>,
}
impl DriverConfig {
    fn default() -> DriverConfig {
        DriverConfig {
            browser: Some(String::from("firefox")),
            server_url: Some(String::from("http://localhost:4444")),
            webdriver_path: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WebDriverConfig {
    capabilities: Capabilities,
    server_url: String,
    pub webdriver_path: Option<String>,
}

impl Default for WebDriverConfig {
    fn default() -> WebDriverConfig {
        WebDriverConfig {
            capabilities: Capabilities::from(DesiredCapabilities::firefox()),
            server_url: DriverConfig::default().server_url.unwrap(),
            webdriver_path: None,
        }
    }
}

impl WebDriverConfig {
    pub fn new(args: &Args) -> Result<WebDriverConfig, String> {
        if let Ok(config) = Self::get_config(&args.config_path) {
            let capabilities = config.capabilities;
            let server_url = config.server_url;
            let webdriver_path = config.webdriver_path;

            return Ok(WebDriverConfig {
                capabilities,
                server_url,
                webdriver_path,
            });
        }

        let default = WebDriverConfig::default();

        let browser = match &args.browser {
            Some(browser) => browser,
            None => &Browser::FIREFOX,
        };

        let capabilities = match browser {
            Browser::CHROME => Capabilities::from(Self::get_google_capabilities()),
            Browser::FIREFOX => Capabilities::from(DesiredCapabilities::firefox()),
        };

        let server_url = match &args.server_url {
            Some(url) => url,
            None => &default.server_url,
        };

        Ok(WebDriverConfig {
            capabilities,
            server_url: server_url.to_string(),
            webdriver_path: args.webdriver_path.clone(),
        })
    }

    fn get_google_capabilities() -> ChromeCapabilities {
        let mut capabilities = DesiredCapabilities::chrome();
        capabilities.add_chrome_arg("--enable-automation").unwrap();
        capabilities
    }

    fn get_config(config_path: &Option<PathBuf>) -> Result<WebDriverConfig, String> {
        if let Some(path) = config_path {
            if path.as_os_str().is_empty() {
                return Ok(WebDriverConfig::default());
            }

            let yaml = match fs::read_to_string(path) {
                Ok(data) => data,
                Err(e) => {
                    println!("{:#?}", e.to_string());
                    return Ok(WebDriverConfig::default());
                }
            };

            let driver_config: DriverConfig = match serde_yaml::from_str(&yaml) {
                Ok(data) => data,
                Err(_) => DriverConfig::default(),
            };

            let browser = Browser::from_str(&driver_config.browser.unwrap_or_default())?;

            let capabilities = match browser {
                Browser::CHROME => Capabilities::from(Self::get_google_capabilities()),
                Browser::FIREFOX => Capabilities::from(DesiredCapabilities::firefox()),
            };

            let server_url = driver_config.server_url.unwrap_or_default();
            let webdriver_path = driver_config.webdriver_path.unwrap_or_default();

            return Ok(WebDriverConfig {
                capabilities,
                server_url,
                webdriver_path: Some(webdriver_path),
            });
        }

        Err("config path not provided".to_string())
    }
}
