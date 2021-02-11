use stack_vm::{InstructionTable, Instruction, Machine};

use crate::operand::Operand;

pub fn get_instructions() -> InstructionTable<Operand> {
    let mut instruction_table = InstructionTable::new();
    instruction_table.insert(Instruction::new(0, "pushl", 1, push_l));
    instruction_table.insert(Instruction::new(1, "pushv", 1, push_v));
    instruction_table.insert(Instruction::new(2, "movl", 2, mov_l));
    instruction_table.insert(Instruction::new(3, "movv", 2, mov_v));
    instruction_table.insert(Instruction::new(4, "add", 0, add));
    instruction_table.insert(Instruction::new(5, "popv", 1, pop_v));
    instruction_table.insert(Instruction::new(6, "call", 1, call));
    instruction_table.insert(Instruction::new(7, "ret", 0, ret));
    instruction_table.insert(Instruction::new(8, "jmp", 1, jmp));
    instruction_table.insert(Instruction::new(9, "jz", 1, jz));
    instruction_table.insert(Instruction::new(10, "jnz", 1, jnz));
    instruction_table.insert(Instruction::new(11, "sub", 0, sub));
    instruction_table.insert(Instruction::new(12, "print_s", 0, print_s));
    instruction_table.insert(Instruction::new(13, "inc", 0, inc));
    instruction_table.insert(Instruction::new(14, "dec", 0, dec));
    instruction_table.insert(Instruction::new(15, "copy", 0, copy));
    instruction_table.insert(Instruction::new(1, "pushc", 1, push_c));
    instruction_table
}

/* Data Transfer */

//Push a literal on to the stack
pub fn push_l(machine: &mut Machine<Operand>, args: &[usize]) {
    let arg = machine.get_data(args[0]).clone();
    machine.operand_push(arg);
}

//Push the contents of a local variable on the stack
pub fn push_v(machine: &mut Machine<Operand>, args: &[usize]) {
    let operand = machine.get_data(args[0]).clone();

    let local = machine.get_local(operand.str_identifier().as_str()).clone();
    match local {
        Some(value) => {
            machine.operand_push(value.clone());
        },
        None => {
            //Send info to host to inform of error
            panic!("No such local variable");
        }
    }
}

pub fn push_c(machine: &mut Machine<Operand>, args: &[usize]) {
    let ind = machine.get_data(args[0]).clone();
    let constant = machine.constants.get(ind.str_identifier().as_str()).expect("Unkown constant").clone();
    machine.operand_push(constant);
}

//Pop value off stack into local variable
pub fn pop_v(machine: &mut Machine<Operand>, args: &[usize]) {
    let operand = machine.get_data(args[0]).clone();
    let arg = machine.operand_pop().clone();
    machine.set_local(operand.str_identifier().as_str(), arg);
}

//Move a literal into a local variable
pub fn mov_l(machine: &mut Machine<Operand>, args: &[usize]) {
    let operand = machine.get_data(args[0]).clone();
    let arg = machine.get_data(args[1]).clone();
    machine.set_local(operand.str_identifier().as_str(), arg);
}

//Move a value from one local variable directly to another?
pub fn mov_v(machine: &mut Machine<Operand>, args: &[usize]) {
    let opa = machine.get_data(args[0]).clone();
    let opb = machine.get_data(args[1]).clone();
    let local = machine.get_local(opb.str_identifier().as_str());
    match local {
        Some(variable) => {
            machine.set_local(opa.str_identifier().as_str(), variable.clone());
        },
        None => {
            //Send info to host to inform of error
            panic!("No such local variable");
        }
    }
}

pub fn copy(machine: &mut Machine<Operand>, args: &[usize]) {
    let top = machine.operand_stack.peek().clone();
    machine.operand_push(top);
}

/* Maths functions */

pub fn add(machine: &mut Machine<Operand>, _args: &[usize]) {
    let rhs = machine.operand_pop().clone();
    let lhs = machine.operand_pop().clone();
    machine.operand_push(lhs + rhs);
}

pub fn sub(machine: &mut Machine<Operand>, _args: &[usize]) {
    let rhs = machine.operand_pop().clone();
    let lhs = machine.operand_pop().clone();
    machine.operand_push(lhs - rhs);
}

pub fn inc(machine: &mut Machine<Operand>, _args: &[usize]) {
    let top = machine.operand_pop().clone();
    machine.operand_push(top + Operand::I64(1));
}

pub fn dec(machine: &mut Machine<Operand>, _args: &[usize]) {
    let top = machine.operand_pop().clone();
    machine.operand_push(top - Operand::I64(1));
}


/* Compare */



/* Jumps */

fn jmp(machine: &mut Machine<Operand>, args: &[usize]) {
    let label = machine.get_data(args[0]).clone();
    machine.jump(label.str_identifier().as_str());
}

fn jz(machine: &mut Machine<Operand>, args: &[usize]) {
    let top = machine.operand_pop();

    if top == Operand::I64(0) {
        let label = machine.get_data(args[0]).clone();
        machine.jump(label.str_identifier().as_str());
    }

}

fn jnz(machine: &mut Machine<Operand>, args: &[usize]) {
    let top = machine.operand_pop();

    if top != Operand::I64(0) {
        let label = machine.get_data(args[0]).clone();
        machine.jump(label.str_identifier().as_str());
    }

}

/* Call */

fn call(machine: &mut Machine<Operand>, args: &[usize]) {
    let label = machine.get_data(args[0]).clone();
    machine.call(label.str_identifier().as_str());
}

fn ret(machine: &mut Machine<Operand>, _args: &[usize]) {
    machine.ret();
}

/* Debug */

fn print_s(machine: &mut Machine<Operand>, _args: &[usize]) {
    let top = machine.operand_stack.peek().clone();

    println!("Top: {:?} (ip: {})", top, machine.ip);
}
