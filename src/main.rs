use std::path::PathBuf;
use std::process;
use ls_oxide::executor::Executor;

use clap::Parser;
use std::error::Error;

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

    #[arg(short = 'v', value_parser = parse_key_val::<String, String>)]
    vars: Option<Vec<(String, String)>>,
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

    match executor.execute_filter(args.vars).await {
        Ok(r) => println!("{:?}", r),
        Err(x) => println!("{}", x)
    };

}
