# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]
### To Do
- Use a TUI in host.rs to allow user to view the participants, their status, send data, code and commands.
  - Any `println!` message currently used needs to be mapped to some form of TUI widget
  - Allow user to control participants and execute code via host
  - Show status of participants in TUI (Idle, calculating, etc.)
  - Show any critical errors or warnings from host or participant 
- Implement split function that can be used by Lua script to split data up for each participant

### Unfinished Ideas
- rlua or hlua?
- Implement play/pause/stop
- Implement progress function
- Can we use `AnyLuaValue` type to store tables?
  - Experiment with the `LuaArray` option to see if we can use this as a table

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

## Removed
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