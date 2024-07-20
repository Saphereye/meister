use crate::process::{Process, Workflow}; // Adjust this import according to your actual module structure
use ron::Value;
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
    pub data: Value,
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
    pub data: Value,
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

#[cfg(test)]
mod tests {
    use super::*;
    use ron::de::from_str;
    use std::collections::HashMap;

    #[test]
    fn test_deserialize_to_manager_edits() {
        let ron_string = r#"(
            name: "ExampleEdit",
            version: "v0.1.0",
            schema: "v0.1.0",
            workflow: {
                Process(service:"license",function:"add"):[
                    Process(service:"legal",function:"update")
                ],
                Process(service:"legal",function:"update"):[
                    Process(service:"license",function:"add")
                ]
            },
        )"#;

        let deserialized: ToManagerEdits = from_str(ron_string).expect("Deserialization failed");

        assert_eq!(deserialized.name, "ExampleEdit");
        assert_eq!(deserialized.version, "v0.1.0");
        assert_eq!(deserialized.schema, "v0.1.0");

        let process1 = Process {
            service: "license".to_string(),
            function: "add".to_string(),
        };
        let process2 = Process {
            service: "legal".to_string(),
            function: "update".to_string(),
        };
        let expected_workflow = HashMap::from([
            (process1.clone(), vec![process2.clone()]),
            (process2.clone(), vec![process1.clone()]),
        ]);

        assert_eq!(deserialized.workflow, expected_workflow);
    }
}
