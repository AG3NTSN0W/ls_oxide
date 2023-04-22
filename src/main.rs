use std::path::PathBuf;
use std::process;
use ls_oxide::executor::Executor;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "Little Sister")]
#[command(version = "1.0")]
#[command(about = "Automation Tool", long_about = None)]
struct Args {
    /// Path to task file
    #[arg(short, long)]
    task_path: PathBuf,

    /// Path to config file
    #[arg(short, long)]
    config_path: Option<PathBuf>,
}
#[tokio::main]
async fn main() {

    let args = Args::parse();

    let mut executor = match Executor::new(args.task_path, args.config_path) {
        Ok(exec) => exec,
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    };


    match executor.execute().await {
        Ok(x) => println!("{:#?}", x),
        Err(x) => println!("{}", x)
    };
}
