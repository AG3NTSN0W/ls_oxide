use std::{fs, path::PathBuf};

use ls_oxide::{
    executor::Executor,
    structs::{task_ok::TaskOk, task_err::TaskErr, task_data::TaskData, validation_result::ValidationReultType}, web_driver_session::{WebDriverSession, WebDriverConfig},
};

pub fn resource_path_tmp() -> PathBuf {
    let mut base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    base_path.push("tests/resources/tmp");

    base_path
}

pub async fn get_executor_ok(file_name: &str) -> Vec<TaskOk> {
    let mut path: PathBuf = resource_path_tmp();
    path.push(format!("{file_name}.yml"));

    let mut executor = match Executor::validate_tasks(&path) {
        Ok(executor) => executor,
        Err(err) => {
            cleanup(file_name);
            panic!("{}", err)
        }
    };

    let web_driver_session: WebDriverSession = WebDriverSession::new(WebDriverConfig::default()).await.unwrap();

    let result = match executor.execute(&None, web_driver_session).await {
        Ok(result) => result,
        Err(err) => {
            cleanup(file_name);
            panic!("{}", err)
        }
    };

    result.to_vec()
}

#[allow(dead_code)]
pub fn get_executor_err(file_name: &str) -> TaskErr {
    let mut path: PathBuf = resource_path_tmp();
    path.push(format!("{file_name}.yml"));

    match Executor::validate_tasks(&path) {
        Err(e) => e,
        Ok(_) => {
            cleanup(file_name);
            panic!("Task should have returnd and error")
        },
    }
}

pub fn setup(file_name: &str, data: &str) {
    let mut path = resource_path_tmp();
    path.push(format!("{file_name}.yml"));

    let f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
        .expect("Couldn't open file");
    let data: TaskData = match serde_yaml::from_str::<TaskData>(&data) {
        Ok(data) => data,
        Err(err) => {
            cleanup(file_name);
            panic!("{}", err)
        }
    };
    
    match serde_yaml::to_writer(f, &data) {
        Ok(_) => return,
        Err(err) => {
            cleanup(file_name);
            panic!("{}", err)
        }
    };
}

pub fn cleanup(file_name: &str) {
    let mut path = resource_path_tmp();
    path.push(format!("{file_name}.yml"));
    fs::remove_file(&path).expect(&format!("clean_up: unable to remove file: {:#?}", &path));
}

#[allow(dead_code)]
pub fn validate_first_result(task: &TaskOk, message: &str) {
    let results = task.result.clone().unwrap();
    let first_result = results.get(0).unwrap();
    assert_eq!(ValidationReultType::SUCCESS, first_result.validation);
    assert_eq!(message, first_result.message);
}

#[allow(dead_code)]
pub fn validation_results_success(task: &TaskOk) {
    let results = task.result.clone().unwrap();
    for result in results {
        assert_eq!(ValidationReultType::SUCCESS, result.validation);
    }
}


#[macro_export]
macro_rules! local_tester {
    ($f:ident, $data:expr) => {{
        use rand::distributions::{Alphanumeric, DistString};
        let file_name: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
        common::setup(&file_name, $data);
        $f(&file_name).await;
        common::cleanup(&file_name);
    }};
}

#[macro_export]
macro_rules! executor_err {
    ($data:expr, $expected:expr) => {{
        use rand::distributions::{Alphanumeric, DistString};
        let file_name: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
        common::setup(&file_name, $data);
        let result = common::get_executor_err(&file_name);
        common::cleanup(&file_name);
        assert_eq!(result, $expected);
    }};
}

#[macro_export]
macro_rules! executor_err_message {
    ($data:expr, $expected:expr) => {{
        use rand::distributions::{Alphanumeric, DistString};
        let file_name: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
        common::setup(&file_name, $data);
        let result = common::get_executor_err(&file_name);
        assert_eq!(result.get_message(), $expected);
        common::cleanup(&file_name);
    }};
}

#[macro_export]
macro_rules! executor_ok {
    ($data:expr) => {{
        use rand::distributions::{Alphanumeric, DistString};
        let file_name: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
        common::setup(&file_name, $data);
        let result = common::get_executor_ok(&file_name).await;
        common::cleanup(&file_name);
        result
    }};
}
