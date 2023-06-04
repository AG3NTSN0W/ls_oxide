use std::{collections::HashMap, fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResultType {
    SUCCESS,
    FAILED,
}

impl fmt::Display for ValidationResultType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidationResultType::SUCCESS => write!(f, "success"),
            ValidationResultType::FAILED => write!(f, "failed"),
        }
    }
}

impl FromStr for ValidationResultType {
    type Err = String;

    fn from_str(input: &str) -> Result<ValidationResultType, Self::Err> {
        match input {
            "success" => Ok(ValidationResultType::SUCCESS),
            "failed" => Ok(ValidationResultType::FAILED),
            _ => Err(format!("Unknow Validation Result Type: {:#?}", input)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResultsType {
    TASk,
    VALIDATE,
    READ,
}


impl fmt::Display for ResultsType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResultsType::TASk => write!(f, "task"),
            ResultsType::VALIDATE => write!(f, "validate"),
            ResultsType::READ => write!(f, "read"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskResults {
    pub result_type: ResultsType,
    pub results: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    pub validation: ValidationResultType,
    pub message: String,
}

impl From<ValidationResult> for TaskResults {
    fn from(validation_result: ValidationResult) -> Self {
        let mut results: HashMap<String, String> = HashMap::new();

        results.insert(
            "validation".to_string(),
            validation_result.validation.to_string(),
        );
        results.insert("message".to_string(), validation_result.message);

        Self {
            result_type: ResultsType::VALIDATE,
            results,
        }
    }
}

impl From<TaskResults> for ValidationResult {
    fn from(task_result: TaskResults) -> Self {
        let result = task_result.results;
        let validation = match ValidationResultType::from_str(to_str(result.get("validation"))) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        };
        let message = to_str(result.get("message")).to_string();

        Self {
            validation,
            message,
        }
    }
}

fn to_str(value: Option<&String>) -> &String {
    match value {
        Some(v) => v,
        None => panic!("Results: Value not present"),
    }
}