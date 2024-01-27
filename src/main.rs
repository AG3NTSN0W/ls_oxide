use crate::args::Args;
use clap::Parser;
use ls_oxide::web_driver_session::{WebDriverConfig, WebDriverSession};
use ls_oxide::{
    executor::Executor, web_driver_process::WebDriverProcess, web_driver_session::Browser,
};
use std::error::Error;
use std::path::PathBuf;
use std::process;

#[tokio::main]
async fn main() {
    let args: Args = Args::parse();

    let config: WebDriverConfig = WebDriverConfig::new(&args).unwrap_or_default();

    if let Some(path_to_drive) = &config.webdriver_path {
        let _web_driver_process = match WebDriverProcess::new(path_to_drive) {
            Ok(process) => process,
            Err(error) => {
                println!("{}", error);
                process::exit(1);
            }
        };

        let web_driver_session: WebDriverSession = get_web_driver_session(config).await;

        run(args.task_path, args.vars, web_driver_session).await;
    } else {
        let web_driver_session: WebDriverSession = get_web_driver_session(config).await;

        run(args.task_path, args.vars, web_driver_session).await
    }
}

async fn get_web_driver_session(config: WebDriverConfig) -> WebDriverSession {
    match WebDriverSession::new(config).await {
        Ok(web_driver_session) => web_driver_session,
        Err(error) => {
            println!("{}", error);
            process::exit(1);
        }
    }
}

async fn run(
    path: PathBuf,
    vars: Option<Vec<(String, String)>>,
    web_driver_session: WebDriverSession,
) {
    let mut executor = match Executor::validate_tasks(path) {
        Ok(exec) => exec,
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    };

    match executor.execute(vars, web_driver_session).await {
        Ok(x) => println!("{:#?}", x),
        Err(x) => println!("{}", x),
    };
}
