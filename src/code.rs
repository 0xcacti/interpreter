pub type Opcode = u8;

pub type Instructions = Vec<Opcode>;

pub struct Definition {
    pub name: String,
    pub operand_widths: Vec<u64>,
}

pub enum Opcodes {
    Constant,
}

impl Opcodes {
    pub fn definition(&self) -> Definition {
        match self {
            Opcodes::Constant => {
                return Definition {
                    name: "OpConstant".to_string(),
                    operand_widths: vec![2],
                }
            }
        }
    }
}

pub fn make(op: Opcode, operands: Vec<u64>) -> Instructions {
    let definition = op.definition()
    let length = op.

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
