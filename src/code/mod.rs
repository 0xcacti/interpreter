pub mod error;
use self::error::CodeError;

pub type Opcode = u8;

pub type Instructions = Vec<Opcode>;

pub struct Definition {
    pub name: String,
    pub operand_widths: Vec<u64>,
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

pub fn make(op: Opcode, operands: Vec<u64>) -> Instructions {
    let definition = op.definition().unwrap();
    let length: u64 = definition.operand_widths.into_iter().sum();
    let mut instructions = Vec::new();
    instructions[0] = op;
    return instructions;
}

#[cfg(test)]
mod test {
    use super::*;

    fn check(opcode: Opcode, operands: Vec<u64>, expected: Vec<u8>) {
        let instruction = make(opcode, operands);
        assert_eq!(instruction.len(), expected.len());
        for (i, b) in expected.iter().enumerate() {
            assert_eq!(instruction[i], *b);
        }
    }

    #[test]
    fn it_makes_correctly() {
        check(0, vec![0], vec![0]);
        assert!(false);
    }
}
