import { Colors, FlumeConfig, Controls } from "flume";

// Mock arrays for services and functions
// const services = ['User', 'License', 'Membership', 'Legal'];
const functions = ["create", "delete", "update"];

const serviceFunctions = {
  User: ["create", "delete"],
  License: ["create", "delete"],
  Membership: ["create", "delete"],
  Legal: ["update", "delete"],
};

const config = new FlumeConfig()
  .addPortType({
    type: "workflow",
    name: "Workflow",
    label: "Workflow",
    color: Colors.blue,
    noControls: true,
  })
  .addPortType({
    type: "multiSelect",
    name: "Service and Function Select",
    label: "Service and Function Select",
    controls: [
      Controls.select({
        name: "service_name",
        label: "Service Name",
        options: Object.keys(serviceFunctions).map((service) => ({
          value: service,
          label: service,
        })),
      }),
      Controls.select({
        name: "function_name",
        label: "Function Name",
        options: functions.map((func) => ({
          value: func,
          label: func,
        })),
      }),
    ],
  })
  .addNodeType({
    type: "workflow",
    label: "Workflow",
    inputs: (ports) => [
      ports.workflow({ name: "workflowPrev", label: "Prev" }),
      ports.multiSelect({ name: "multiSelect", label: "Multi Select" }),
    ],
    outputs: (ports) => [
      ports.workflow({ name: "workflowNext", label: "Next" }),
    ],
  });

export default config;
