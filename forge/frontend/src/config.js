import { Colors, FlumeConfig, Controls } from 'flume';

// Mock arrays for services and functions
// const services = ['User', 'License', 'Membership', 'Legal'];
const functions = [
    'create_user', 'delete_user', 
    'add_license', 'remove_license',
    'add_membership', 'remove_membership',
    'update_legal', 'delete_legal'
];

const serviceFunctions = {
    'User': ['create_user', 'delete_user'],
    'License': ['add_license', 'remove_license'],
    'Membership': ['add_membership', 'remove_membership'],
    'Legal': ['update_legal', 'delete_legal']
};

const config = new FlumeConfig()
    .addPortType({
        type: 'workflow',
        name: 'Workflow',
        label: 'Workflow',
        color: Colors.blue,
        noControls: true
    })
    .addPortType({
        type: 'multiSelect',
        name: 'Service and Function Select',
        label: 'Service and Function Select',
        controls: [
            Controls.select({
                name: 'service_name',
                label: 'Service Name',
                options: Object.keys(serviceFunctions).map(service => ({
                    value: service,
                    label: service
                }))
            }),
            Controls.select({
                name: 'function_name',
                label: 'Function Name',
                options: functions.map(func => ({
                    value: func,
                    label: func
                }))
            })
        ]
    })
    .addNodeType({
        type: 'workflow',
        label: 'Workflow',
        inputs: ports => [
            ports.workflow({ name: 'workflowPrev', label: 'Prev' }),
            ports.multiSelect({ name: 'multiSelect', label: 'Multi Select' })
        ],
        outputs: ports => [
            ports.workflow({ name: 'workflowNext', label: 'Next' }),
        ]
    });

export default config;
