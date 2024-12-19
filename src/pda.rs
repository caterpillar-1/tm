use std::{
    collections::{HashMap, HashSet, VecDeque},
    str::FromStr,
};

use crate::parse::{self, parse, valid_state_char, valid_symbol_char, Value};

pub type State = String;
pub type InputSymbol = char;
pub type StackSymbol = char;
// stack top is on the right side
pub type TransL = (State, Option<InputSymbol>, StackSymbol);
pub type TransR = (State, Vec<StackSymbol>);
pub type Trans = (TransL, TransR);

#[derive(Default, Debug)]
pub struct PushDownAutomata {
    Q: HashSet<State>,
    S: HashSet<InputSymbol>,
    G: HashSet<StackSymbol>,
    q0: State,
    z0: StackSymbol,
    F: HashSet<State>,
    delta: HashMap<TransL, TransR>,
}

impl PushDownAutomata {
    pub fn input_valid(&self, s: &str) -> Result<(), usize> {
        for (col, ch) in s.chars().enumerate() {
            if !self.S.contains(&ch) {
                return Err(col);
            }
        }
        Ok(())
    }

    pub fn Q(&self) -> &HashSet<State> {
        &self.Q
    }

    pub fn S(&self) -> &HashSet<InputSymbol> {
        &self.S
    }
    pub fn G(&self) -> &HashSet<StackSymbol> {
        &self.G
    }
    pub fn q0(&self) -> State {
        self.q0.clone()
    }
    pub fn z0(&self) -> StackSymbol {
        self.z0
    }
    pub fn F(&self) -> &HashSet<State> {
        &self.F
    }
    pub fn delta(&self) -> &HashMap<TransL, TransR> {
        &self.delta
    }
    pub fn get(
        &self,
        q: &State,
        a: Option<InputSymbol>,
        X: StackSymbol,
    ) -> Option<(Option<InputSymbol>, &(State, Vec<StackSymbol>))> {
        let mut query = (q.clone(), a, X);
        if let Some(r) = self.delta.get(&query) {
            return Some((query.1, r));
        }
        if matches!(query.1, Some(_)) {
            query.1 = None;
        }
        if let Some(r) = self.delta.get(&query) {
            return Some((query.1, r));
        }
        None
    }
}

#[derive(Debug, Clone)]
pub enum SpecError {
    DeclItem(HashSet<String>),
    Type(String),
    QChar(State, char),
    SChar(char),
    GChar(char),
    MultiCharSymbol(String),
    q0NotInQ,
    z0NotInG,
    TLen(Vec<String>),
    TInvalidState(String),
    TInvalidSymbol(char),
    FNotSubsetQ,
}

#[derive(Debug, Clone)]
pub enum ParseError {
    Syntax(parse::ParseError),
    Spec(SpecError),
}

impl FromStr for PushDownAutomata {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut pda = Self::default();

        let mut c = match parse(s, 5) {
            Ok(c) => c,
            Err(e) => return Err(ParseError::Syntax(e)),
        };

        let decl_items_ref = HashSet::from(["Q", "S", "G", "q0", "z0", "F"]);
        let decl_items_dut = c
            .store
            .iter()
            .map(|kv| kv.0.as_str())
            .collect::<HashSet<_>>();

        if decl_items_dut != decl_items_ref {
            return Err(ParseError::Spec(SpecError::DeclItem(
                decl_items_dut
                    .symmetric_difference(&decl_items_ref)
                    .map(|s| (*s).to_owned())
                    .collect(),
            )));
        }

        for k in decl_items_ref {
            let (k, v) = c.store.remove_entry(k).unwrap();
            fn valid_states(states: &HashSet<String>) -> Result<(), ParseError> {
                for state in states.iter() {
                    for ch in state.chars() {
                        if !valid_state_char(ch) {
                            return Err(ParseError::Spec(SpecError::QChar(state.to_owned(), ch)));
                        }
                    }
                }
                Ok(())
            }
            match k.as_str() {
                "Q" | "S" | "G" | "F" => {
                    if let Value::Set(v) = v {
                        match k.as_str() {
                            "Q" => {
                                if let Err(e) = valid_states(&v) {
                                    return Err(e);
                                }
                                pda.Q = v;
                            }
                            "S" => {
                                for symbol in v.iter() {
                                    if symbol.len() != 1 {
                                        return Err(ParseError::Spec(SpecError::MultiCharSymbol(
                                            symbol.to_owned(),
                                        )));
                                    }
                                    let ch = symbol.chars().nth(0).unwrap();
                                    if !valid_symbol_char(ch) || ch == '_' {
                                        return Err(ParseError::Spec(SpecError::SChar(ch)));
                                    }
                                    pda.S.insert(ch);
                                }
                            }
                            "G" => {
                                for symbol in v.iter() {
                                    if symbol.len() != 1 {
                                        return Err(ParseError::Spec(SpecError::MultiCharSymbol(
                                            symbol.to_owned(),
                                        )));
                                    }
                                    let ch = symbol.chars().nth(0).unwrap();
                                    if !valid_symbol_char(ch) || ch == '_' {
                                        return Err(ParseError::Spec(SpecError::GChar(ch)));
                                    }
                                    pda.G.insert(ch);
                                }
                            }
                            "F" => {
                                if let Err(e) = valid_states(&v) {
                                    return Err(e);
                                }
                                pda.F = v;
                            }
                            _ => panic!(),
                        }
                    } else {
                        return Err(ParseError::Spec(SpecError::Type(k.to_owned())));
                    }
                }
                "q0" | "z0" => {
                    if let Value::Str(v) = v {
                        match k.as_str() {
                            "q0" => pda.q0 = v,
                            "z0" => {
                                if v.len() != 1 {
                                    return Err(ParseError::Spec(SpecError::MultiCharSymbol(v)));
                                }
                                pda.z0 = v.chars().nth(0).unwrap();
                            }
                            _ => panic!(),
                        }
                    } else {
                        return Err(ParseError::Spec(SpecError::Type(k.to_owned())));
                    }
                }
                _ => panic!(),
            }
        }

        if !pda.Q.contains(&pda.q0) {
            return Err(ParseError::Spec(SpecError::q0NotInQ));
        }

        if !pda.G.contains(&pda.z0) {
            return Err(ParseError::Spec(SpecError::z0NotInG));
        }

        if !pda.F.is_subset(&pda.Q) {
            return Err(ParseError::Spec(SpecError::FNotSubsetQ));
        }

        for t in c.trans {
            if let [q, a, X, p, beta] = &t[..] {
                for state in [p, q] {
                    if !pda.Q.contains(state) {
                        return Err(ParseError::Spec(SpecError::TInvalidState(state.to_owned())));
                    }
                }
                for ch in [a, X] {
                    if ch.len() != 1 {
                        return Err(ParseError::Spec(SpecError::MultiCharSymbol(ch.to_owned())));
                    }
                }
                let a = a.chars().nth(0).unwrap();
                let a = match a {
                    '_' => None,
                    a => {
                        if !pda.S.contains(&a) {
                            return Err(ParseError::Spec(SpecError::TInvalidSymbol(a)));
                        }
                        Some(a)
                    }
                };
                let X = X.chars().nth(0).unwrap();
                if X == '_' {
                    return Err(ParseError::Spec(SpecError::TInvalidSymbol(X)));
                }
                let beta = match beta.as_str() {
                    "_" => Vec::new(),
                    beta => beta.to_owned().chars().collect(),
                };
                for ch in &beta {
                    if !pda.G.contains(ch) {
                        return Err(ParseError::Spec(SpecError::TInvalidSymbol(*ch)));
                    }
                }
                pda.delta.insert((q.to_owned(), a, X), (p.to_owned(), beta));
            } else {
                return Err(ParseError::Spec(SpecError::TLen(t)));
            }
        }

        Ok(pda)
    }
}

pub struct ArchState {
    pda: PushDownAutomata,
    step: usize,
    state: State,
    input: VecDeque<InputSymbol>,
    stack: VecDeque<StackSymbol>,
}

#[derive(Debug, Clone)]
pub enum Exception {
    InvalidInput { col: usize },
    Accept,
    Reject,
}

impl ArchState {
    pub fn new(pda: PushDownAutomata, input: &str) -> Result<Self, Exception> {
        let q0 = pda.q0.clone();
        let z0 = pda.z0.clone();
        match pda.input_valid(input) {
            Ok(_) => (),
            Err(col) => return Err(Exception::InvalidInput { col }),
        }
        Ok(ArchState {
            pda,
            step: 0,
            state: q0,
            input: input.chars().collect(),
            stack: VecDeque::from([z0]),
        })
    }

    pub fn step(&mut self) -> Result<(), Exception> {
        let q = &self.state;
        if self.input.is_empty() && self.pda.F().contains(q) {
            return Err(Exception::Accept);
        }
        let a = self.input.front();
        let X = match self.stack.front() {
            Some(X) => *X,
            None => return Err(Exception::Reject),
        };
        if let Some((used, (p, beta))) = self.pda.get(q, a.copied(), X) {
            if used.is_some() {
                self.input.pop_front();
            }
            self.stack.pop_front().unwrap();
            self.state = p.clone();
            for ch in beta.iter().rev() {
                self.stack.push_front(*ch);
            }
            self.step += 1;
            Ok(())
        } else {
            return Err(Exception::Reject);
        }
    }
}

impl std::fmt::Display for ArchState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Step : {}", self.step)?;
        writeln!(f, "State: {}", self.state)?;
        writeln!(f, "Input: {}", self.input.iter().collect::<String>())?;
        writeln!(f, "Stack: {}", self.stack.iter().collect::<String>())
    }
}
