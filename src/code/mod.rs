pub mod error;
use std::io::Cursor;

use byteorder::{BigEndian, WriteBytesExt};

use self::error::CodeError;

pub type Opcode = u8;

pub type Instructions = Vec<Opcode>;

pub struct Definition {
    pub name: String,
    pub operand_widths: Vec<usize>,
}

trait Operation {
    fn definition(&self) -> Result<Definition, CodeError>;
}

impl Operation for Opcode {
    fn definition(&self) -> Result<Definition, CodeError> {
        match self {
            0 => {
                return Ok(Definition {
                    name: "OpConstant".to_string(),
                    operand_widths: vec![2],
                })
            }
            _ => return Err(CodeError::new("opcode not found".to_string())),
        }
    }
}

pub fn make(op: Opcode, operands: Vec<usize>) -> Instructions {
    println!("make");
    println!("{:?}", op);
    println!("{:?}", operands);
    let def = op.definition().unwrap();
    let length: usize = (def.operand_widths.iter().sum::<usize>()) + 1;
    let mut instructions = Vec::with_capacity(length);

    instructions.push(op);
    let mut offset = 1;
    for (i, &o) in operands.iter().enumerate() {
        println!("====================");
        println!("{}: {}", i, o);
        println!("====================");
        let width = def.operand_widths[i];
        match width {
            2 => {
                instructions.push((o >> 8) as u8);
                instructions.push(o as u8);
                println!("{:?}", (o as u16).to_be_bytes());
                // let mut wtr = Cursor::new(&mut instructions[offset..]);
                // wtr.write_u16::<BigEndian>(o as u16).unwrap();

                // instructions.extend_from_slice(&o.to_be_bytes()[..2]);
            }
            _ => panic!("invalid operand width"),
        }
        offset += width
    }
    println!("{:?}", instructions);
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
        let tests = (0, vec![65534], vec![0, 255, 254]);
        check(tests.0, tests.1, tests.2);
    }
}
