use stack_vm::{Machine, Builder, WriteManyTable, Code};

mod operand;
mod instructions;

use operand::Operand;
use operand::Operand::{I64, F64};

use stack_vm::Table;

use serde::ser::{Serialize, SerializeStruct, Serializer};


fn main() {

    let instruction_table = instructions::get_instructions();

    let mut builder: Builder<Operand> = Builder::new(&instruction_table);
    builder.push("pushc", vec![I64(0)]);
    builder.push("pushl", vec![I64(2)]);
    builder.push("add", vec![]);
    /*builder.label("0");
    builder.push("dec", vec![]);
    builder.push("print_s", vec![]);
    builder.push("copy", vec![]);
    builder.push("jnz", vec![I64(0)]);*/

    let mut constants: WriteManyTable<Operand> = WriteManyTable::new();
    constants.insert("0", F64(3.14159265359));
    constants.insert("1", F64(0.15915494309));
    let mut machine = Machine::new(Code::from(builder), &constants, &instruction_table);
    machine.run();

    println!("Hello, world! {:?}", machine.operand_pop());
}
