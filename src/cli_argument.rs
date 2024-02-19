use crate::web_driver_session::Browser;
use clap::{command, Parser};
use std::{error::Error, path::PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "Little Sister")]
#[command(version = "1.0")]
#[command(about = "Automation Tool", long_about = None)]
pub struct Args {
    /// Path to task file
    #[arg(short = 't', long)]
    pub task_path: Option<PathBuf>,

    #[arg(short = 'd', long)]
    pub task_suite: Option<PathBuf>,

    /// Path to config file
    #[arg(short = 'c', long)]
    pub config_path: Option<PathBuf>,

    /// browser: 'firefox' or 'chrome' | default firefox
    #[arg(short = 'b', long)]
    pub browser: Option<Browser>,

    /// Server url: http://localhost:4444
    #[arg(short = 's', long)]
    pub server_url: Option<String>,

    /// Path to chromedriver or geckodriver
    #[arg(short = 'w', long)]
    pub webdriver_path: Option<String>,

    /// Path to chromedriver or geckodriver
    #[arg(short = 'p', long)]
    pub port: Option<u32>,

    #[arg(short = 'v', long, value_parser = parse_key_val::<String, String>)]
    pub vars: Option<Vec<(String, String)>>,
}

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}
