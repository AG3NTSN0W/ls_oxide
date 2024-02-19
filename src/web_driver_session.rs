use serde::{Deserialize, Serialize};
use thirtyfour::{Capabilities, ChromeCapabilities, DesiredCapabilities, WebDriver};

use std::{collections::HashMap, fmt, fs, path::PathBuf, str::FromStr};

use crate::cli_argument::Args;

#[derive(Clone)]
pub struct WebDriverSession {
    pub driver: WebDriver,
    pub variables: HashMap<String, String>,
}

impl WebDriverSession {
    pub async fn new(web_driver_config: WebDriverConfig) -> Result<WebDriverSession, String> {
        let server_url = format!(
            "{}:{}",
            &web_driver_config.server_url,
            web_driver_config.get_port()
        );

        let driver = match WebDriver::new(&server_url, web_driver_config.capabilities).await {
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
    #[default]
    CHROME,
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

impl fmt::Display for Browser {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Browser::CHROME => write!(f, "chrome"),
            Browser::FIREFOX => write!(f, "firefox"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DriverConfig {
    browser: Option<String>,
    server_url: Option<String>,
    port: Option<u32>,
    webdriver_path: Option<String>,
}
impl DriverConfig {
    fn default() -> DriverConfig {
        DriverConfig {
            browser: Some(Browser::default().to_string()),
            server_url: Some(String::from("http://localhost")),
            port: Some(9515),
            webdriver_path: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WebDriverConfig {
    capabilities: Capabilities,
    server_url: String,
    port: u32,
    pub webdriver_path: Option<String>,
}

impl Default for WebDriverConfig {
    fn default() -> WebDriverConfig {
        WebDriverConfig {
            capabilities: Capabilities::from(DesiredCapabilities::chrome()),
            server_url: DriverConfig::default().server_url.unwrap(),
            port: DriverConfig::default().port.unwrap(),
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
            let port = config.port;

            return Ok(WebDriverConfig {
                capabilities,
                server_url,
                port,
                webdriver_path,
            });
        }

        let default = WebDriverConfig::default();

        let default_browser = Browser::default();

        let browser = match &args.browser {
            Some(browser) => browser,
            None => &default_browser,
        };

        let capabilities = match browser {
            Browser::CHROME => Capabilities::from(Self::get_google_capabilities()),
            Browser::FIREFOX => Capabilities::from(DesiredCapabilities::firefox()),
        };

        let server_url = match &args.server_url {
            Some(url) => url,
            None => &default.server_url,
        };

        let port = match args.port {
            Some(port) => port,
            None => WebDriverConfig::default().port,
        };

        Ok(WebDriverConfig {
            capabilities,
            server_url: server_url.to_string(),
            port,
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

            let browse_str = &driver_config
                .browser
                .unwrap_or(Browser::default().to_string());

            let browser = Browser::from_str(browse_str).unwrap_or(Browser::default());

            let capabilities = match browser {
                Browser::CHROME => Capabilities::from(Self::get_google_capabilities()),
                Browser::FIREFOX => Capabilities::from(DesiredCapabilities::firefox()),
            };

            let server_url = driver_config
                .server_url
                .unwrap_or(WebDriverConfig::default().server_url);

            let port = driver_config
                .port
                .unwrap_or(WebDriverConfig::default().port);

            let webdriver_path: String = driver_config.webdriver_path.unwrap_or_default();

            let webdriver_path = if webdriver_path.is_empty() {
                None
            } else {
                Some(webdriver_path)
            };

            return Ok(WebDriverConfig {
                capabilities,
                server_url,
                port,
                webdriver_path: webdriver_path,
            });
        }

        Err("config path not provided".to_string())
    }

    pub fn get_port(&self) -> String {
        self.port.to_string()
    }
}
