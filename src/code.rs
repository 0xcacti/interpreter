pub struct Definition {
    pub name: String,
    pub size: u64,
}
pub enum Opcode {
    OpConstant(u64),
}

pub type Instructions = Vec<Opcode>;
