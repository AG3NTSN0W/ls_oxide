#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationReultType {
    SUCCESS,
    FAILED,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct ValidationResult {
    pub validation: ValidationReultType,
    pub message: String,
}