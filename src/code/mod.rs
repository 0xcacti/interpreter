pub mod error;
use std::ops::{Index, IndexMut};

use std::fmt::{Debug, Display};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Opcode {
    Constant,
    Add,
    Pop,
    Sub,
    Mul,
    Div,
    True,
    False,
    Equal,
    NotEqual,
    GreaterThan,
    Minus,
    Bang,
    JumpNotTruthy,
    Jump,
    Null,
    GetGlobal,
    SetGlobal,
}
impl From<u8> for Opcode {
    fn from(op: u8) -> Opcode {
        match op {
            0 => Opcode::Constant,
            1 => Opcode::Add,
            2 => Opcode::Pop,
            3 => Opcode::Sub,
            4 => Opcode::Mul,
            5 => Opcode::Div,
            6 => Opcode::True,
            7 => Opcode::False,
            8 => Opcode::Equal,
            9 => Opcode::NotEqual,
            10 => Opcode::GreaterThan,
            11 => Opcode::Minus,
            12 => Opcode::Bang,
            13 => Opcode::JumpNotTruthy,
            14 => Opcode::Jump,
            15 => Opcode::Null,
            16 => Opcode::GetGlobal,
            17 => Opcode::SetGlobal,
            _ => panic!("unknown opcode"),
        }
    }
}

impl Into<Instructions> for Vec<u8> {
    fn into(self) -> Instructions {
        Instructions::new(self)
    }
}

pub struct Definition {
    name: &'static str,
    operand_widths: Vec<usize>,
}

#[derive(Clone, PartialEq)]
pub struct Instructions(pub Vec<u8>);

impl Index<usize> for Instructions {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Instructions {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl FromIterator<u8> for Instructions {
    fn from_iter<I: IntoIterator<Item = u8>>(iter: I) -> Self {
        Instructions(iter.into_iter().collect())
    }
}

impl Instructions {
    pub fn new(bytes: Vec<u8>) -> Self {
        Instructions(bytes)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::slice::Iter<u8> {
        self.0.iter()
    }

    pub fn write(&mut self, bytes: Vec<u8>) {
        self.0.extend(bytes);
    }

    pub fn extend(&mut self, instructions: Instructions) {
        self.0.extend(instructions.0);
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn slice(&self, start: usize, end: usize) -> Vec<u8> {
        self.0[start..end].to_vec()
    }

    pub fn slice_from(&self, start: usize, end: usize) -> Vec<u8> {
        self.0[start..end].to_vec()
    }
}

impl Opcode {
    pub fn name(&self) -> &str {
        match self {
            Opcode::Constant => "OpConstant",
            Opcode::Add => "OpAdd",
            Opcode::Pop => "OpPop",
            Opcode::Sub => "OpSub",
            Opcode::Mul => "OpMul",
            Opcode::Div => "OpDiv",
            Opcode::True => "OpTrue",
            Opcode::False => "OpFalse",
            Opcode::Equal => "OpEqual",
            Opcode::NotEqual => "OpNotEqual",
            Opcode::GreaterThan => "OpGreaterThan",
            Opcode::Minus => "OpMinus",
            Opcode::Bang => "OpBang",
            Opcode::JumpNotTruthy => "OpJumpNotTruthy",
            Opcode::Jump => "OpJump",
            Opcode::Null => "OpNull",
            Opcode::GetGlobal => "OpGetGlobal",
            Opcode::SetGlobal => "OpSetGlobal",
        }
    }

    pub fn operand_widths(&self) -> Vec<usize> {
        match self {
            Opcode::Constant => vec![2],
            Opcode::Add => vec![],
            Opcode::Pop => vec![],
            Opcode::Sub => vec![],
            Opcode::Mul => vec![],
            Opcode::Div => vec![],
            Opcode::True => vec![],
            Opcode::False => vec![],
            Opcode::Equal => vec![],
            Opcode::NotEqual => vec![],
            Opcode::GreaterThan => vec![],
            Opcode::Minus => vec![],
            Opcode::Bang => vec![],
            Opcode::JumpNotTruthy => vec![2],
            Opcode::Jump => vec![2],
            Opcode::Null => vec![],
            Opcode::GetGlobal => vec![2],
            Opcode::SetGlobal => vec![2],
        }
    }
}

pub fn lookup(op: u8) -> Option<Definition> {
    match op {
        0 => Some(Definition {
            name: "OpConstant",
            operand_widths: vec![2],
        }),

        1 => Some(Definition {
            name: "OpAdd",
            operand_widths: vec![],
        }),

        2 => Some(Definition {
            name: "OpPop",
            operand_widths: vec![],
        }),

        3 => Some(Definition {
            name: "OpSub",
            operand_widths: vec![],
        }),

        4 => Some(Definition {
            name: "OpMul",
            operand_widths: vec![],
        }),

        5 => Some(Definition {
            name: "OpDiv",
            operand_widths: vec![],
        }),

        6 => Some(Definition {
            name: "OpTrue",
            operand_widths: vec![],
        }),

        7 => Some(Definition {
            name: "OpFalse",
            operand_widths: vec![],
        }),

        8 => Some(Definition {
            name: "OpEqual",
            operand_widths: vec![],
        }),

        9 => Some(Definition {
            name: "OpNotEqual",
            operand_widths: vec![],
        }),

        10 => Some(Definition {
            name: "OpGreaterThan",
            operand_widths: vec![],
        }),

        11 => Some(Definition {
            name: "OpMinus",
            operand_widths: vec![],
        }),

        12 => Some(Definition {
            name: "OpBang",
            operand_widths: vec![],
        }),

        13 => Some(Definition {
            name: "OpJumpNotTruthy",
            operand_widths: vec![2],
        }),

        14 => Some(Definition {
            name: "OpJump",
            operand_widths: vec![2],
        }),

        15 => Some(Definition {
            name: "OpNull",
            operand_widths: vec![],
        }),

        16 => Some(Definition {
            name: "OpGetGlobal",
            operand_widths: vec![2],
        }),

        17 => Some(Definition {
            name: "OpSetGlobal",
            operand_widths: vec![2],
        }),

        _ => None,
    }
}

pub fn format_instruction(def: &Definition, operands: &Vec<usize>) -> String {
    let operand_count = def.operand_widths.len();
    if operands.len() != operand_count {
        return format!(
            "ERROR: operand len {} does not match defined {}\n",
            operands.len(),
            operand_count
        )
        .to_string();
    }
    match operand_count {
        0 => return def.name.to_string(),
        1 => return format!("{} {}", def.name, operands[0]).to_string(),
        _ => return format!("ERROR: unhandled operand_count for {}\n", def.name),
    }
}

impl Display for Instructions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut i = 0;
        while i < self.0.len() {
            let definition = lookup(self.0[i]);
            if definition.is_none() {
                write!(f, "ERROR: undefined opcode {}", self.0[i])?;
                continue;
            }
            let def = definition.unwrap();
            let (operands, n) = read_operands(&def, &self.0[i + 1..]);
            let _ = write!(f, "{:04} {}\n", i, format_instruction(&def, &operands));
            i += n + 1;
        }
        Ok(())
    }
}

impl Debug for Instructions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut i = 0;
        write!(f, "\n")?;
        while i < self.0.len() {
            let definition = lookup(self.0[i]);
            if definition.is_none() {
                write!(f, "ERROR: undefined opcode {}", self.0[i])?;
                continue;
            }
            let def = definition.unwrap();
            let (operands, n) = read_operands(&def, &self.0[i + 1..]);
            let _ = write!(f, "{:04} {}\n", i, format_instruction(&def, &operands));
            i += n + 1;
        }
        Ok(())
    }
}

pub fn make(op: Opcode, operands: Vec<usize>) -> Vec<u8> {
    let length: usize = (op.operand_widths().iter().sum::<usize>()) + 1;
    let mut instructions = Vec::with_capacity(length);
    instructions.push(op as u8);

    for (i, &o) in operands.iter().enumerate() {
        let width = op.operand_widths()[i];
        match width {
            2 => {
                let bytes = (o as u16).to_be_bytes();
                instructions.push(bytes[0]);
                instructions.push(bytes[1]);
            }
            _ => panic!("invalid operand width"),
        }
    }
    instructions
}

pub fn read_operands(def: &Definition, instructions: &[u8]) -> (Vec<usize>, usize) {
    let mut operands: Vec<usize> = Vec::with_capacity(def.operand_widths.len());
    let mut offset = 0;

    for width in def.operand_widths.iter() {
        match width {
            2 => {
                let bytes = instructions[offset..offset + 2].to_vec();
                operands.push(u16::from_be_bytes([bytes[0], bytes[1]]) as usize);
            }
            _ => panic!("invalid operand width"),
        }

        offset = offset + width
    }
    return (operands, offset);
}

pub fn read_u16(instructions: &Instructions, start: usize) -> u16 {
    u16::from_be_bytes([instructions[start], instructions[start + 1]])
}

#[cfg(test)]
mod test {
    use super::*;

    fn check(opcode: Opcode, operands: Vec<usize>, expected: Vec<u8>) {
        let instruction = make(opcode, operands);
        assert_eq!(instruction.len(), expected.len());
        for (i, b) in expected.iter().enumerate() {
            assert_eq!(instruction[i], *b);
        }
    }

    #[test]
    fn it_makes_correctly() {
        let tests = vec![
            (Opcode::Constant, vec![65534], vec![0, 255, 254]),
            (Opcode::Add, vec![], vec![Opcode::Add as u8]),
        ];
        for (opcode, operands, expected) in tests {
            check(opcode, operands, expected);
        }
    }

    #[test]
    fn it_reads_operands_correctly() {
        struct OperandTest {
            opcode: Opcode,
            operands: Vec<usize>,
            bytes_read: usize,
        }
        let tests = vec![OperandTest {
            opcode: Opcode::Constant,
            operands: vec![65535],
            bytes_read: 2,
        }];

        for test in tests {
            let instruction = make(test.opcode, test.operands.clone());
            let definition = lookup(test.opcode as u8).unwrap();
            let (operands_read, n) = read_operands(&definition, &instruction[1..]);

            if n != test.bytes_read {
                panic!("n wrong");
            }
            for (i, want) in test.operands.iter().enumerate() {
                if operands_read[i] != *want {
                    panic!("operand wrong. Want {}, got {}", want, operands_read[i]);
                }
            }
        }
    }

    #[test]
    fn it_prints_correctly() {
        let instructions = vec![
            make(Opcode::Add, vec![]),
            make(Opcode::Constant, vec![2]),
            make(Opcode::Constant, vec![65534]),
        ];

        let expected = r#"0000 OpAdd
0001 OpConstant 2
0004 OpConstant 65534
"#;

        let concattenated = instructions.into_iter().flatten().collect::<Instructions>();
        println!("{}", concattenated);

        println!("{}", expected);

        if concattenated.to_string() != expected {
            panic!(
                "wrong length: expected {}, got {}",
                expected.len(),
                concattenated.to_string().len()
            );
        }
    }
}
