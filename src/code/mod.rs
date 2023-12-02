pub mod error;
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
    let def = op.definition().unwrap();
    let length: usize = (def.operand_widths.iter().sum::<usize>()) + 1;
    let mut instructions = Vec::new();
    instructions.resize_with(length, Default::default);
    instructions[0] = op;
    let mut offset = 1;
    for (i, o) in operands.iter().enumerate() {
        let width = def.operand_widths[i];
        match width {
            2 => {
                instructions[offset + 1] = *o as u8;
                instructions[offset + 2] = (*o >> 8) as u8;
            }
            _ => panic!("invalid operand width"),
        }
        offset += width
    }

    return instructions;
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
        let tests = (0, vec![65543], vec![0, 255, 254]);
        check(tests.0, tests.1, tests.2);
    }
}
