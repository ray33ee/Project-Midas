# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]
### To Do
- When dealing with tables returned by Lua, make sure none of th key-data pairs are `LuaOther` as these will not be converted correctly. Warn user that tables within tables are not yet supported.
- Upgrade to messages-io `0.10.0`

### Unfinished Ideas
- Can we use `AnyLuaValue` type to store tables?
  - Experiment with the `LuaArray` option to see if we can use this as a table
- See if rlua supports tables of tables, if it does migrate to rlua.
- Use the Gague widget to show progress?
- Can we selectively pick the parts of the dependencies we need instead of loading all of it?
- Think of a good way to select scripts while running?
  - At the moment the user can just modify the script (thge one chosen when the application started) since executing will load the modified scriptc

## [0.2.18] - 2021-03-07

### Added
- We now make sure the participants are idle before we start execution
- Error handling for script loading error and bad script
- Error handling for various types of script-based errors

### Fixed
- Word 'client' changed to 'participant' for clarity and uniformity
- Selecting up or down key with no participants no longer causes a panic
- Now if Lua script functions fail, or throw an error we handle the error and add an entry to the log

### Changed
- SendCode, Execute and SendData commands removed in favour of direct functions that return `Result`

## [0.2.17] - 2021-03-07

### Added
- Correct image used in readme

## [0.2.16] - 2021-03-07

### Added
- Scroll up/down added for log events
- PgUp and PgDn shortcuts added to shortcut bar
- Added binaries to SourceForge and added entry to readme

### Changed
- Threads are numbered with leading zeros when named
- Bounds checking on participant list removed to make use of TUI's built in scrolling for participant list

## [0.2.15] - 2021-03-07

### Added
- Decent looking color palette
- `Starting` Severity for when we start execution

### Fixed
- Before we quit we remove all endpoints to avoid panic in thread 'message-io: tcp-adapter'
- Cleared warnings

### Removed
- We no longer print log message when a client connects (as a connecting client could be a test to determine whether to connect the real participants)
- Removed old commented code

### Changed
- Highlighting now changes background colour of highlighted text to dark grey
- We now only send the endpoint AND name in messages that need it



## [0.2.14] - 2021-03-04
### Added
- First participant is automatically selected when first available
- Renamed stop to kill
- Kill command now adds entry to log
- Kill works by calling `std::process::exit` instead of panicking
- Before `Participant` struct connects, we check to make sure a host is available. we do this by creating a temporary client to connect to host.
- When a device disconnects, we make sure it has registered before we try and remove it.
- Participant will now sit and wait until the host is available

### Fixed
- When the network monitor thread call to recv fails, the thread exits

## [0.2.13] - 2021-03-03
### Added
- `_print` to allow user to display messages in the Host ui from participants. Can only be called in `execute_code` function

### Changed
- `_progress` now ensures that the progress update cannot be sent too frequently by choosing a duration in milliseconds between progress updates (users should still minimise how often this function is called though)
- Since progress must be stored as an integer (to derive Hash for ParticipantInfo) we multiply the percentage by 100 when we store it, to give us 2 decimal places
- Selecting from a list now shows info in TUI (no need to press enter each time)
- Updated readme with extra, more helpful information
- Starting the server now adds an entry to the log
- All println! calls in host changed to eprintln!
- We now quit the host properly, without panicking
- Shortcut added to clear log

## [0.2.12] - 2021-03-03
### Added
- List state for selecting participants and showing information
- `Result` severity for highlighting when a result is returned from execution

### Fixed
- Pausing and playing while idle will no longer change the status, since participants must inform the host when they pause/unpause

### Removed
- Actions panel in favour of keyboard shortcuts

## [0.2.11] - 2021-03-02
### Added
- Added MIT license
- Tui has been added showing participants and events
- All println! commands removed from `Host` and converted to `Panel` events
- Status of participants now added, colours indicate status
- Errors, warnings and info sent to or created by `Host` are also showed in the Tui
- Structs and enums for storing information on participants in `Panel` and functions to convert to TUI objects
- Shortcut bar at bottom of ui showing all the available shortcuts

### Changed 
- Updated readme to reflect changes to command line options
- Using stop when participants are idling now stops them

## [0.2.10] - 2021-02-28
### Added
- `Participant::recv_message` function simplifies the processing of incoming messages and the possible errors that can occur 

### Removed
- `capture` dependency removed as it wasn't used enough to warrant it

### Changed
- We use crossbeam's `scope` to allow us to use references in spawned threads, meaning we don't have to clone the `participant_name` and `ip_address` strings for each thread
- Modified the `thread count` option such that when it is used the user specifies the number of threads, and when it is omitted we use all available concurrency

## [0.2.9] - 2021-02-26

### Changed
- Stop message has been changed to kill, which forcibly removes the participant during execution
- Updated readme.md

### Added
- Code to handle pause/play and

## [0.2.8] - 2021-02-25
### Added
- Proof of concept for play/pause/stop commands and progress
- Temporary code to `Host` to process a `Message::Progress` message
- _progress and _check functions to Lua test script

### Changed
- Adding a monitoring thread to `Participant` means that the network is effectively multiple sender single consumer

### Removed
- `Event` enum in participant.rs since it is only relevant for GUI code

## [0.2.7] - 2021-02-24
### Changed
- Migrated from std `mpsc` to crossbeam-channel message passing which allow us to move a cloned receiver to the `check` closure
- Renamed `Participant::check_events` to `Participant::tick`

### Added
- Extra keystrokes to temporary ui code, that allows us to send pause, start and stop signals
- `Message::PlayAll`, `Message::StopAll` and `Message::PauseAll` messages

## [0.2.6] - 2021-02-23
### Changed
- Migrated from message-io's `EventQueue` to `mpsc` message passing 

### Added
- `Host::check_events` method now executes in saparate thread alongside the main thread which takes care of the ui.

## [0.2.5] - 2021-02-21

### Added
- `Host::new` returns a `Result` that indicates if the starting of the server failed or succeeded

### Changed
- `Host::participants_finished` field is reset when the `Host::start_participants` is called, not when the data is received
- All participants are executed on spawned threads, the main thread now only waits for spawned threads
- Certain println! messages migrated from `Host` methods to `Panel` events
- Host events moved into messages.rs
- `ParticipantStatus` enum added and implemented for `Idle` and `Calculating` 
- `Participant::check_events` now returns `Result` so if the function fails, the thread can terminate. If all threads terminate, the application ends.

## [0.2.4] - 2021-02-21

### Added
- `Host::participants_startedwith` field added to keep track of the number of participants registered when the computation began.
- We now use `thread::available_concurrency` to automatically select the number of threads to use (use -t or --threads without specifying a number)
- Temporary key-press commands added (`e` for execution, `d` for displaying participants and `c` for showing how many participants are connected.)

### Changed
- Less debuggy, less verbose printed messages from `Host`
- All spawned threads in participant nodes are named based on the supplied participant name

### Fixed
- Trying to add a participant with a name that already exists will cause the offending participant to be disconnected
- We verify that the host acts correctly when a `ParticipantError` is received, namely
  - The offending participant is removed from the participant list, so no more calculations can take place
- Host now stops computation if the participants have changed since the start of the computation (i.e. any participants have connected/disconnected)

## [0.2.3] - 2021-02-20
### Added
- Multithreading added, so multiple participants can be executed from a single process
- Command line arguments for the number of threads to spawn and the name of participant(s)
- `Host::participants` has been changed to a bidirectional map for easy insertion and deletion from either string name or the endpoint
- If multiple threads are used, then the number of the thread (arbitrary) is appended to the --name to ensure each participant has a unique name
- The participant name is now sent in the `Messages::Register` message
- Host `Message` matching now includes arms for `ParticipantError` and `ParticipantWarning`

## [0.2.2] - 2021-02-20
### Added
- Host and participant selection via clap now uses subcommands, and --script option is only required for host subcommand
- --script validator added
- Error handling implemented for participant
- Extra messages added for sending strings from participant to host

### Fixed
- Various warnings cleared (Pause, play and stop warnings still exist as these are not yet implemented)

## [0.2.1] - 2021-02-20
### Added
- We now work with `SerdeLuaTable`, `Vec<(AnyLuaValue, AnyLuaValue)>`, to allow user to pass tables between host and participant
- Modified host code so that the results global variable (used by `interpret_results`) is now an array of tables, each table coming from a participant
- Added lua.rs to handle common hlua (or rlua) code

### Changed
- Modified `AnyLuaValue` to be serializable 

### Removed
- We no longer store the incoming data in Host as it is sent directly to Lua context

## [0.2.0] - 2021-02-19
### Removed
- `stack_vm` has been removed in favor of a Lua script
  
### Added
- Clap now accepts a --script command line argument
- hlua which handles the Lua compiling
- Host and Participants now call their function from the Lua script  
- `interpret_results` now returns a string
- Simple prime divisibility algorithm implemented as test Lua script

### Changed
- Certain Send events (within the host) now send data/code to all endpoints. 
  NOTE: This may change in the future.
- Code loading and execution now two separate events
- Host contains data field containing all data sent from all participants (created by each participants execution of `execute_code`)

## [0.1.8] - 2021-02-18
### Fixed
- Timeout on event_queue receive_timeout is reduced to 1micro second to speed up vm. This event is executed very quickly, and really only servers to check if an event has been sent
- Cleared various 'unused import' warnings

### Added
- Host::test_participant_event now works for an arbitrary number of participants
- Added readme
- Added print_v for printing contents of variables
- Added compile_file function

### Changed
- compile_file and compile_source added to Compiler impl

### Removed
- Removed test() function in compiler.rs

## [0.1.7] - 2021-02-17
### Added
- We now declare lazy statics in a more global sense to avoid having to pass them around
- mod instruction added, and is used in the sample_code
- Sample code now uses a jz instruction in loop
- Compiler.rs now contains an iterator that walks over each line in source and outputs a Statement enum
- The statement enum is then interpreted and added to the builder
- print_s instruction now prints the entire stack instead of just the top

## [0.1.6] - 2021-02-15
### Added
- Added functionality to enable play, pause and stopping of the VM
- Better changelog created
- Bulk of the work for compiler is complete
  - Enums for statements, tokens, etc.
  - Conversion from string into enums
  - Thinking behind strategy documented in compiler.rs
  - Regexes added for matching
  - Extensive use of regex and slice pattern matching

### Fixed
- \_d functions now use Machine::heap field instead of local variables

## [0.1.5] - 2021-02-15
### Added
- We now automatically pull the version number from Cargo.toml for use in clap
- stack_vm now modified into iterator that allows us to step instructions
- stack_vm modified to allow a heap
- New stack_vm added to github, can be found [here](https://github.com/ray33ee/stack-vm)
- Participant now contains the machine as a field
- Each call of check_events will call Machine::next() if the machine is loaded
- Stepping over machine prevents the blocking affect of Machine::run
- Participant::check_events now uses EventQueue::receive_timeout so we can execute in within main loop

## [0.1.4] - 2021-02-14
### Added
- Logical numbering of opcodes grouping similar instructions together and leaving gaps for future codes
- Added hash map for constants (to be used by compiler)
- Added conversion from Operand to f64 (for use in f64 functions)
- Sample source code for MidasVM has now been added to outline what source code will look like

### Fixed
- Now we use SocketAddrV4 to validate IP addresses (with port number)

## [0.1.3] - 2021-02-13
### Added
- Simple address command line option for both host and participant
- We now work with Vec<Operand> instead of two different vectors
- When the MidasVM finishes, it sends what's left of its stack to the host
- Added a bunch of useful constants from std::f64::consts
- We now have a way to transfer data to participant independent of the Code. We do this by moving the data on to a list of local variables addressed from 'd_0'
- Added more mov functions (movdl, movds, movdv and movc)
- Constants declared in instructions.rs

### Fixed
- Copy trait added to Operand, cleared various borrow-check warnings

## [0.1.2] - 2021-02-12
### Added
- Can now send code from host to participant
- Modified code in Host and Participant structs to setup in new() and check in check_events
- Participant and Host now check network for the various types of messages outlined in Message enum
- Cleared various warnings
- -mode argument now validated with possible_values function instead of custom validator
- Participant to Host Vectors now include an i64, used to identify if needed

## [0.1.1] - 2021-02-11
### Added
- Added very simple client-server network connection
- Added various modules for network host, participant, common code(messages.rs) and compiler option
- StackVM code is now serializable
- Host or participant mode now selected via command line arg

## [0.1.0] - 2021-02-10
### Added
- Initial commit