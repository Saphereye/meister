use kafka::consumer::Consumer;
use kafka::producer::{Producer, Record};
use serde::Deserialize;
use std::collections::{HashMap, VecDeque};
use std::fmt::Write;
use std::sync::{Arc, RwLock};

pub mod models;
use models::*;

pub mod process;
use process::*;

use log::*;

#[derive(Deserialize)]
pub struct ManagerConfiguration {
    pub hosts: Vec<String>,
    pub rollback_functions: HashMap<Process, Process>,
}

fn send_from_manager(message: &FromManager, producer: &mut Producer) {
    let mut buf = String::new();
    write!(&mut buf, "{}", ron::ser::to_string(&message).unwrap()).unwrap();
    producer
        .send(&Record::from_value("frommanager", buf.as_bytes()))
        .unwrap();
}

pub fn normal_consumer_runner(
    mut normal_consumer: Consumer,
    mut producer: Producer, // TODO use this
    workflows: Arc<RwLock<Workflows>>,
    rollback_functions: HashMap<Process, Process>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        for ms in normal_consumer.poll().unwrap().iter() {
            for m in ms.messages() {
                let message: ToManager = match ron::de::from_bytes(m.value) {
                    Ok(value) => value,
                    Err(e) => {
                        error!("Error: {}", e);
                        continue;
                    }
                };

                let message_version = if let Some(version) = message.version.clone() {
                    version
                } else {
                    warn!("Version is not set, using latest version");
                    let version = {
                        let workflows_read = match workflows.read() {
                            Ok(workflows) => workflows,
                            Err(poisoned) => {
                                error!("Workflows lock is poisoned");
                                poisoned.into_inner()
                            }
                        };
                        let workflow_versions = workflows_read.get(&message.name);
                        if let Some(versions) = workflow_versions {
                            versions.keys().max().cloned()
                        } else {
                            None
                        }
                    };

                    if let Some(version) = version {
                        version
                    } else {
                        error!("No versions found for workflow: {}", message.name);
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
                            let workflows_read = match workflows.read() {
                                Ok(workflows) => workflows,
                                Err(poisoned) => {
                                    error!("Workflows lock is poisoned");
                                    poisoned.into_inner()
                                }
                            };

                            if let Some(workflow_versions) = workflows_read.get(&message.name) {
                                if let Some(triple) = workflow_versions.get(&message_version) {
                                    if let Some(processes) = triple.workflow.get(&current_process) {
                                        processes.clone()
                                    } else {
                                        error!(
                                            "No next processes found for current process: {}",
                                            current_process
                                        );
                                        continue;
                                    }
                                } else {
                                    error!("No workflow found for version: {}", message_version);
                                    continue;
                                }
                            } else {
                                error!(
                                    "No workflow versions found for message name: {}",
                                    message.name
                                );
                                continue;
                            }
                        };

                        for process in next_processes {
                            let next_message = FromManager {
                                uuid: message.uuid.clone(),
                                name: message.name.clone(),
                                version: message.version.clone().unwrap(),
                                process: process.clone(),
                                schema: message.schema.clone(),
                                data: message.data.clone(),
                            };
                            send_from_manager(&next_message, &mut producer);
                            trace!("Sending message: {}", next_message);
                        }
                    }
                    Status::InProgress => {
                        info!("{} is in progress", message);

                        // TODO: Implement timeout
                    }
                    Status::Failed => {
                        let anti_workflow = {
                            let workflows_read = match workflows.read() {
                                Ok(workflows) => workflows,
                                Err(poisoned) => {
                                    error!("Workflows lock is poisoned");
                                    poisoned.into_inner()
                                }
                            };

                            if let Some(workflow_versions) = workflows_read.get(&message.name) {
                                if let Some(triple) = workflow_versions.get(&message_version) {
                                    triple.anti_workflow.clone()
                                } else {
                                    error!("No workflow found for version: {}", message_version);
                                    continue;
                                }
                            } else {
                                error!(
                                    "No workflow versions found for message name: {}",
                                    message.name
                                );
                                continue;
                            }
                        };

                        let mut remaining_processes = VecDeque::from(vec![message.process.clone()]);

                        while !remaining_processes.is_empty() {
                            let current_process = remaining_processes.pop_front().unwrap();
                            for workflow in anti_workflow
                                .get(&current_process)
                                .unwrap_or(&Vec::<Process>::new())
                            {
                                remaining_processes.push_back(workflow.clone());
                                let rollback_process = rollback_functions.get(workflow).unwrap();

                                let next_message = FromManager {
                                    uuid: message.uuid.clone(),
                                    name: message.name.clone(),
                                    version: message.version.clone().unwrap(),
                                    process: rollback_process.clone(),
                                    schema: message.schema.clone(),
                                    data: message.data.clone(),
                                };

                                send_from_manager(&next_message, &mut producer);
                                trace!("Calling rollback: {}", rollback_process);
                            }
                        }
                    }
                }
            }
            let _ = normal_consumer.consume_messageset(ms);
        }
        normal_consumer.commit_consumed().unwrap();
    }
}

pub fn edits_consumer_runner(mut edits_consumer: Consumer, workflows: Arc<RwLock<Workflows>>) {
    loop {
        for ms in edits_consumer.poll().unwrap().iter() {
            for m in ms.messages() {
                let message: ToManagerEdits = match ron::de::from_bytes(m.value) {
                    Ok(value) => value,
                    Err(e) => {
                        error!("Error: {}", e);
                        continue;
                    }
                };
                info!("Received edit: {}", message);

                let anti_workflow = create_anti_workflow(&message.workflow);

                workflows
                    .write()
                    .unwrap()
                    .entry(message.name.clone())
                    .or_default()
                    .entry(message.version.clone())
                    .or_insert_with(|| {
                        WorkflowTriple::new(
                            message.workflow.clone(),
                            anti_workflow,
                            message.version.clone(), // Store version with workflow
                        )
                    });

                info!("Workflows updated: {:?}", workflows);
            }
            let _ = edits_consumer.consume_messageset(ms);
        }
    }
}

fn create_anti_workflow(original: &Workflow) -> Workflow {
    let mut reversed_connections: HashMap<Process, Vec<Process>> = HashMap::new();

    for (src, targets) in original.iter() {
        for target in targets {
            reversed_connections
                .entry(target.clone())
                .or_default()
                .push(src.clone());
        }
    }

    reversed_connections
}
