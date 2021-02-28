# Midas
Midas is a distributed computing system written entirely in Rust. 

By running Midas on a host device then assigning participants we can create a distributed computing network using Lua. Messages between host and 
participant are passed using [message-io](https://docs.rs/message-io/0.8.1/message_io/) and code is executed using [hlua](https://docs.rs/hlua/0.4.1/hlua/).
These two combined allow the host to send code to participants for them to execute. 

## Host setup

Creating a host can be done by specifying an IP address (with port number) and the script to execute:

```shell
midas --mode=host --address=127.0.0.1:3000 --script"C:\script.lua"
```

## Participant setup

Creating a participant is similar, we must use the address we specified for the host (in this case 127.0.0.1:3000)

```shell
midas --mode=participant --address=127.0.0.1:3000
```

Note: Multiple instances of the participant can be executed on a single node, taking advantage of the CPU multiprocessing power of nodes. 

## Lua scripts

The Lua scripts are executed by the host and participants, not only to execute the parallel code, but also to load the input data and process the output data. The script must implement the three following functions

### `generate_data`

This function is called by the host for each participant and should be used to generate the input data for participants. It takes two integers as arguments, the index of the participant, and the number of participants registered, these can be used to split the data up. 

The `generate_data` function can be used to algorithmically generate data, or load data from a file on the host.

The return value is a table which is sent to the participant

Midas provides two extra functions that can be used to communicate extra information to the host, at the expense of increased overhead.
Using these functions is not mandatory, so for performance intensive calculations these can be ignored.

#### `_check`

The `_check` function is used to detect and handle pause/play/stop events sent by the host. 
For example, if a main loop is used within `generate_data` then calling `_check` occasionally within this loop will allow users to pause, play and stop the execution.

Note: The `_check` function carries some overhead, so calling it every iteration of a loop is highly discouraged. 

#### `_progress`

The `_progress` function is used so send a percentage (as f32) to the host to indicate the progress through the execution

### `execute_code`

The `execute_code` function is called by each participant and takes no arguments, but it does have access to a global variable, `global_data` which is simply the table returned by the `generate_data`. 

While no arguments are accepted by the function, any data can be sent to the function by including it in the `generate_data` step.

This function also returns a table and sends it back to the host on completion.

### `interpret_results`

This function is used to take the data from the `execute_code` calls, collects them and processes it. It also takes no arguments, and exposes another global variable `results` which is an array of tables, one for each table from each participant returned by `execute_code`.

This functions returns a string, which can be used to show a message indicating the result of the processing, or show an error message.

## Build

Midas uses a modified version of hlua, so if you build Midas yourself you must modify hlua such that the structs `AnyLuaValue` and `AnyLuaString` are serde serializable and deserializable.

This modified version of hlua can be found [here](https://github.com/ray33ee/hlua).

We also use the [available_concurrency](https://doc.rust-lang.org/std/thread/fn.available_concurrency.html) function which is currently nightly only.

## Host longevity

Once a task is started, the host application must run at least until the partcipants have all stopped, it may not stop earlier. It is also important to mention that a node can host as well as participate by using different processes for the host. 
This means that a dedicated Host node is not needed, and the host code can be run on any of the nodes.