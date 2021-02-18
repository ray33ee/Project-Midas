use stack_vm::{InstructionTable, Instruction, Machine, Code, Builder, WriteManyTable};

use crate::operand::Operand;



use serde::{Serialize, Deserialize};
use std::convert::From;
use std::convert::Into;

use std::collections::HashMap;


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
    //Data transfer
    instruction_table.insert(Instruction::new(0, "pushl", 1, push_l));
    instruction_table.insert(Instruction::new(1, "pushv", 1, push_v));
    instruction_table.insert(Instruction::new(2, "pushc", 1, push_c));

    instruction_table.insert(Instruction::new(10, "pushdl", 1, push_d_l));
    instruction_table.insert(Instruction::new(11, "pushds", 0, push_d_s));
    instruction_table.insert(Instruction::new(12, "pushdv", 1, push_d_v));

    instruction_table.insert(Instruction::new(20, "popv", 1, pop_v));

    instruction_table.insert(Instruction::new(30, "movl", 2, mov_l));
    instruction_table.insert(Instruction::new(31, "movv", 2, mov_v));
    instruction_table.insert(Instruction::new(32, "movc", 2, mov_c));

    instruction_table.insert(Instruction::new(40, "movds", 1, mov_d_s));
    instruction_table.insert(Instruction::new(41, "movdl", 2, mov_d_l));
    instruction_table.insert(Instruction::new(42, "movdv", 2, mov_d_v));

    instruction_table.insert(Instruction::new(50, "copy", 0, copy));

    //Function calls
    instruction_table.insert(Instruction::new(100, "call", 1, call));
    instruction_table.insert(Instruction::new(101, "ret", 0, ret));

    //Control Flow
    instruction_table.insert(Instruction::new(200, "jmp", 1, jmp));

    instruction_table.insert(Instruction::new(210, "jz", 1, jz));
    instruction_table.insert(Instruction::new(211, "jnz", 1, jnz));

    instruction_table.insert(Instruction::new(220, "abort", 0, abort));

    //Binary operations
    instruction_table.insert(Instruction::new(300, "sub", 0, sub));
    instruction_table.insert(Instruction::new(301, "add", 0, add));

    //Unary operations
    instruction_table.insert(Instruction::new(400, "inc", 0, inc));
    instruction_table.insert(Instruction::new(401, "dec", 0, dec));
    instruction_table.insert(Instruction::new(402, "div", 0, div));
    instruction_table.insert(Instruction::new(403, "mod", 0, modulo));

    //Debug instructions
    instruction_table.insert(Instruction::new(500, "print_s", 0, print_s));
    instruction_table.insert(Instruction::new(501, "print_v", 1, print_v));

    //Comparative instructions
    instruction_table.insert(Instruction::new(600, "eq", 0, eq));
    instruction_table.insert(Instruction::new(601, "ne", 0, ne));

    instruction_table.insert(Instruction::new(610, "lt", 0, lt));
    instruction_table.insert(Instruction::new(611, "gt", 0, gt));
    instruction_table.insert(Instruction::new(612, "le", 0, le));
    instruction_table.insert(Instruction::new(613, "ge", 0, ge));
    instruction_table
}

pub fn get_constants() -> Constants {
    use std::f64::consts;

    use stack_vm::Table;


    let mut constants = WriteManyTable::new();

    constants.insert("0", Operand::F64(consts::PI));                    //Pi
    constants.insert("1", Operand::F64(consts::FRAC_1_PI));             //1/pi
    constants.insert("2", Operand::F64(consts::FRAC_1_PI * 0.5f64));    //1/(2pi)
    constants.insert("3", Operand::F64(consts::FRAC_PI_2));             //pi/2
    constants.insert("4", Operand::F64(2.0f64 * consts::PI));           //2pi
    constants.insert("5", Operand::F64(consts::E));                     //e
    constants.insert("6", Operand::F64(consts::SQRT_2));                //sqrt(2)
    constants.insert("7", Operand::F64(consts::FRAC_1_SQRT_2));         //1/sqrt(2)
    constants.insert("8", Operand::F64(consts::LN_2));                  //ln(2)
    constants.insert("9", Operand::F64(consts::LN_10));                 //ln(10)
    constants.insert("10", Operand::F64(consts::LOG2_10));               //log_2(10)
    constants.insert("11", Operand::F64(consts::LOG2_E));               //log_2(e)
    constants.insert("12", Operand::F64(consts::LOG10_2));              //log_10(2)
    constants.insert("13", Operand::F64(consts::LOG10_E));              //log_10(e)

    constants
}

pub fn get_constants_map() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();

    map.insert("PI", "0");
    map.insert("FRAC_1_PI", "1");
    map.insert("FRAC_1_2PI", "2");
    map.insert("FRAC_PI_2", "3");
    map.insert("2PI", "4");
    map.insert("E", "5");
    map.insert("SQRT_2", "6");
    map.insert("FRAC_1_SQRT_2", "7");
    map.insert("LN_2", "8");
    map.insert("LN_10", "9");
    map.insert("LOG2_10", "10");
    map.insert("LOG2_E", "11");
    map.insert("LOG10_2", "12");
    map.insert("LOG10_E", "13");

    map
}

/* Data Transfer */

//Push a literal on to the stack
//push LITERAL
fn push_l(machine: &mut Machine<Operand>, args: &[usize]) {
    let arg = *machine.get_data(args[0]);
    machine.operand_push(arg);
}

//Push the contents of a local variable on the stack
//push $VARIABLE
fn push_v(machine: &mut Machine<Operand>, args: &[usize]) {
    let operand = *machine.get_data(args[0]);

    let local = *machine.get_local(operand.str_identifier().as_str()).expect("No such local variable");


    machine.operand_push(local);

}

//Push a constant onto the stack
//push .CONSTANT
fn push_c(machine: &mut Machine<Operand>, args: &[usize]) {
    let ind = machine.get_data(args[0]);
    let constant = machine.constants.get(ind.str_identifier().as_str()).expect("Unkown constant");
    machine.operand_push(*constant);
}

//Push an element (addressed via a literal) from the pseudo heap onto the operand stack
//push [LITERAL]
fn push_d_l(machine: &mut Machine<Operand>, args: &[usize]) {
    let ind = *machine.get_data(args[0]);
    let element = machine.heap[ind.identifier() as usize];
    machine.operand_push(element);
}

//Push an element (addressed via the operand stack) from the pseudo heap onto the operand stack
//push [$stack]
fn push_d_s(machine: &mut Machine<Operand>, _args: &[usize]) {
    let ind = machine.operand_pop();
    let element = machine.heap[ind.identifier() as usize];
    machine.operand_push(element);
}

//Push an element (addressed via a variable) from the pseudo heap onto the operand stack
//push [$VARIABLE]
fn push_d_v(machine: &mut Machine<Operand>, args: &[usize]) {
    let ind = *machine.get_data(args[0]);
    let variable = *machine.get_local(ind.str_identifier().as_str()).expect("No such local variable");
    let element = machine.heap[variable.identifier() as usize];
    machine.operand_push(element);
}

//Pop value off stack into local variable
//pop $VARIABLE
fn pop_v(machine: &mut Machine<Operand>, args: &[usize]) {

    let operand = *machine.get_data(args[0]);
    let arg = machine.operand_pop();
    machine.set_local(operand.str_identifier().as_str(), arg);
}

//Move a literal into a local variable
//mov $VARIABLE LITERAL
fn mov_l(machine: &mut Machine<Operand>, args: &[usize]) {
    let operand = *machine.get_data(args[0]);
    let arg = *machine.get_data(args[1]);
    machine.set_local(operand.str_identifier().as_str(), arg);
}

//Move a value from one local variable directly to another
//mov $VARIABLE_1 $VARIABLE_2
fn mov_v(machine: &mut Machine<Operand>, args: &[usize]) {
    let opa = *machine.get_data(args[0]);
    let opb = *machine.get_data(args[1]);
    let local = *machine.get_local(opb.str_identifier().as_str()).expect("No such local variable");

    machine.set_local(opa.str_identifier().as_str(), local);

}

//Move constant into local variable
//mov $VARIABLE .CONSTANT
fn mov_c(machine: &mut Machine<Operand>, args: &[usize]) {
    let variable_ind= *machine.get_data(args[0]);
    let constant_ind = *machine.get_data(args[1]);
    let constant = *machine.constants.get(constant_ind.str_identifier().as_str()).expect("Unknown constant");


    machine.set_local(variable_ind.str_identifier().as_str(), constant);
}

//Move data from heap (addressed with a literal) into local variable
//mov $VARIABLE [LITERAL]
fn mov_d_l(machine: &mut Machine<Operand>, args: &[usize]) {
    let heap_ind = *machine.get_data(args[1]);
    let local_ind = *machine.get_data(args[0]);
    let element = machine.heap[heap_ind.identifier() as usize];

    machine.set_local(local_ind.str_identifier().as_str(), element);
}

//Move data from heap (addressed with the top of the op stack) into local variable
//mov $VARIABLE [$stack]
fn mov_d_s(machine: &mut Machine<Operand>, args: &[usize]) {
    let heap_ind = machine.operand_pop();
    let local_ind = *machine.get_data(args[0]);
    let element = machine.heap[heap_ind.identifier() as usize];

    machine.set_local(local_ind.str_identifier().as_str(), element);
}

//Move data from heap (addressed with local variable) into local variable
//mov $VARIABLE [$VARIABLE]
fn mov_d_v(machine: &mut Machine<Operand>, args: &[usize]) {
    let heap_variable_ind = *machine.get_data(args[1]);
    let heap_variable = *machine.get_local(heap_variable_ind.str_identifier().as_str()).expect("No such local variable");
    let local_ind = *machine.get_data(args[0]);
    let element = machine.heap[heap_variable.identifier() as usize];

    machine.set_local(local_ind.str_identifier().as_str(), element);
}

//Duplicate the top element of the stack
//copy
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

fn div(machine: &mut Machine<Operand>, _args: &[usize]) {
    let rhs = machine.operand_pop();
    let lhs = machine.operand_pop();
    machine.operand_push(lhs / rhs)
}

fn modulo(machine: &mut Machine<Operand>, _args: &[usize]) {
    let rhs = machine.operand_pop();
    let lhs = machine.operand_pop();
    machine.operand_push(lhs % rhs);
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

fn eq(machine: &mut Machine<Operand>, _args: &[usize]) {
    let rhs = machine.operand_pop();
    let lhs = machine.operand_pop();
    machine.operand_push(Operand::I64((lhs == rhs) as i64));
}

fn ne(machine: &mut Machine<Operand>, _args: &[usize]) {
    let rhs = machine.operand_pop();
    let lhs = machine.operand_pop();
    machine.operand_push(Operand::I64((lhs != rhs) as i64));
}

fn lt(machine: &mut Machine<Operand>, _args: &[usize]) {
    let rhs = machine.operand_pop();
    let lhs = machine.operand_pop();
    machine.operand_push(Operand::I64((lhs < rhs) as i64));
}

fn gt(machine: &mut Machine<Operand>, _args: &[usize]) {
    let rhs = machine.operand_pop();
    let lhs = machine.operand_pop();
    machine.operand_push(Operand::I64((lhs > rhs) as i64));
}

fn le(machine: &mut Machine<Operand>, _args: &[usize]) {
    let rhs = machine.operand_pop();
    let lhs = machine.operand_pop();
    machine.operand_push(Operand::I64((lhs <= rhs) as i64));
}

fn ge(machine: &mut Machine<Operand>, _args: &[usize]) {
    let rhs = machine.operand_pop();
    let lhs = machine.operand_pop();
    machine.operand_push(Operand::I64((lhs >= rhs) as i64));
}

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

fn abort(machine: &mut Machine<Operand>, _args: &[usize]) {
    machine.jump("stop");
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
    println!("Stack: {:?}", machine.operand_stack);
}

fn print_v(machine: &mut Machine<Operand>, args: &[usize]) {
    let var_ind = *machine.get_data(args[0]);
    println!("Variable {:?} = {:?}", var_ind, machine.get_local(var_ind.str_identifier().as_str()));
}