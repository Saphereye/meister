import React, { useRef, useState } from "react";
import { NodeEditor } from "flume";
import config from "./config";

function App() {
  const editorRef = useRef();

  // State to hold user input for name and version
  const [name, setName] = useState("user_registration");
  const [version, setVersion] = useState("v0.1.0");

  const convertToRon = (jsonState) => {
    const processes = Object.keys(jsonState).reduce((acc, key) => {
      const node = jsonState[key];
      const service = node.inputData.multiSelect.service_name.toLowerCase();
      const func = node.inputData.multiSelect.function_name;
      const connections =
        node.connections.outputs.workflowNext?.map((connection) => {
          const targetNode = jsonState[connection.nodeId];
          const targetService =
            targetNode.inputData.multiSelect.service_name.toLowerCase();
          const targetFunction = targetNode.inputData.multiSelect.function_name;
          return `(service:"${targetService}",function:"${targetFunction}")`;
        }) || [];
      acc[`(service:"${service}",function:"${func}")`] = connections;
      return acc;
    }, {});

    // Convert the `processes` object to RON format
    const processesRon = Object.entries(processes)
      .map(([key, value]) => {
        const connections =
          value.length > 0 ? `[\n  ${value.join(",\n  ")}\n]` : "[]";
        return `${key}:${connections}`;
      })
      .join(",\n  ");

    return `(
      name: "${name}",
      version: "${version}",
      schema: "v0.1.0",
      workflow: {\n  ${processesRon}\n}
    )`
      .replace("\n", "")
      .replace(/\s+/g, "");
  };

  const handleSendToKafka = async () => {
    const currentState = editorRef.current.getNodes();
    console.log("Current state:", JSON.stringify(currentState, null, 2));
    const ronState = convertToRon(currentState);
    console.log("RON state:", ronState);

    try {
      // Convert RON string to a Blob
      const blob = new Blob([ronState], { type: "application/octet-stream" });

      const response = await fetch("http://localhost:5000/send-to-kafka", {
        method: "POST",
        body: blob,
      });

      if (response.ok) {
        console.log("Message sent to Kafka successfully");
      } else {
        console.error("Failed to send message to Kafka");
      }
    } catch (error) {
      console.error("Error sending message to Kafka:", error);
    }
  };

  const handleNameChange = (e) => setName(e.target.value);
  const handleVersionChange = (e) => setVersion(e.target.value);

  const buttonStyle = {
    position: "absolute",
    top: "10px",
    right: "10px",
    zIndex: "999",
    backgroundColor: "#007bff",
    color: "#fff",
    border: "none",
    borderRadius: "4px",
    padding: "10px 15px",
    cursor: "pointer",
    fontSize: "16px",
  };

  const formStyle = {
    position: "absolute",
    top: "10px",
    left: "10px",
    zIndex: "999",
    backgroundColor: "#fff",
    padding: "10px",
    borderRadius: "8px",
    boxShadow: "0 2px 4px rgba(0, 0, 0, 0.1)",
    fontFamily: "'Arial', sans-serif",
  };

  const labelStyle = {
    display: "block",
    marginBottom: "5px",
    fontSize: "14px",
    fontWeight: "bold",
  };

  const inputStyle = {
    display: "block",
    width: "200px",
    padding: "8px",
    marginBottom: "10px",
    border: "1px solid #ccc",
    borderRadius: "4px",
    fontSize: "14px",
  };

  return (
    <div style={{ position: "relative", width: "100vw", height: "100vh" }}>
      <NodeEditor
        ref={editorRef}
        portTypes={config.portTypes}
        nodeTypes={config.nodeTypes}
        style={{ width: "100vw", height: "100vh" }}
      />
      <div style={formStyle}>
        <label style={labelStyle}>
          Name:
          <input
            type="text"
            value={name}
            onChange={handleNameChange}
            style={inputStyle}
          />
        </label>
        <label style={labelStyle}>
          Version:
          <input
            type="text"
            value={version}
            onChange={handleVersionChange}
            style={inputStyle}
          />
        </label>
      </div>
      <button style={buttonStyle} onClick={handleSendToKafka}>
        Send to Kafka
      </button>
    </div>
  );
}

export default App;
