pub mod dfa;
pub mod pda;
pub mod tm;

pub enum Exception {
    Dfa(dfa::Exception),
    Pda(pda::Exception),
    Tm(tm::Exception),
}

pub trait ArchState: std::fmt::Display {
    fn input(&mut self, s: &str) -> Result<(), Exception>;
    fn step(&mut self) -> Result<(), Exception>;
}

pub use pda::ArchState as PdaArchState;
pub use pda::PushDownAutomata;
pub use tm::ArchState as TmArchState;
pub use tm::TuringMachine;
