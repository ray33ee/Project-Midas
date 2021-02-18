# Midas
Midas is a distributed computing system written entirely in Rust. 

By running Midas on a host device then assigning nodes (participants) we can create a distributed computing networl. Messages between host and 
participant are passed using [message-io](https://docs.rs/message-io/0.8.1/message_io/) and code is executed using [MidasVM](https://github.com/ray33ee/stack-vm) which is itself heavily based on [stack_vm](https://docs.rs/stack-vm/1.0.1/stack_vm/).
These two combined allow the host to send code to participants for them to execute. 

Other features allow us to send data, commands and play, pause and stop individual participants. 

## Host setup

Creating a host can be done by specifying an IP address (with port number) 

```shell
midas --mode=host --address=127.0.0.1:3000
```

## Participant setup

Creating a participant is similar, we must use the address we specified for the host (in this case 127.0.0.1:3000)

```shell
midas --mode=participant --address=127.0.0.1:3000
```

Note: Multiple instances of the participant can be executed on a single node, taking advantage of the CPU multiprocessing power of nodes. 

## MidasVM

At the core of Midas is MidasVM, a custom stack based assembly-like weakly typed language. The language has a built in call stack, local variables, 
individual instruction stepping and even support for a heap (which we use in a read only capacity to accept data from the host). Midas VM
supports i64 and f64 primitives, and functions called on types will automatically convert them. Example code that
tests a number for primality can be found [here](https://github.com/ray33ee/Project-Midas/blob/master/docs/sample_code.txt). It takes three integers as data from the host, one as the prime to test, and the other two as a 
range of divisors to test. 

Most instructions work by pushing/popping values off the operand stack. MidasVM supports certain other operands like variables, built-in constants and literals. A brief
overview of operand syntax is given below
- \$VARIABLE a local variable, where VARIABLE is a string that adheres to the C-style of variable naming
- LABEL a label used for jumps
- .CONSTANT a built in constant defined in the body, or a pre-defined constant
- LITERAL an i64 or f64 number
- $stack the top of the stack
- \[INDEX\] data from the heap, indexed with INDEX (which can be indexed by $stack, a literal i64 or an i64 variable)

## Future ideas

- Use TUI to show connected participants (and their status, ip, name, etc.), progress through a task, play/pause/stop of participants
- Loading code and data from host disk to send to participant
- Saving/collection of data sent from participant to host
