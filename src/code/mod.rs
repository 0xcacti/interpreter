pub mod error;
use std::{fmt::Display, io::Cursor};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Opcode {
    Constant,
}

struct Instructions(Vec<u8>);

impl Opcode {
    pub fn name(&self) -> &str {
        match self {
            Opcode::Constant => "OpConstant",
        }
    }

    pub fn operand_widths(&self) -> Vec<usize> {
        match self {
            Opcode::Constant => vec![2],
        }
    }
}

impl Display for Instructions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
        let tests = vec![(Opcode::Constant, vec![65534], vec![0, 255, 254])];
        for (opcode, operands, expected) in tests {
            check(opcode, operands, expected);
        }
    }

    #[test]
    fn it_prints_correctly() {
        let instructions = vec![
            make(Opconstant, vec![1]),
            make(Opconstant, vec![2]),
            make(Opconstant, vec![65534]),
        ];

        let expected = r#"0000 OpConstant 1
            0003 OpConstant 2
            0006 OpConstant 65534
        "#;
        let concattenated = instructions.iter().flatten().collect::<Instructions>();
        if concattenated.to_string() != expected {
            panic!("wrong length");
        }
    }
}
