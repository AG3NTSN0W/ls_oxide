use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_yaml::Value;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct TaskData {
    pub meta_data: HashMap<String, Value>,
    pub tasks: Vec<HashMap<String, Value>>,
}