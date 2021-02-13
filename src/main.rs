mod operand;
mod instructions;
mod host;
mod participant;
mod messages;

extern crate clap;

use clap::{Arg, App};

fn main() {

    let matches = App::new("Midas")
        .version("0.1.2")
        .author("Will Cooper")
        .about("Distributed network based paralell computing system")
        .arg(Arg::with_name("mode")
            .short("m")
            .long("mode")
            .takes_value(true)
            .help("Either host, participant or compile")
            .possible_values(&["host", "participant", "compile"])
            .required(true))
        .get_matches();

    match matches.value_of("mode").unwrap() {
        "host" => {
            let mut host = host::Host::new();
            loop {
                host.check_events();
            }
        },
        "participant" => {
            let mut participant = participant::Participant::new();
            participant.check_events();
        },
        "compile" => println!("Compile mode"),
        _ => unreachable!()
    };

    /*let instruction_table = instructions::get_instructions();

    let mut builder: Builder<Operand> = Builder::new(&instruction_table);
    builder.push("pushc", vec![I64(0)]);
    builder.push("pushl", vec![I64(2)]);
    builder.push("add", vec![]);
    /*builder.label("0");
    builder.push("dec", vec![]);
    builder.push("print_s", vec![]);
    builder.push("copy", vec![]);
    builder.push("jnz", vec![I64(0)]);*/

    let mycode = instructions::SerdeCode::from(builder);

    let mut constants: WriteManyTable<Operand> = WriteManyTable::new();
    constants.insert("0", F64(3.14159265359));
    constants.insert("1", F64(0.15915494309));
    let mut machine = Machine::new(mycode.into(), &constants, &instruction_table);
    machine.run();

    println!("Hello, world! {:?}", machine.operand_pop());*/
}
