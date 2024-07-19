use kafka::consumer::*;
use log::*;

mod models;
use models::*;

mod process;
use process::*;

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;

use ron;

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

    let mut normal_consumer = Consumer::from_hosts(vec!["localhost:9092".to_owned()])
        .with_topic("tomanager".to_owned())
        .with_group("manager".to_owned())
        .with_offset_storage(Some(GroupOffsetStorage::Kafka))
        .create()?;

    let mut edits_consumer = Consumer::from_hosts(vec!["localhost:9092".to_owned()])
        .with_topic("tomanageredits".to_owned())
        .with_group("".to_owned()) // Group less
        .with_offset_storage(Some(GroupOffsetStorage::Kafka))
        .create()?;

    let workflows_clone = workflows.clone();

    // Normal Consumer
    thread::spawn(move || loop {
        for ms in normal_consumer.poll().unwrap().iter() {
            for m in ms.messages() {
                let message: ToManager = match ron::de::from_bytes(m.value) {
                    Ok(value) => value,
                    Err(e) => {
                        error!("Error: {}", e);
                        continue;
                    }
                };
                info!("Received message: {}", message);

                match message.status {
                    Status::Success => {
                        let current_process = Process::new(
                            message.process.service.clone(),
                            message.process.function.clone(),
                        );
                        let next_processes = {
                            let workflows_guard = workflows.lock().unwrap();
                            workflows_guard
                                .get(&message.name)
                                .unwrap() // BUG: unwrap
                                .workflow
                                .get(&current_process)
                                .unwrap() // BUG: unwrap
                                .clone()
                        };

                        for process in next_processes {
                            let next_message = FromManager {
                                uuid: message.uuid.clone(),
                                name: message.name.clone(),
                                version: message.version.clone(),
                                process: process.clone(),
                                schema: message.schema.clone(),
                                data: message.data.clone(),
                            };

                            trace!("Sending message: {}", next_message);
                        }
                    }
                    Status::InProgress => {
                        info!("{} is in progress", message);
                    }
                    Status::Failed => {
                        let anti_workflow = workflows
                            .lock()
                            .unwrap()
                            .get(&message.name)
                            .unwrap()
                            .anti_workflow
                            .clone();

                        let mut remaining_processes = VecDeque::from(vec![message.process.clone()]);

                        while !remaining_processes.is_empty() {
                            let current_process = remaining_processes.pop_front().unwrap();
                            for workflow in anti_workflow
                                .get(&current_process)
                                .unwrap_or(&Vec::<Process>::new())
                            {
                                remaining_processes.push_back(workflow.clone());

                                let rollback_process = rollback_functions.get(workflow).unwrap();
                                trace!("Calling rollback: {}", rollback_process);
                            }
                        }
                    }
                }

                info!("Message over: {}", message);
            }
            let _ = normal_consumer.consume_messageset(ms);
        }
        normal_consumer.commit_consumed().unwrap();
    });

    // Edits Consumer
    thread::spawn(move || loop {
        for ms in edits_consumer.poll().unwrap().iter() {
            for m in ms.messages() {
                let message: ToManagerEdits = match ron::de::from_bytes(m.value) {
                    Ok(value) => value,
                    Err(e) => {
                        error!("Error: {}", e);
                        continue;
                    }
                };
                info!("Received edit: {:?}", message);

                let anti_workflow = create_anti_workflow(&message.workflow);
                let mut workflows = workflows_clone.lock().unwrap();
                workflows.insert(
                    message.name.clone(),
                    WorkflowTriple::new(
                        message.workflow.clone(),
                        anti_workflow,
                        message.version.clone(), // Store version with workflow
                    ),
                );

                info!("Workflows updated: {:?}", workflows);
            }
            let _ = edits_consumer.consume_messageset(ms);
        }
    });

    loop {
        thread::park();
    }
}

fn create_anti_workflow(original: &Workflow) -> Workflow {
    let mut reversed_connections = HashMap::new();

    for (src, targets) in original.iter() {
        for target in targets {
            reversed_connections
                .entry(target.clone())
                .or_insert_with(Vec::new)
                .push(src.clone());
        }
    }

    reversed_connections
}
