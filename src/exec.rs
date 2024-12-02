use crate::tm::*;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct ArchState {
    tm: TuringMachine,
    step: usize,
    state: String,
    tapes: Vec<VecDeque<TapeSymbol>>,
    /// (index (on abstarct tape), offset (on VecDeque))
    heads: Vec<(isize, usize)>,
}

#[derive(Debug, Clone)]
pub enum Exception {
    InvalidInput { input: String, offset: usize },
    RepeatedInput,
    Halt,
    Accept,
}
use Exception::*;

pub type Result = std::result::Result<(), Exception>;

impl ArchState {
    pub fn new(tm: TuringMachine) -> Self {
        let N = tm.N();
        let q0 = tm.q0().clone();
        let mut tapes = Vec::new();
        tapes.reserve_exact(N);
        for _ in 0..N {
            tapes.push(VecDeque::from(['_']));
        }
        Self {
            tm,
            step: 0,
            state: q0,
            tapes,
            heads: vec![(0, 0); N],
        }
    }

    pub fn input(&mut self, s: &str) -> Result {
        if self.step > 0 {
            return Err(RepeatedInput);
        }
        match self.tm.input_valid(s) {
            Ok(()) => self.tapes[0] = VecDeque::from_iter(s.to_owned().chars()),
            Err(offset) => {
                return Err(InvalidInput {
                    input: s.to_owned(),
                    offset,
                })
            }
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result {
        if self.tm.F().contains(&self.state) {
            return Err(Accept);
        }
        match self.tm.get(
            &self.state,
            &self
                .tapes
                .iter()
                .zip(self.heads.iter())
                .map(|(t, (_, off))| t[*off])
                .collect::<Vec<char>>(),
        ) {
            Some((nts, dirs, new_state)) => {
                self.state = new_state.to_owned();
                for i in 0..self.tm.N() {
                    let tape = &mut self.tapes[i];
                    let head = &mut self.heads[i];
                    tape[head.1] = (*nts)[i];

                    let B = self.tm.B();

                    match dirs[i] {
                        Direction::Stay => (),
                        Direction::Left => {
                            if head.1 == 0 {
                                tape.push_front(B);
                            } else {
                                head.1 -= 1;
                            }
                            
                            head.0 -= 1;
                        }
                        Direction::Right => {
                            if head.1 == tape.len() - 1 {
                                tape.push_back(B);
                            } else {
                                head.1 += 1;
                            }

                            head.0 += 1;
                        }
                    }                    

                    while tape.len() > head.1 + 1 && tape.back().unwrap() == &B {
                        tape.pop_back();
                    }

                    while head.1 > 0 && tape.front().unwrap() == &B {
                        tape.pop_front();
                        head.1 -= 1;
                    }
                }
                self.step += 1;
                Ok(())
            }
            None => Err(Halt),
        }
    }

    pub fn result(&self) -> Option<String> {
        if self.tm.F().contains(&self.state) {
            Some(self.tapes[0].iter().collect())
        } else {
            None
        }
    }
}

impl std::fmt::Display for ArchState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Step   : {}", self.step)?;
        for i in 0..self.tm.N() {
            let head = &self.heads[i];
            let tape = &self.tapes[i];
            write!(f, "Index{:<2}: ", i)?;
            for pos in 0..tape.len() {
                write!(f, "{:3}", pos as isize + head.0)?;
            }
            writeln!(f, "")?;
            write!(f, "Tape{:<3}: ", i)?;
            for pos in 0..tape.len() {
                write!(f, "{:3}", tape[pos])?;
            }
            writeln!(f, "")?;
            writeln!(f, "Head{:3}: {}", i, "   ".repeat(head.1) + " ^")?;
        }
        writeln!(f, "State  : {}", self.state)?;
        Ok(())
    }
}
