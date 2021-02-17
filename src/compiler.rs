//Converts source code into a SerdePartialBuilder object and output to file
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

use crate::compiler::Argument::Label;
use std::collections::HashMap;

use stack_vm::Builder;
use crate::instructions::get_instructions;


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

#[derive(Debug)]
pub enum Statement<'a> {
    Instruction(& 'a str, Vec<Argument>),
    Label(i64),
    Declaration(i64, Operand),
    Empty
}

#[derive(Debug)]
pub enum Index {
    I64(i64),
    Variable(i64),
    Stack
}

#[derive(Debug)]
pub enum Argument {
    Label(i64),
    Constant(i64),
    Literal(Operand),
    Variable(i64),
    Stack,
    Heap(Index),
}

pub struct Compiler<'a> {
    variable_map: HashMap<& 'a str, i64>,
    label_map: HashMap<& 'a str, i64>,
    constants_map: HashMap<& 'a str, i64>,
    line_iterator: std::str::Lines<'a>
}

impl<'a> Compiler<'a> {

    pub fn new(source: & 'a str) -> Self {
        let constants_map = Self::get_identifiers(source, &CONSTANT);
        Compiler {
            variable_map: Self::get_identifiers(source, & VARIABLE),
            label_map: Self::get_identifiers(source, &LABEL_DECLARATION),
            constants_map: Self::get_identifiers(source, &CONSTANT),
            line_iterator: source.lines()
        }
    }

    fn get_identifiers(source: & 'a str, regex: & 'static Regex) -> HashMap<& 'a str, i64> {
        let mut map = HashMap::new();

        for (i, cap) in regex.captures_iter(source).enumerate() {
            let identifier = cap.get(1).unwrap().as_str();
            if !map.contains_key(identifier) {
                map.insert(identifier, i as i64);
            }
        }

        map
    }

    fn get_consttants_map_len(& self) -> usize {
        self.constants_map.len()
    }

    fn get_operand(& self, string: & 'a str) -> Argument
    {
        let matches: Vec<_> = OPERAND.matches(string).into_iter().collect();

        match matches.as_slice() {
            [0] => {
                Argument::Label(self.label_map.get(string).unwrap().clone())
            },
            [0, 1] => { //Constant will also match label
                let constant_str = CONSTANT.captures(string).unwrap().get(1).unwrap().as_str();
                Argument::Constant(self.constants_map.get(constant_str).unwrap().clone())
            },
            [2] => {
                Argument::Literal(Operand::I64(string.parse::<i64>().unwrap()))
            },
            [2, 3] => { //Floats will also match integers
                Argument::Literal(Operand::F64(string.parse::<f64>().unwrap()))
            },
            [0, 4] => { //Variable will also match label
                let var_str = VARIABLE.captures(string).unwrap().get(1).unwrap().as_str();
                Argument::Variable(self.variable_map.get(var_str).unwrap().clone())
            },
            [0, 4, 5] => { //Stack will match with variable AND label
                Argument::Stack
            },
            [2, 6] => {
                let st = HEAP_INTEGER.captures(string).unwrap().get(1).unwrap().as_str();
                Argument::Heap(Index::I64(st.parse::<i64>().unwrap()))
            },
            [0, 4, 7] => {
                let var_str = VARIABLE.captures(string).unwrap().get(1).unwrap().as_str();
                Argument::Heap(Index::Variable(self.variable_map.get(var_str).unwrap().clone()))
            },
            [0, 4, 5, 7, 8] => {//HeapStack will match with stack, variable, heap_variable and label
                Argument::Heap(Index::Stack)
            }
            _ => { panic!(format!("Could not match operand {:?} ({:?})", string, matches)); }
        }

    }

    fn process_source_line(& self, source_line: & 'a str) -> Statement<'a> {
        // Match pattern using whitespace and beginning of string as anchor

        let line_as_list: Vec<_>  = source_line.split_whitespace().into_iter().collect();

        if line_as_list.is_empty() {
            return Statement::Empty;
        }


        let first_in_statement = line_as_list[0];

        let matches: Vec<_> = STATEMENT.matches(first_in_statement).into_iter().collect();

        match matches.as_slice() {
            [] => Statement::Empty,
            [0, 2] => {
                let label = LABEL_DECLARATION.captures(first_in_statement).unwrap().get(1).unwrap().as_str();
                Statement::Label(self.label_map.get(label).unwrap().clone())
            },
            [1, 2] => {

                if line_as_list.len() != 2 {
                    panic!("Invalid constant declaration. Must be .CONSTANT_NAME INT or FLOAT")
                }

                let constant = CONSTANT.captures(first_in_statement).unwrap().get(1).unwrap().as_str();

                let match_literal: Vec<_> = LITERAL_OPERAND.matches(line_as_list[1]).into_iter().collect();

                let literal_number = match match_literal.as_slice() {

                    [0] => {
                        Operand::I64(line_as_list[1].parse::<i64>().unwrap())
                    },
                    [0, 1] => {
                        Operand::F64(line_as_list[1].parse::<f64>().unwrap())
                    }

                    _ => { panic!(format!("Could not match literal {:?} ({:?})", line_as_list[1], match_literal)); }
                };

                Statement::Declaration(self.constants_map.get(constant).unwrap().clone(), literal_number)

            },
            [2] => {

                let mut argumment_vector = Vec::new();

                for s in &line_as_list[1..] {
                    argumment_vector.push(self.get_operand(s))
                }

                Statement::Instruction(first_in_statement, argumment_vector)
            }

            _ => { panic!(format!("Could not match statement {:?} ({:?})", source_line, matches)); }
        }


    }

}

impl<'a> Iterator for Compiler<'a> {
    type Item = Statement<'a>;

    fn next(& mut self) -> Option<Self::Item> {

        match self.line_iterator.next() {
            Some(line) => {

                let without_comments = line.split("#").next().unwrap();

                Some(self.process_source_line(without_comments))
            },
            None => {
                None
            }
        }

    }

}

pub fn compile_source<'a>(source: &str, instructions: & 'a crate::instructions::Instructions) -> (Builder<'a, Operand>, Vec<Operand>) {
    use std::fs::File;
    use std::io::Read;

    let mut code = Builder::new(instructions);

    let comp = Compiler::new(source);

    let mut constants_list = Vec::with_capacity(comp.get_consttants_map_len());

    for inst in comp {
        //println!("Statement: {:?}", inst);
        match inst {
            Statement::Empty => {},
            Statement::Instruction(name, vector) => {
                match name {
                    "push" => {
                        match vector.as_slice() {
                            [Argument::Literal(lit)] => {
                                code.push("pushl", vec![lit.clone()]);
                            },
                            [Argument::Variable(ind)] => {
                                code.push("pushv", vec![Operand::I64(*ind)]);
                            },
                            [Argument::Constant(ind)] => {
                                code.push("pushc", vec![Operand::I64(*ind)]);
                            },
                            [Argument::Heap(index)] => {
                                match index
                                {
                                    Index::I64(lit) => {
                                        code.push("pushdl", vec![Operand::I64(*lit)]);
                                    } ,
                                    Index::Variable(ind) => {
                                        code.push("pushdv", vec![Operand::I64(*ind)]);
                                    },
                                    Index::Stack => {
                                        code.push("pushds", vec![]);
                                    }
                                }
                            }
                            _ => { panic!("Invalid arguments for 'push' instruction ({:?}", vector); }
                        }
                    },
                    "mov" => {
                        match vector.as_slice() {
                            [Argument::Variable(inda), Argument::Literal(lit)] => {
                                code.push("movl", vec![Operand::I64(*inda), lit.clone()]);
                            },
                            [Argument::Variable(inda), Argument::Variable(ind)] => {
                                code.push("movv", vec![Operand::I64(*inda), Operand::I64(*ind)]);
                            },
                            [Argument::Variable(inda), Argument::Constant(ind)] => {
                                code.push("movc", vec![Operand::I64(*inda), Operand::I64(*ind)]);
                            },
                            [Argument::Variable(inda), Argument::Heap(index)] => {
                                match index
                                {
                                    Index::I64(lit) => {
                                        code.push("movdl", vec![Operand::I64(*inda), Operand::I64(*lit)]);
                                    } ,
                                    Index::Variable(ind) => {
                                        code.push("movdv", vec![Operand::I64(*inda), Operand::I64(*ind)]);
                                    },
                                    Index::Stack => {
                                        code.push("movds", vec![Operand::I64(*inda)]);
                                    }
                                }
                            }
                            _ => { panic!("Invalid arguments for 'mov' instruction ({:?}", vector); }
                        }
                    },
                    "jz" => {
                        match vector.as_slice() {
                            [Argument::Label(ind)] => {
                                code.push("jz", vec![Operand::I64(*ind)]);
                            },
                            _ => { panic!("Invalid arguments for 'jz' instruction ({:?}", vector); }
                        }
                    },
                    "jnz" => {
                        match vector.as_slice() {
                            [Argument::Label(ind)] => {
                                code.push("jnz", vec![Operand::I64(*ind)]);
                            },
                            _ => { panic!("Invalid arguments for 'jnz' instruction ({:?}", vector); }
                        }
                    },
                    "jmp" => {
                        match vector.as_slice() {
                            [Argument::Label(ind)] => {
                                code.push("jmp", vec![Operand::I64(*ind)]);
                            },
                            _ => { panic!("Invalid arguments for 'jmp' instruction ({:?}", vector); }
                        }
                    },
                    "pop" => {
                        match vector.as_slice() {
                            [Argument::Variable(ind)] => {
                                code.push("popv", vec![Operand::I64(*ind)]);
                            },
                            _ => { panic!("Invalid arguments for 'pop' instruction ({:?}", vector); }
                        }
                    },
                    "print" => {
                        match vector.as_slice() {
                            [Argument::Stack] => {
                                code.push("print_s", vec![]);
                            },
                            _ => { panic!("Invalid arguments for 'print' instruction ({:?}", vector); }
                        }
                    }
                    _ => {
                        code.push(name, vec![]);
                    }
                }
            },
            Statement::Declaration(index, literal) => {
                constants_list[index as usize] = literal;
            },
            Statement::Label(index) => {
                code.label(index.to_string().as_str());
            }
        }

    }

    (code, constants_list)

}

pub fn test() {

    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(".\\docs\\sample_code.txt").unwrap();

    let mut source = String::new();

    file.read_to_string(&mut source);

    let ins = get_instructions();

    let (build, _) = compile_source(source.as_str(), &ins);


}