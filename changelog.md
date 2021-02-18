# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]
### To Do
- Add instructions (including loads of useful maths functions, bitwise functions)
- Make sure all bytecode is appended with the 'stop' label (with an index of -1 to avoid conflicts)
- Find a way dynamically generate certain repetitive code based on xml or json. Candidates include:
  - List of instructions (created in get_instructions)
  - List of pre-defined constants (created in get_constants)
  - Pre-defined constant map (created in get_constants_map)
- Modify float_reg to allow lack of leading zero, such as '.3' and exponents such as '0.3e4' still making sure we do not allow integers
- Implement error handling for compiler.rs (go over unpack and expect first)
- Make sure the constants table we generate doesn't conflict with the predefined constants
- Use a TUI in host.rs to allow user to view the participants, their status, send data, code and commands.

### Unfinished Ideas
- Think of a way to handle data sent from participant to host.
  - Where should it be saved on the computer?
  - How should it be serialised
- Do the constants, label and variable maps need to be maps? can't they just be lists?

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