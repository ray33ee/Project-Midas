# Midas
Midas is a distributed computing system written entirely in Rust. 

By running Midas on a host device then assigning participants we can create a distributed computing network using Lua. Messages between host and 
participant are passed using [message-io](https://docs.rs/message-io/0.8.1/message_io/) and code is executed using [hlua](https://docs.rs/hlua/0.4.1/hlua/).
These two combined allow the host to send code to participants for them to execute. 

![Screenshot](https://imgur.com/AI14BqC)

## Why?

There is no shortage of distributed computing models, and each model has many implementations. If power and performance is
required these solutions are undoubtedly the best, especially for performing one task, and doing it well. 

However, these solutions are extremely thorough and therefore have a steep learning curve. For general experimenting Midas is 
perfect since it is easy to learn (only knowledge of Lua and this readme is required) and it easy to swap out algorithms, 
rather than being having to stick with a single executable.

## Host setup

Creating a host can be done by specifying an IP address (with port number) and the script to execute:

```shell
midas --address=127.0.0.1:3000 host --script"C:\script.lua"
```

## Participant setup

Creating a participant is similar, we must use the address we specified for the host (in this case 127.0.0.1:3000) and this time a unique name for the participant.

```shell
midas --address=127.0.0.1:3000 participant --name="laptop"
```

A name must be supplied to identify the participants in the host. If the number of threads is omitted, we automatically determine the number of threads to use.

## Lua scripts

The Lua scripts are executed by the host and participants, not only to execute the parallel code, but also to load the input data and process the output data. The script must implement the three following functions

A single command may create multiple participants, this is because we try to create as many participants as the computer can handle concurrently. This can be controlled with the threads command line option.

### `generate_data`

This function is called by the host for each participant and should be used to generate the input data for participants. It takes two integers as arguments, the index of the participant, and the number of participants registered, these can be used to split the data up. 

The `generate_data` function can be used to algorithmically generate data, or load data from a file on the host.

The return value is a table which is sent to the participant

Midas provides two extra functions that can be used to communicate extra information to the host, at the expense of increased overhead.
Using these functions is not mandatory, so for performance intensive calculations these can be ignored.

#### `_check`

Detects and handles pause/play/stop events sent by the host. 
For example, if a main loop is used within `generate_data` then calling `_check` occasionally within this loop will allow users to pause, play and stop the execution.

Note: The `_check` function carries some overhead, so calling it every iteration of a loop is highly discouraged. 

#### `_progress`

Sends a percentage (as `f32`) to the host to indicate the progress through the execution.

It also takes a `u32`, which is the max duration (in milliseconds) between progress updates. It helps prevent the code from sending too many progress updates too frequently and slowing down execution.

Even with the duration restriction, calling `_progress` still incurs some overhead, so should not be called too frequently.

#### `_print`

Accepts a string, used to print custom messages which will be displayed on the host. Using Lua's `print` will print to the participant and will NOT print to host.

### `execute_code`

The `execute_code` function is called by each participant and takes no arguments, but it does have access to a global variable, `global_data` which is simply the table returned by the `generate_data`. 

While no arguments are accepted by the function, any data can be sent to the function by including it in the `generate_data` step.

This function also returns a table and sends it back to the host on completion.

### `interpret_results`

This function is used to take the data from the `execute_code` calls, collects them and processes it. It also takes no arguments, and exposes another global variable `results` which is an array of tables, one for each table from each participant returned by `execute_code`.

This functions returns a string, which can be used to show a message indicating the result of the processing, or show an error message.

## Build

To build, simply download and unzip the [repo](https://github.com/ray33ee/Project-Midas/archive/master.zip), navigate to the unzipped repo and execute the following command

```shell
cargo +nightly build --release
```

Then navigate to the `target/release` folder and execute `midas` with the command options as stated above

We also use the [available_concurrency](https://doc.rust-lang.org/std/thread/fn.available_concurrency.html) function which is currently nightly only.

## Native binaries

Alternatively you can find compiled binaries for midas [here](https://sourceforge.net/projects/project-midas/)

## Host longevity

Once a task is started, the host application must run at least until the partcipants have all stopped, it may not stop earlier. If it does, all participants will stop immediately. It is also important to mention that a node can host as well as participate by using different processes for the host. 
This means that a dedicated Host node is not needed, and the host code can be run on any of the nodes.