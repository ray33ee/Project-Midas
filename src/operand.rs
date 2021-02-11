



use std::ops::Add;
use std::ops::Mul;
use std::ops::Sub;
use std::ops::Div;
use std::ops::Rem;

use std::cmp::Ordering;


#[derive(Clone, Debug)]
pub enum Operand {
    I64(i64),
    F64(f64)
}

pub enum OperandPair {
    I642(i64, i64),
    F642(f64, f64)
}

impl Operand {
    //For operands that are used as local variable identifiers, get the identifier. Panic if not identifier
    pub fn identifier(&self) -> i64 {
        match &self {
            Operand::I64(val) => { val.clone() },
            _ => {
                panic!("Operand is not an identifier");
            }
        }
    }

    //Convert the numeric identifier to a string identifier
    pub fn str_identifier(&self) -> String {
        self.identifier().to_string()
    }

    //Convert two operands (possibly of different types) into a pair of the same type
    fn pair(&self, rhs: &Self) -> OperandPair {
        match &self {
            Operand::I64(num1) => {
                match rhs {
                    Operand::I64(num2) => {
                        OperandPair::I642(*num1, *num2)
                    },
                    Operand::F64(num2) => {
                        OperandPair::F642(*num1 as f64, *num2)
                    }
                }
            },
            Operand::F64(num1) => {
                match rhs {
                    Operand::I64(num2) => {
                        OperandPair::F642(*num1, *num2 as f64)
                    },
                    Operand::F64(num2) => {
                        OperandPair::F642(*num1, *num2)
                    }
                }
            }
        }
    }
}

impl Add for Operand {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let pair = self.pair(&other);

        match pair {
            OperandPair::I642(a, b) => {
                Operand::I64(a + b)
            },
            OperandPair::F642(a, b) => {
                Operand::F64(a + b)
            }
        }
    }
}

impl Mul for Operand {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let pair = self.pair(&other);

        match pair {
            OperandPair::I642(a, b) => {
                Operand::I64(a * b)
            },
            OperandPair::F642(a, b) => {
                Operand::F64(a * b)
            }
        }
    }
}

impl Sub for Operand {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let pair = self.pair(&other);

        match pair {
            OperandPair::I642(a, b) => {
                Operand::I64(a - b)
            },
            OperandPair:: F642(a, b) => {
                Operand::F64(a - b)
            }
        }
    }
}

impl Div for Operand {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        let pair = self.pair(&other);

        match pair {
            OperandPair::I642(a, b) => {
                Operand::I64(a / b)
            },
            OperandPair::F642(a, b) => {
                Operand::F64(a / b)
            }
        }
    }
}

impl Rem for Operand {
    type Output = Self;

    fn rem(self, other: Self) -> Self {
        let pair = self.pair(&other);

        match pair {
            OperandPair::I642(a, b) => {
                Operand::I64(a % b)
            },
            OperandPair::F642(a, b) => {
                Operand::F64(a % b)
            }
        }
    }
}

impl PartialOrd for Operand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let pair = self.pair(&other);

        match pair {
            OperandPair::I642(a, b) => {
                a.partial_cmp(&b)
            },
            OperandPair::F642(a, b) => {
                a.partial_cmp(&b)
            }
        }
    }
}

impl PartialEq for Operand {
    fn eq(&self, other: &Self) -> bool {
        let pair = self.pair(&other);

        match pair {
            OperandPair::I642(a, b) => {
                a == b
            },
            OperandPair::F642(a, b) => {
                a == b
            }
        }
    }
}
