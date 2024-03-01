use std::{fmt, path::PathBuf};

use super::validation_result::{ValidationResult, ValidationReultType};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TaskSuite<'a> {
    directory: String,
    path: &'a PathBuf,
    file_name: String,
    results: Option<TaskSuiteResult>,
    error: Option<String>,
    duration: u64,
}

impl<'a> TaskSuite<'a> {
    pub fn new(path: &PathBuf) -> TaskSuite {
        let directory: String = match path.parent() {
            Some(s) => s.to_str().unwrap().to_owned(),
            None => "".to_string(),
        };

        let file_name: String = match path.file_name() {
            Some(s) => s.to_str().unwrap().to_owned(),
            None => "".to_string(),
        };

        TaskSuite {
            directory,
            path,
            file_name,
            results: None,
            error: None,
            duration: 0
        }
    }

    pub fn add_results(&mut self, results: Vec<ValidationResult>) {
        self.results = Some(TaskSuiteResult::new(results));
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn set_duration(&mut self, duration: u64) {
        self.duration = duration;
    }
}

impl<'a> fmt::Display for TaskSuite<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.results.is_some() {
            let r = self.results.clone().unwrap();
            write!(
                f,
                "Test results: {}, Success: {}, Failed: {}, Duration: {}\n{}",
                self.path.display(),
                r.success,
                r.failed.0,
                self.duration,
                error_results_to_string(&r.failed.1)
            )
        } else if self.error.is_some() {
            let r = self.error.clone().unwrap();
            write!(f, "Test results: {}, Reason: {}", self.path.display(), r)
        } else {
            write!(f, "Test no results found results: {}", self.path.display())
        }
    }


}
fn error_results_to_string(results: &Vec<ValidationResult>) -> String {
    results.iter().cloned().map(|x| format!(" - {}\n", x.message)).collect()
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TaskSuiteResult {
    success: usize,
    failed: (usize, Vec<ValidationResult>),
    results: Vec<ValidationResult>,
}

impl TaskSuiteResult {
    pub fn new(results: Vec<ValidationResult>) -> TaskSuiteResult {
        TaskSuiteResult {
            success: Self::get_success_count(&results),
            failed: Self::get_failed_count(&results),
            results,
        }
    }

    fn get_success_count(results: &Vec<ValidationResult>) -> usize {
        results
            .into_iter()
            .filter(|x| x.validation == ValidationReultType::SUCCESS)
            .count()
    }

    fn get_failed_count(results: &Vec<ValidationResult>) -> (usize, Vec<ValidationResult>) {
        let filter = results
            .into_iter()
            .cloned()
            .filter(|x| x.validation == ValidationReultType::FAILED);
        (filter.clone().count(), filter.collect())
    }
}
