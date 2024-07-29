use kafka::consumer::*;

use kafka::producer::Producer;
use log::info;
use manager::{edits_consumer_runner, normal_consumer_runner};
use manager::{process::*, ManagerConfiguration};

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // BUG: Should probably use never type here
    env_logger::init();

    ctrlc::set_handler(move || {
        info!("Closing application");
        std::process::exit(0); // BUG: Will this leave orphan threads?
    })
    .expect("Error setting Ctrl-C handler");

    let config: ManagerConfiguration = ron::de::from_str(include_str!("../config.ron"))?;

    // TODO: extend to many rollback process for one process
    let rollback_functions = config.rollback_functions;

    let workflows: Arc<RwLock<Workflows>> = Arc::new(RwLock::new(HashMap::new()));

    let normal_consumer = Consumer::from_hosts(config.hosts.clone())
        .with_topic("tomanager".to_owned())
        .with_group("manager".to_owned())
        .with_offset_storage(Some(GroupOffsetStorage::Kafka))
        .create()?;

    let edits_consumer = Consumer::from_hosts(config.hosts.clone())
        .with_topic("tomanageredits".to_owned())
        .with_group("".to_owned()) // Group less
        .with_offset_storage(Some(GroupOffsetStorage::Kafka))
        .create()?;

    let producer = Producer::from_hosts(config.hosts)
        .with_ack_timeout(std::time::Duration::from_secs(1))
        .with_required_acks(kafka::client::RequiredAcks::One)
        .create()?;

    let workflows_clone = workflows.clone();

    // Normal Consumer
    thread::spawn(move || {
        normal_consumer_runner(normal_consumer, producer, workflows, rollback_functions).unwrap()
    });

    // Edits Consumer
    thread::spawn(move || edits_consumer_runner(edits_consumer, workflows_clone));

    loop {
        thread::park();
    }

    // Bye Bye 🤫🧏
}
