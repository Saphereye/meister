use kafka::consumer::Consumer;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

pub mod models;
use models::*;

pub mod process;
use process::*;

use log::*;

pub fn normal_consumer_runner(
    mut normal_consumer: Consumer,
    workflows: Arc<Mutex<Workflows>>,
    rollback_functions: HashMap<Process, Process>,
) {
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
    }
}

pub fn edits_consumer_runner(mut edits_consumer: Consumer, workflows: Arc<Mutex<Workflows>>) {
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
                info!("Received edit: {:?}", message);

                let anti_workflow = create_anti_workflow(&message.workflow);
                let mut workflows = workflows.lock().unwrap();
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
    }
}

pub fn create_anti_workflow(original: &Workflow) -> Workflow {
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
