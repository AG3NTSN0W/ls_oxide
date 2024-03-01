use clap::Parser;
use ls_oxide::cli_argument::Args;
use ls_oxide::structs::task_ok::TaskOk;
use ls_oxide::structs::task_suite::TaskSuite;
use ls_oxide::structs::validation_result::ValidationResult;
use ls_oxide::thread_pool::ThreadPool;
use ls_oxide::web_driver_session::{WebDriverConfig, WebDriverSession};
use ls_oxide::{executor::Executor, web_driver_process::WebDriverProcess};
use tokio::runtime::Runtime;
use walkdir::WalkDir;

use std::ffi::OsStr;
use std::path::PathBuf;
use std::process;
use std::time::Instant;

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
    path: &PathBuf,
    vars: &Option<Vec<(String, String)>>,
    config: WebDriverConfig,
) -> Result<Vec<TaskOk>, String> {
    let mut executor = match Executor::validate_tasks(path) {
        Ok(exec) => exec,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    let web_driver_session: WebDriverSession = get_web_driver_session(config).await;
    return match executor.execute(vars, web_driver_session).await {
        Ok(task_results) => Ok(task_results.to_vec()),
        Err(error) => Err(error),
    };
}

#[tokio::main]
async fn main() {
    let args: Args = Args::parse();

    let config: WebDriverConfig = WebDriverConfig::new(&args).unwrap_or_default();

    let _d: Option<WebDriverProcess> = if let Some(path_to_drive) = &config.webdriver_path {
        match WebDriverProcess::new(path_to_drive, &config.get_port()) {
            Ok(process) => Some(process),
            Err(error) => {
                println!("{}", error);
                process::exit(1);
            }
        }
    } else {
        None
    };

    if let Some(task_path) = &args.task_path {
        run_task(&args, config, task_path).await;
        return;
    }

    if let Some(task_suite) = &args.task_suite {
        run_dir(task_suite).await;
        return;
    }
}

async fn run_dir(task_suite: &PathBuf) {
    let pool: ThreadPool = ThreadPool::new(2);

    if task_suite.is_file() {
        println!("Task path can`t be a File: {:?}", task_suite);
        process::exit(1);
    }
    
    let walker = WalkDir::new(task_suite).into_iter();

    for entry in walker {
        let dir_entry = match entry {
            Ok(e) => e,
            Err(error) => {
                println!("{:#?}", error);
                process::exit(1);
            }
        };

        if dir_entry.path().is_file()
            && dir_entry
                .path()
                .extension()
                .unwrap_or(OsStr::new(""))
                .eq("yml")
        {
            let taks_path = dir_entry.path().to_path_buf();

            pool.execute(move || {
                let start = Instant::now();
                let rt = Runtime::new().unwrap();
                let mut task_result = TaskSuite::new(&taks_path);
                match rt.block_on(async {
                    let args: Args = Args::parse();
                    let config: WebDriverConfig = WebDriverConfig::new(&args).unwrap_or_default();
                    run(&taks_path, &args.vars, config).await
                }) {
                    Ok(task_ok) => {
                        let results: Vec<ValidationResult> = task_ok
                            .iter()
                            .cloned()
                            .map(|x| x.result)
                            .filter(|x| x.is_some())
                            .map(|x| x.unwrap())
                            .flat_map(|x| x)
                            .collect();

                            task_result.add_results(results)
                    }
                    Err(e) => task_result.set_error(e),
                };
                task_result.set_duration(start.elapsed().as_secs());
                println!("{}", task_result.to_string())
            });
        }
    }

    // let t: PathBuf = task_suite.clone();
}

async fn run_task(args: &Args, config: WebDriverConfig, task_path: &PathBuf) {
    if task_path.is_dir() {
        println!("Task path can`t be a directory: {:?}", task_path);
        process::exit(1);
    }

    match run(task_path, &args.vars, config).await {
        Ok(x) => println!("{:#?}", x),
        Err(e) => println!("{}", e),
    };
}
