use kafka::consumer::*;

use manager::process::*;
use manager::{create_anti_workflow, edits_consumer_runner, normal_consumer_runner};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // BUG: Should probably use never type here
    env_logger::init();

    // Sample rollback functions
    let rollback_functions = HashMap::from([
        (
            Process::new("user".to_owned(), "create".to_owned()),
            Process::new("user".to_owned(), "delete".to_owned()),
        ),
        (
            Process::new("license".to_owned(), "add".to_owned()),
            Process::new("license".to_owned(), "remove".to_owned()),
        ),
        (
            Process::new("membership".to_owned(), "add".to_owned()),
            Process::new("membership".to_owned(), "remove".to_owned()),
        ),
        (
            Process::new("legal".to_owned(), "update".to_owned()),
            Process::new("legal".to_owned(), "revert".to_owned()),
        ),
    ]);

    let user_registration: Workflow = HashMap::from([
        (
            Process::new("user".to_owned(), "create".to_owned()),
            vec![
                Process::new("license".to_owned(), "add".to_owned()),
                Process::new("membership".to_owned(), "add".to_owned()),
            ],
        ),
        (
            Process::new("license".to_owned(), "add".to_owned()),
            vec![Process::new("legal".to_owned(), "update".to_owned())],
        ),
    ]);

    let anti_user_registration = create_anti_workflow(&user_registration);

    let workflows: Arc<Mutex<Workflows>> = Arc::new(Mutex::new(HashMap::from([(
        "user_registration".to_owned(),
        WorkflowTriple::new(
            user_registration.clone(),
            anti_user_registration,
            "v0.1.0".to_owned(),
        ),
    )])));

    let normal_consumer = Consumer::from_hosts(vec!["localhost:9092".to_owned()])
        .with_topic("tomanager".to_owned())
        .with_group("manager".to_owned())
        .with_offset_storage(Some(GroupOffsetStorage::Kafka))
        .create()?;

    let edits_consumer = Consumer::from_hosts(vec!["localhost:9092".to_owned()])
        .with_topic("tomanageredits".to_owned())
        .with_group("".to_owned()) // Group less
        .with_offset_storage(Some(GroupOffsetStorage::Kafka))
        .create()?;

    let workflows_clone = workflows.clone();
    // Normal Consumer
    thread::spawn(move || normal_consumer_runner(normal_consumer, workflows, rollback_functions));

    // Edits Consumer
    thread::spawn(move || edits_consumer_runner(edits_consumer, workflows_clone));

    loop {
        thread::park();
    }
}
