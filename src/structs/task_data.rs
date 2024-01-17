use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_yaml::Value;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct TaskData {
    pub tasks: Option<Vec<HashMap<String, Value>>>,
    pub validate: Option<Vec<HashMap<String, Value>>>
}