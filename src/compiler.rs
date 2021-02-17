//Converts source code into a SerdePartialBuilder object and output to file

use crate::compiler::Argument::Label;
use std::collections::HashMap;

/// # Abstract
/// An algorithm for taking assembly-style source code and compiling it into a valid MidasCM SerdeCode object.
///
/// This algorithm must take into account the several types of arguments (constants, local variables, literals, etc.)
/// and the many combinations of arguments that certain instructions may accept.
///
/// # Four primitive regexes:
/// Here we give four important regexes
///
/// - [IDENTIFIER] Variable, label, constant name or instruction name identifier: https://rgxdb.com/r/1FYIQWRM
/// - [INTEGER] https://rgxdb.com/r/1TRQJALL
/// - [FLOAT] \^[-+]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][-+]?[0-9]+)?$
/// - [COMMENT] #.*$
///
/// # Tokens
/// Combining the four primitive regexes above enables us to match any of the following tokens
///
/// - [LABEL] 'IDENTIFIER:' Used in jumps and function calls.
/// - [CONSTANT] '.IDENTIFIER' Used in constant declaration and uses
/// - [LITERAL] 'INTEGER' or 'FLOAT' Used as a literal argument
/// - [VARIABLE] '$IDENTIFIER' Used as a variable argument
/// - [STACK] '$stack' Used as a stack argument
/// - [INDEX] 'INTEGER' or 'VARIABLE' or 'STACK' Used to address the heap
/// - [HEAP] '[INDEX]' Used to access the heap
/// - [ARGUMENT] 'LITERAL' or 'HEAP' or 'CONSTANT' or 'VARIABLE' or 'STACK' or 'LABEL'
///
/// # Statements
/// Finally, using a combination of tokens we can match the following valid statements
///
/// - [INSTRUCTION] 'IDENTIFIER argument argument argument ... argument'
/// - [LABEL] 'LABEL'
/// - [DECLARATION] 'CONSTANT LITERAL'
/// - [EMPTY] ''
///
/// Any valid statement appended with a [COMMENT] is also a valid statement.
///
/// # Matching
/// Using the definitions above, we can express each token as an enum, with the option of storing extra information
/// (such as the literal value or the index of a constant) within the enum.
/// Next we can identify argument combinations as a vector of arguments. For example, if two variants of the 'add' instruction exist,
/// one that takes two variables and one that takes the $stack identifier, we can represent this with something like
///
/// vec![VARIABLE, VARIABLE]
/// vec![STACK]
///
/// We can then use Rust's powerful pattern matching to map these argument tuples with the exact op code for the instruction
///
/// # Functions
/// - get_statement Returns a Statement enum from a line of code
/// - get_argument returns a single Argument enum from a single argument, called by get_statement
/// - get_identifiers returns two HashMap<&str, &str> one for labels and another for variables


use regex::{Regex, RegexSet};
use crate::operand::Operand;
use crate::compiler::Index::Stack;

#[derive(Debug)]
enum Statement<'a> {
    Instruction(& 'a str, Vec<Argument<'a>>),
    Label(& 'a str),
    Declaration(& 'a str, Operand),
    Empty
}

#[derive(Debug)]
enum Index<'a> {
    I64(i64),
    Variable(& 'a str),
    Stack
}

#[derive(Debug)]
enum Argument<'a> {
    Label(& 'a str),
    Constant(& 'a str),
    Literal(Operand),
    Variable(& 'a str),
    Stack,
    Heap(Index<'a>),
}

fn get_identifiers<'a>(source: & 'a str, regex: &Regex) -> HashMap<& 'a str, String> {
    let mut map = HashMap::new();

    for (i, cap) in regex.captures_iter(source).enumerate() {
        let identifier = cap.get(1).unwrap().as_str();
        if !map.contains_key(identifier) {
            map.insert(identifier, i.to_string());
        }
    }

    map
}

fn do_something() {

    use std::fs::File;
    use std::io::Read;

    // Primitive regexes (as defined above) used as macros (so they can be used with concat!)
    macro_rules! identifier_reg {
        () => {"[a-zA-Z_][a-zA-Z0-9_]*"}
    }

    macro_rules! integer_reg {
        () => {"[-+]?\\d+"}
    }

    macro_rules! float_reg {
        () => {"^[-+]?\\d+(\\.\\d+){1}"}
    }

    lazy_static! {
        static ref LABEL: Regex = Regex::new(concat!(identifier_reg!())).unwrap();
        static ref CONSTANT: Regex = Regex::new(concat!("\\.(", identifier_reg!(), ")")).unwrap();
        static ref LITERAL_INTEGER: Regex = Regex::new(concat!(integer_reg!())).unwrap();
        static ref LITERAL_FLOAT: Regex = Regex::new(concat!(float_reg!())).unwrap();
        static ref VARIABLE: Regex = Regex::new(concat!("\\$(", identifier_reg!(), ")")).unwrap();
        static ref STACK: Regex = Regex::new(concat!("\\$stack")).unwrap();
        static ref HEAP_INTEGER: Regex = Regex::new(concat!("\\[(", integer_reg!(), ")\\]")).unwrap();
        static ref HEAP_VARIABLE: Regex = Regex::new(concat!("\\[\\$(", identifier_reg!(), ")\\]")).unwrap();
        static ref HEAP_STACK: Regex = Regex::new(concat!("\\[\\$stack\\]")).unwrap();

        static ref OPERAND: RegexSet = RegexSet::new(&[
            LABEL.as_str(),
            CONSTANT.as_str(),
            LITERAL_INTEGER.as_str(),
            LITERAL_FLOAT.as_str(),
            VARIABLE.as_str(),
            STACK.as_str(),
            HEAP_INTEGER.as_str(),
            HEAP_VARIABLE.as_str(),
            HEAP_STACK.as_str(),
        ]).unwrap();


        static ref LABEL_DECLARATION: Regex = Regex::new(concat!("(", identifier_reg!(), "):")).unwrap();
        static ref INSTRUCTION: Regex = Regex::new(concat!(identifier_reg!())).unwrap();

        static ref STATEMENT: RegexSet = RegexSet::new(&[
            LABEL_DECLARATION.as_str(),
            CONSTANT.as_str(),
            INSTRUCTION.as_str()
        ]).unwrap();

        static ref LITERAL_OPERAND: RegexSet = RegexSet::new(&[
            LITERAL_INTEGER.as_str(),
            LITERAL_FLOAT.as_str()
        ]).unwrap();
    }


    let mut variable_map = HashMap::new();
    variable_map.insert("var", String::from("0"));

    let mut label_map = HashMap::new();
    label_map.insert("stop", String::from("0"));

    let mut constant_map = HashMap::new();
    constant_map.insert("pi", String::from("0"));

    println!("Statement: {:?}", process_source_line(".pi 3.15",
                                                    &STATEMENT,
                                                    &LABEL_DECLARATION,
                                                    &CONSTANT,
                                                    &OPERAND,
                                                    &CONSTANT,
                                                    &VARIABLE,
                                                    &HEAP_INTEGER,
                                                    &variable_map,
                                                    &label_map,
                                                    &constant_map,
                                                    &LITERAL_OPERAND));


    println!("Argument: {:?}", get_operand("[$stack]",
                                           &OPERAND,
                                           &CONSTANT,
                                           &VARIABLE,
                                           &HEAP_INTEGER,
                                           &variable_map,
                                           &label_map,
                                           &constant_map
    ));

}

fn get_operand<'a>(string: & 'a str,
               operand_regex: &RegexSet,
               constant: &Regex,
               variable: &Regex,
               heap_int: &Regex,
               variable_map: &'a HashMap<& str, String>,
               label_map: &'a HashMap<& str, String>,
               constants_map: &'a HashMap<& str, String>) -> Argument<'a>
{
    let matches: Vec<_> = operand_regex.matches(string).into_iter().collect();

    match matches.as_slice() {
        [0] => {
            Argument::Label(label_map.get(string).unwrap().as_str())
        },
        [0, 1] => { //Constant will also match label
            let constant_str = constant.captures(string).unwrap().get(1).unwrap().as_str();
            Argument::Constant(constants_map.get(constant_str).unwrap().as_str())
        },
        [2] => {
            Argument::Literal(Operand::I64(string.parse::<i64>().unwrap()))
        },
        [2, 3] => { //Floats will also match integers
            Argument::Literal(Operand::F64(string.parse::<f64>().unwrap()))
        },
        [0, 4] => { //Variable will also match label
            let var_str = variable.captures(string).unwrap().get(1).unwrap().as_str();
            Argument::Variable(variable_map.get(var_str).unwrap().as_str())
        },
        [0, 4, 5] => { //Stack will match with variable AND label
            Argument::Stack
        },
        [2, 6] => {
            let st = heap_int.captures(string).unwrap().get(1).unwrap().as_str();
            Argument::Heap(Index::I64(st.parse::<i64>().unwrap()))
        },
        [0, 4, 7] => {
            let var_str = variable.captures(string).unwrap().get(1).unwrap().as_str();
            Argument::Heap(Index::Variable(variable_map.get(var_str).unwrap().as_str()))
        },
        [0, 4, 5, 7, 8] => {//HeapStack will match with stack, variable, heap_variable and label
            Argument::Heap(Index::Stack)
        }
        _ => { panic!(format!("Could not match operand {:?} ({:?})", string, matches)); }
    }

}

fn process_source(source: &str) {

}

fn process_source_line<'a>(source_line: & 'a str,
                           statement: &RegexSet,
                           label_statement: &Regex,
                           constant_statement: &Regex,
                           operand_regex: &RegexSet,
                           constant: &Regex,
                           variable: &Regex,
                           heap_int: &Regex,
                           variable_map: &'a HashMap<& str, String>,
                           label_map: &'a HashMap<& str, String>,
                           constants_map: &'a HashMap<& str, String>,
                           literal_oper: &RegexSet) -> Statement<'a> {
    // Match pattern using whitespace and beginning of string as anchor

    lazy_static! {
        static ref NON_WHITESPACE: Regex = Regex::new("(\\S*)").unwrap();
    }

    let line_as_list: Vec<_> = NON_WHITESPACE.captures_iter(source_line).map(|cap| cap.get(1).unwrap().as_str()).collect();

    let first_in_statement = line_as_list[0];

    let matches: Vec<_> = statement.matches(first_in_statement).into_iter().collect();

    match matches.as_slice() {
        [] => Statement::Empty,
        [0, 2] => {
            let label = label_statement.captures(first_in_statement).unwrap().get(1).unwrap().as_str();
            Statement::Label(label_map.get(label).unwrap().as_str())
        },
        [1, 2] => {
            println!("const: {:?}", line_as_list[1]);
            let constant = constant_statement.captures(first_in_statement).unwrap().get(1).unwrap().as_str();

            let match_literal: Vec<_> = literal_oper.matches(line_as_list[1]).into_iter().collect();

            let literal_number = match match_literal.as_slice() {

                [0] => {
                    Operand::I64(line_as_list[1].parse::<i64>().unwrap())
                },
                [0, 1] => {
                    Operand::F64(line_as_list[1].parse::<f64>().unwrap())
                }

                _ => { panic!(format!("Could not match literal {:?} ({:?})", line_as_list[1], match_literal)); }
            };

            Statement::Declaration(constants_map.get(constant).unwrap().as_str(), literal_number)

        },
        [2] => {

            let mut argumment_vector = Vec::new();

            for s in &line_as_list[1..] {
                argumment_vector.push(get_operand(s, operand_regex, constant, variable, heap_int, variable_map, label_map, constants_map))
            }

            Statement::Instruction(first_in_statement, argumment_vector)
        }

        _ => { panic!(format!("Could not match statement {:?} ({:?})", source_line, matches)); }
    }


}

pub fn test() {

    do_something();
}
