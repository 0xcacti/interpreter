pub mod error;
use std::{fmt::Display, io::Cursor};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Opcode {
    Constant,
}

pub struct Definition {
    name: &'static str,
    operand_widths: Vec<usize>,
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

pub fn lookup(op: u8) -> Option<Definition> {
    match op {
        0 => Some(Definition {
            name: "OpConstant",
            operand_widths: vec![2],
        }),
        _ => None,
    }
}

pub fn format_instruction(def: &Definition, operands: &[usize]) -> String {
    let operand_count = def.operand_widths.len();
    if operands.len() != operand_count {
        return format!({"ERROR: operand len {} does not match defined {}\n", operands.len(), operand_count}).to_string();
    }
    match operand_count{
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
            let (operands, n) = read_operands(&def, &self.0[i+1..]);
            write!(f, "{:04} {}\n", i, format_instruction(&def, operands));
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

pub fn read_operands(def: &Definition, instructions: &[u8]) -> (Vec<int>, int){
    let mut operands: Vec<usize> = Vec::with_capacity(def.operand_widths.len());
    let mut offset = 0;

    for  width in def.operand_widths.iter() {

        match width {
            2 => {
                let bytes = instructions[offset..offset+2].to_vec();
                operands.push(u16::from_be_bytes([bytes[0], bytes[1]]) as usize);
            }
            _ => panic!("invalid operand width"),
        }

        offset = offset + width
    }
    return (operands, offset)
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
    fn it_reads_operands_correctly() {
        struct OperandTest {
            opcode: Opcode,
            operands: Vec<int>,
            bytes_read: int,
        }
        let tests = vec![OperandTest {
            opcode: Opcode::Constant,
            operands: vec![65535],
            bytes_read: 2,
        }];

        for test in tests {
            let definition = lookup(test.into()).unwrap();
            let (operands_read, n) = read_operands(definition, instruction[1:]);
            if n != test.bytes_read {
                panic!("n wrong");
            }
            for (i, want) in test.operands.iter().enumerate() {
                if operands_read[i] != want {
                    panic!(format!("operand wrong. Want {}, got {}", want, operands_read[i]));
                }
            }

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
