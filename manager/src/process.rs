use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

pub type Service = String;
pub type Function = String;
pub type Workflow = HashMap<Process, Vec<Process>>;
pub type Workflows = HashMap<String, WorkflowTriple>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowTriple {
    pub workflow: Workflow,
    pub anti_workflow: Workflow,
    pub version: String, // Add version field
}

impl WorkflowTriple {
    pub fn new(workflow: Workflow, anti_workflow: Workflow, version: String) -> Self {
        Self {
            workflow,
            anti_workflow,
            version,
        }
    }
}


#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct Process {
    pub service: Service,
    pub function: Function,
}

impl Display for Process {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.service, self.function)
    }
}

impl Process {
    pub fn new(service: Service, function: Function) -> Self {
        Self { service, function }
    }
}
