use super::TaskTypes;

pub struct Cookie {
    _task_types: TaskTypes,
    name: String,
}

#[async_trait]
impl Task for Cookie {

    fn new(task: &HashMap<String, Value>) -> TaskResult<Cookie> {
        let name = get_task_name(task)?;
        if !task.contains_key(TASK_TYPE) {
            return Err(TaskErr::new(
                String::from("Malformed Task"),
                Some(TaskTypes::CLOSE),
                Some(task.clone()),
            ));
        }
        Ok(Close {
            name,
            _task_types: TaskTypes::C,
        })
    }

}