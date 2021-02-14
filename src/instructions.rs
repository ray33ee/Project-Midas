use stack_vm::{InstructionTable, Instruction, Machine, Code, Builder, WriteManyTable};

use crate::operand::Operand;



use serde::{Serialize, Deserialize};
use std::convert::From;
use std::convert::Into;


//A struct identical to the Builder struct, except it doesnt have the instruction_table field and is serde serialisable
#[derive(Serialize, Deserialize, Debug)]
pub struct SerdeCode<T> {
    symbols: Vec<(usize, String)>,
    code: Vec<usize>,
    data: Vec<T>,
    labels: Vec<(usize, String)>
}

impl<'a, T: PartialEq + std::fmt::Debug> From<Builder<'a, T>> for SerdeCode<T> {
    fn from(item: Builder<'a, T>) -> Self {
        let code = Code::from(item);


        SerdeCode {
            symbols: code.symbols,
            code: code.code,
            data: code.data,
            labels: code.labels
        }
    }
}

impl<T> Into<Code<T>> for SerdeCode<T> {
    fn into(self) -> Code<T> {
        Code {
            symbols: self.symbols,
            code: self.code,
            data: self.data,
            labels: self.labels
        }
    }
}

pub type SerdeCodeOperand = SerdeCode<Operand>;
pub type Instructions = InstructionTable<Operand>;
pub type Constants = WriteManyTable<Operand>;

pub fn get_instructions() -> Instructions {
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
    instruction_table.insert(Instruction::new(16, "pushc", 1, push_c));
    instruction_table.insert(Instruction::new(17, "pushdl", 1, push_d_l));
    instruction_table.insert(Instruction::new(18, "pushds", 0, push_d_s));

    instruction_table.insert(Instruction::new(19, "movds", 1, mov_d_s));
    instruction_table.insert(Instruction::new(20, "movdl", 2, mov_d_l));
    instruction_table.insert(Instruction::new(21, "movdv", 2, mov_d_v));
    instruction_table.insert(Instruction::new(22, "movc", 2, mov_c));
    instruction_table
}

pub fn get_constants() -> Constants {
    use std::f64::consts;

    use stack_vm::Table;


    let mut constants = WriteManyTable::new();

    constants.insert("0", Operand::F64(consts::PI));                    //Pi
    constants.insert("1", Operand::F64(consts::FRAC_1_PI));             //1/pi
    constants.insert("2", Operand::F64(consts::FRAC_1_PI * 0.5f64));    //1/(2pi)
    constants.insert("2", Operand::F64(consts::FRAC_PI_2));             //pi/2
    constants.insert("3", Operand::F64(2.0f64 * consts::PI));           //2pi
    constants.insert("4", Operand::F64(consts::E));                     //e
    constants.insert("5", Operand::F64(consts::SQRT_2));                //sqrt(2)
    constants.insert("6", Operand::F64(consts::FRAC_1_SQRT_2));         //1/sqrt(2)
    constants.insert("7", Operand::F64(consts::LN_2));                  //ln(2)
    constants.insert("8", Operand::F64(consts::LN_10));                 //ln(10)
    constants.insert("9", Operand::F64(consts::LOG2_10));               //log_2(10)
    constants.insert("10", Operand::F64(consts::LOG2_E));               //log_2(e)
    constants.insert("11", Operand::F64(consts::LOG10_2));              //log_10(2)
    constants.insert("12", Operand::F64(consts::LOG10_E));              //log_10(e)

    constants
}

/* Data Transfer */

//Push a literal on to the stack
fn push_l(machine: &mut Machine<Operand>, args: &[usize]) {
    let arg = *machine.get_data(args[0]);
    machine.operand_push(arg);
}

//Push the contents of a local variable on the stack
fn push_v(machine: &mut Machine<Operand>, args: &[usize]) {
    let operand = *machine.get_data(args[0]);

    let local = *machine.get_local(operand.str_identifier().as_str()).expect("No such local variable");


    machine.operand_push(local);

}

fn push_c(machine: &mut Machine<Operand>, args: &[usize]) {
    let ind = machine.get_data(args[0]);
    let constant = machine.constants.get(ind.str_identifier().as_str()).expect("Unkown constant");
    machine.operand_push(*constant);
}

//Push the nth element (literal) from the pseudo heap onto the operand stack
fn push_d_l(machine: &mut Machine<Operand>, args: &[usize]) {
    let ind = *machine.get_data(args[0]);
    let element = *machine.get_local_deep(&format!("d_{}", ind.str_identifier())).expect("Local variable deep search failed.");
    machine.operand_push(element);
}

//Push the nth element (top of stack) from the pseudo heap onto the operand stack
fn push_d_s(machine: &mut Machine<Operand>, _args: &[usize]) {
    let ind = machine.operand_pop();
    let element = *machine.get_local_deep(&format!("d_{}", ind.str_identifier())).expect("Local variable deep search failed.");
    machine.operand_push(element);
}

//Pop value off stack into local variable
fn pop_v(machine: &mut Machine<Operand>, args: &[usize]) {

    let operand = *machine.get_data(args[0]);
    let arg = machine.operand_pop();
    machine.set_local(operand.str_identifier().as_str(), arg);
}

//Move a literal into a local variable
fn mov_l(machine: &mut Machine<Operand>, args: &[usize]) {
    let operand = *machine.get_data(args[0]);
    let arg = *machine.get_data(args[1]);
    machine.set_local(operand.str_identifier().as_str(), arg);
}

//Move a value from one local variable directly to another?
fn mov_v(machine: &mut Machine<Operand>, args: &[usize]) {
    let opa = *machine.get_data(args[0]);
    let opb = *machine.get_data(args[1]);
    let local = *machine.get_local(opb.str_identifier().as_str()).expect("No such local variable");

    machine.set_local(opa.str_identifier().as_str(), local);

}

//Move constant into local variable
fn mov_c(machine: &mut Machine<Operand>, args: &[usize]) {
    let variable_ind= *machine.get_data(args[0]);
    let constant_ind = *machine.get_data(args[1]);
    let constant = *machine.constants.get(constant_ind.str_identifier().as_str()).expect("Unknown constant");


    machine.set_local(variable_ind.str_identifier().as_str(), constant);
}

//Move data from heap (addressed with a literal) into local variable
fn mov_d_l(machine: &mut Machine<Operand>, args: &[usize]) {
    let heap_ind = *machine.get_data(args[1]);
    let local_ind = *machine.get_data(args[0]);
    let data = *machine.get_local_deep(&format!("d_{}", heap_ind.str_identifier())).expect("Unknown constant");

    machine.set_local(local_ind.str_identifier().as_str(), data);
}

//Move data from heap (addressed with the top of the op stack) into local variable
fn mov_d_s(machine: &mut Machine<Operand>, args: &[usize]) {
    let heap_ind = machine.operand_pop();
    let local_ind = *machine.get_data(args[0]);
    let data = *machine.get_local_deep(&format!("d_{}", heap_ind.str_identifier())).expect("Unknown constant");

    machine.set_local(local_ind.str_identifier().as_str(), data);
}

//Move data from heap (addressed with local variable) into local variable
fn mov_d_v(machine: &mut Machine<Operand>, args: &[usize]) {
    let heap_variable_ind = *machine.get_data(args[1]);
    let heap_variable = *machine.get_local(heap_variable_ind.str_identifier().as_str()).expect("No such local variable");
    let local_ind = *machine.get_data(args[0]);
    let data = *machine.get_local_deep(&format!("d_{}", heap_variable.str_identifier())).expect("Unknown constant");

    machine.set_local(local_ind.str_identifier().as_str(), data);
}

fn copy(machine: &mut Machine<Operand>, _args: &[usize]) {
    let top = machine.operand_stack.peek().clone();
    machine.operand_push(top);
}

/* Maths functions */

fn add(machine: &mut Machine<Operand>, _args: &[usize]) {
    let rhs = machine.operand_pop();
    let lhs = machine.operand_pop();
    machine.operand_push(lhs + rhs);
}

fn sub(machine: &mut Machine<Operand>, _args: &[usize]) {
    let rhs = machine.operand_pop();
    let lhs = machine.operand_pop();
    machine.operand_push(lhs - rhs);
}

fn inc(machine: &mut Machine<Operand>, _args: &[usize]) {
    let top = machine.operand_pop();
    machine.operand_push(top + Operand::I64(1));
}

fn dec(machine: &mut Machine<Operand>, _args: &[usize]) {
    let top = machine.operand_pop();
    machine.operand_push(top - Operand::I64(1));
}


/* Compare */



/* Jumps */

fn jmp(machine: &mut Machine<Operand>, args: &[usize]) {
    let label = *machine.get_data(args[0]);
    machine.jump(label.str_identifier().as_str());
}

fn jz(machine: &mut Machine<Operand>, args: &[usize]) {
    let top = machine.operand_pop();

    if top == Operand::I64(0) {
        let label = *machine.get_data(args[0]);
        machine.jump(label.str_identifier().as_str());
    }

}

fn jnz(machine: &mut Machine<Operand>, args: &[usize]) {
    let top = machine.operand_pop();

    if top != Operand::I64(0) {
        let label = *machine.get_data(args[0]);
        machine.jump(label.str_identifier().as_str());
    }

}

/* Call */

fn call(machine: &mut Machine<Operand>, args: &[usize]) {
    let label = *machine.get_data(args[0]);
    machine.call(label.str_identifier().as_str());
}

fn ret(machine: &mut Machine<Operand>, _args: &[usize]) {
    machine.ret();
}

/* Debug */

fn print_s(machine: &mut Machine<Operand>, _args: &[usize]) {
    let top = machine.operand_stack.peek();

    println!("Top: {:?} (ip: {})", top, machine.ip);
}

/* Allocation */

/*fn allocate(machine: &mut Machine<Operand>, _args: &[usize]) {
    for i in 0..10 {
        machine.set_local(i.to_string().as_str(), Operand::I64(0));
    }
}*/

/* Communication */

//Send, receive and Check

//Check: Checks the event queue for comms from the host. POssible commands include:
    //Stop: This can be achieved by jumping to a label at the end of the VM.
    //Pause: Block the VM until