use crate::process::{Process, Workflow};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug)]
pub enum Status {
    Success,
    InProgress,
    Failed,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ToManager {
    pub uuid: String,
    /// Name of target workflow
    pub name: String,
    /// Version of target workflow
    pub version: String,
    pub process: Process,
    pub status: Status,
    /// ToManager input schema version
    pub schema: String,
    pub data: String, // BUG treat it as any type
}

impl Display for ToManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ToManager {{ uuid: {}..., name: {} ({}), Process: {}, status: {:?}, data: ... }}",
            self.uuid[..8].to_owned(),
            self.name,
            self.version,
            self.process,
            self.status
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FromManager {
    pub uuid: String,
    /// Name of target workflow
    pub name: String,
    /// Version of target workflow
    pub version: String,
    pub process: Process,
    /// FromManager output schema version
    pub schema: String,
    pub data: String, // BUG treat it as any type
}

impl Display for FromManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FromManager {{ uuid: {}..., name: {} ({}), Process: {}, data: ... }}",
            self.uuid[..8].to_owned(),
            self.name,
            self.version,
            self.process
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ToManagerEdits {
    /// Name of target workflow
    pub name: String,
    /// Version of target workflow
    pub version: String,
    /// ToManagerEdits input schema version
    pub schema: String,
    /// The graph of the workflow
    pub workflow: Workflow,
}

impl Display for ToManagerEdits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ToManagerEdits {{ name: {} ({}), workflow: ... }}",
            self.name, self.version
        )
    }
}
