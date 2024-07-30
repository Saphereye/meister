#import "@preview/ilm:1.1.3": *
#import "@preview/fletcher:0.5.1" as fletcher: diagram, node, edge
#import "@preview/note-me:0.2.1" as note: note, tip
#import "@preview/codly:1.0.0": *
#show: codly-init.with()

#set text(lang: "en")
#let colors = (maroon, olive, eastern)

#show: ilm.with(
  title: [Workflow Engine Report],
  author: "Adarsh Das",
  date: datetime(year: 2024, month: 07, day: 31),
  abstract: [
    This document is a report on the development of a workflow engine. The report covers the design, implementation, and testing of the engine. The engine is designed to be highly customizable and can be used in a variety of applications.
  ],
  preface: [
    #align(center + horizon)[
      This project was developed under the guidance of Mr. Naresh Krishnamoorthy as part of
      the internship at Dover Corporation.
    ]
  ],
  bibliography: bibliography("refs.bib"),
  figure-index: (enabled: true),
  table-index: (enabled: true),
  listing-index: (enabled: true)
)

#codly(display-name: false)
#codly(zebra-fill: none)

= The Basic structure of the engine
#diagram(
    edge-stroke: 1pt,
    node-corner-radius: 5pt,
    edge-corner-radius: 8pt,
    mark-scale: 80%,

    node((0,0.5), [Manager], fill: colors.at(0)),
    node((2,1), [Microservice 1], fill: colors.at(1)),
    node((2,0), [Microservice 2], fill: colors.at(1)),
    node((2,-1), [Microservice 3], fill: colors.at(1)),
    node((2,2), [Microservice n], fill: colors.at(1)),
    node((-2,0.5), [GUI Editor], fill: colors.at(2), shape: fletcher.shapes.hexagon),

    edge((0,0.5), (2,1), "<{-}>"),
    edge((0,0.5), (2,0), "<{-}>"),
    edge((0,0.5), (2,-1), "<{-}>"),
    edge((0,0.5), (2,2), "<{--}>"),
    edge((-2,0.5), (0,0.5), "-}>")
)

The workflow engine is designed to be highly modular and customizable. It consists
of the follwing components:
- Manager: The manager is responsible for coordinating the flow of data between the microservices.
- Microservices: The microservices are responsible for performing specific tasks.
- GUI Editor: The GUI editor allows users to create and edit workflows.

#note[
    Communication between the manager, microservices, and the editor is facilitated using Kafka.
]

= In-depth look at the components
== GUI Editor
#figure(
  image("editor.png", width: 100%),
  caption: [
    Screenshot of the GUI editor
  ],
)

The GUI editor utilizes the `Flume` Javascript library which enables it to drag and drop components to create workflows.
The editor also allows user the edit the metadata of the components, and finally allows to publish the kafka,
where the manager can consume to perform the relevant edits.

== Manager
The manager is responsible for coordinating the flow of data between the microservices.
Currently it listens to the following topics:
- `toManager`: This topic is used by the microservices to send data to the manager.
- `fromManager`: This topic is used by the manager to send data to the microservices.
- `toManagerEdits`: This topic is used by the editor to send data to the manager.

We can explain each topic logic in detail:
=== `toManager`
This topic is used by the microservices to send data to the manager. The data is sent in the form of a RON object.
An example of the data sent is:

```rust
ToManager(
    uuid: "585f6cf4-bf0e-4852-8ca3-c6866ce8a752",
    name: "user_registration",
    version: "v0.1.0",
    process: Process(
        service: "user",
        function: "create"
    ),
    status: Success,
    schema: "v0.1.0",
    data: "adarsh"
)
```

#tip[
    Here the `Process` struct is defined as follows in the code
    ```rust
    pub type Service = String; // aliased as a String
    pub type Function = String; // aliased as a String

    #[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
    pub struct Process {
        pub service: Service,
        pub function: Function,
    }
    ```

    And Success is part of an enum defined as follows:
    ```rust
    #[derive(Serialize, Deserialize, Debug)]
    pub enum Status {
        Success,
        InProgress,
        Failed,
    }
    ```
]

Once we receive the data,
we process it based on the relevant workflow and send the data to the relevant microservice.
For example, if we have the following workflow:

#diagram(
  node-stroke: 1pt,
  edge-stroke: 1pt,
  node-corner-radius: 5pt,
  edge-corner-radius: 8pt,

  node((0,0), [user.create], corner-radius: 2pt),
  edge((0,0), (2,0), "-|>"),
  node((2,0), [license.add], corner-radius: 2pt),
  edge((2,0), (4,0), "-|>"),
  node((4,0), [membership.add], corner-radius: 2pt)
)

Then the manager will send the data to the `license` microservice which executes the `add` function.

Now what if the status is `Failed`?, in that case we employ a rollback mechanism, similar to the way
we have in SAGA pattern @saga.
For example if we get the following data:
```rust
ToManager(
    uuid: "585f6cf4-bf0e-4852-8ca3-c6866ce8a752",
    name: "user_registration",
    version: "v0.1.0",
    process: Process(
        service: "membership",
        function: "add"
    ),
    status: Failed,
    schema: "v0.1.0",
    data: "adarsh"
)
```

we can refer to the rollback dictionary, i.e., a dictionary of processes and their corresponding rollback processes.
#note[
    The rollback for a process can be one or more processes.
]
```rust
Process(service:"legal",function:"update")  : Process(service:"legal",function:"revert"),
Process(service:"user",function:"create")   : Process(service:"user",function:"delete"),
Process(service:"membership",function:"add"): Process(service:"membership",function:"remove"),
Process(service:"license",function:"add")   : Process(service:"license",function:"remove")
```

and the anti-workflow, which is same as our workflow but with connections reversed.
#diagram(
  node-stroke: 1pt,
  edge-stroke: 1pt,
  node-corner-radius: 5pt,
  edge-corner-radius: 8pt,

  node((0,0), [user.create], corner-radius: 2pt),
  edge((0,0), (2,0), "<{-|"),
  node((2,0), [license.add], corner-radius: 2pt),
  edge((2,0), (4,0), "<{-|"),
  node((4,0), [membership.add], corner-radius: 2pt)
)

Thus as `membership.add` failed, we will use our anti workflow
and see that the previous process was `license.add` and thus we will call the `license.remove` function of the `license` microservice.
Then we will keep on following the anti-workflow until we reach the `user.create` function.
Finally here we will call the `user.delete` function of the `user` microservice.

#note[
  The `toManager` topic is read using a group id. This enables the manager to read the data from the microservices in a round-robin fashion.
]

=== `fromManager`
This topic is used by the manager to send data back to the microservices.
An example output is as follows,
```rust
FromManager(
    uuid: "585f6cf4-bf0e-4852-8ca3-c6866ce8a752",
    name: "user_registration",
    version: "v0.1.0",
    process: Process(
        service: "membership",
        function: "add"
    ),
    schema: "v0.1.0",
    data: "adarsh"
)
```

This signals the reading microservice that this data is for the `membership.add` function.

=== `toManagerEdits`
This topic is used by the editor to send data to the manager.
An example of the data sent is:
```rust
ToManagerEdits(
    name: "user_registration",
    version: "v0.1.0",
    schema: "v0.1.0",
    workflow: {
        Process(service:"license",function:"add"):[
            Process(service:"legal",function:"update")
        ],
        Process(service:"user",function:"create"):[
            Process(service:"license",function:"add"),
            Process(service:"membership",function:"add")
        ]
    }
)
```

This data contains the workflow for the required process.
The manager can then use this data to update the workflow in it local storage.

#note[
  Unlike `toManager` and `fromManager`, the `toManagerEdits` topic is not read using a group id, as we require edits to be propogated to all the managers.
]

= Conclusion
In conclusion, the workflow engine is a highly modular and customizable system designed to streamline and automate complex processes. It consists of a manager that coordinates data flow between various microservices, each performing specific tasks. The GUI editor enhances user experience by allowing easy creation and modification of workflows. Communication between components is efficiently managed using Kafka, ensuring seamless data exchange and process execution. This robust design makes the workflow engine adaptable to a wide range of applications, providing a solid foundation for efficient workflow management.
