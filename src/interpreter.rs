#[derive(Debug)]
pub struct State {
    pub memory: Vec<u8>,
    pub pointer: usize,
    pub input: Vec<u8>,
    pub output: Vec<u8>,
}

const MEMORY_SIZE: usize = 8;

impl State {
    pub fn new(input: Vec<u8>) -> Self {
        Self {
            memory: vec![0; MEMORY_SIZE],
            pointer: 0,
            input,
            output: Vec::new(),
        }
    }
}

pub trait Interpret {
    fn interpret(&mut self, state: &mut State);
}

pub struct Interpreter {
    pub state: State,
}

impl Interpreter {
    pub fn new(input: Vec<u8>) -> Self {
        Self {
            state: State::new(input),
        }
    }

    pub fn interpret(&mut self, program: &mut dyn Interpret) {
        program.interpret(&mut self.state);
    }
}
