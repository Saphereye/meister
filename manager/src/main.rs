use kafka::consumer::*;

use manager::{edits_consumer_runner, normal_consumer_runner};
use manager::{process::*, ManagerConfiguration};

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // BUG: Should probably use never type here
    env_logger::init();

    ctrlc::set_handler(move || {
        println!("Bye Bye 🤫🧏");
        std::process::exit(0); // BUG: Will this leave orphan threads?
    })
    .expect("Error setting Ctrl-C handler");

    let config: ManagerConfiguration = ron::de::from_str(include_str!("../config.ron"))?;

    // TODO: extend to many rollback process for one process
    let rollback_functions = config.rollback_functions;

    let workflows: Arc<RwLock<Workflows>> = Arc::new(RwLock::new(HashMap::new()));

    // TODO: put all repeated stuff in a config file
    let normal_consumer = Consumer::from_hosts(config.hosts.clone())
        .with_topic("tomanager".to_owned())
        .with_group("manager".to_owned())
        .with_offset_storage(Some(GroupOffsetStorage::Kafka))
        .create()?;

    let edits_consumer = Consumer::from_hosts(config.hosts)
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
