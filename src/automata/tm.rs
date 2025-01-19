use std::collections::{HashSet, VecDeque};

use crate::parse::{parse, valid_state_char, valid_symbol_char, Position, Value};

pub type State = String;
pub type InputSymbol = char;
pub type TapeSymbol = char;
pub type TransL = (State, Vec<TapeSymbol>);
pub type TransR = (Vec<TapeSymbol>, Vec<Direction>, State);
pub type DeltaType = Vec<(TransL, TransR)>;

#[derive(Debug, Clone)]
pub enum Direction {
    Left,
    Right,
    Stay,
}

impl TryFrom<char> for Direction {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'l' => Ok(Direction::Left),
            'r' => Ok(Direction::Right),
            '*' => Ok(Direction::Stay),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Direction::Left => 'l',
                Direction::Right => 'r',
                Direction::Stay => '*',
            }
        )
    }
}

#[derive(Clone, Debug, Default)]
pub struct TuringMachine {
    N: usize,
    Q: HashSet<State>,
    S: HashSet<InputSymbol>,
    G: HashSet<TapeSymbol>,
    q0: State,
    B: TapeSymbol,
    F: HashSet<State>,
    delta: DeltaType,
}

impl TuringMachine {
    pub fn input_valid(&self, input: &str) -> Result<(), usize> {
        for (i, ch) in input.chars().enumerate() {
            if !self.S.contains(&ch) {
                return Err(i);
            }
        }
        Ok(())
    }
    pub fn N(&self) -> usize {
        self.N
    }
    pub fn Q(&self) -> &HashSet<State> {
        &self.Q
    }
    pub fn S(&self) -> &HashSet<InputSymbol> {
        &self.S
    }
    pub fn G(&self) -> &HashSet<TapeSymbol> {
        &self.G
    }
    pub fn q0(&self) -> &State {
        &self.q0
    }
    pub fn B(&self) -> TapeSymbol {
        self.B
    }
    pub fn F(&self) -> &HashSet<State> {
        &self.F
    }
    pub fn delta(&self) -> &DeltaType {
        &self.delta
    }

    pub fn get(&self, q: &State, content: &[TapeSymbol]) -> Option<TransR> {
        if content.len() != self.N {
            return None;
        }
        'outer: for t @ ((_, ots), (nts, dirs, p)) in &self.delta {
            if &t.0 .0 != q {
                continue;
            }
            let mut rnts = Vec::new();
            rnts.reserve_exact(self.N);

            for i in 0..self.N {
                let pat = ots[i];
                let syn = content[i];
                let nsyn = nts[i];

                // if pat == '*' {         // always match
                //     if syn == '_' {
                //         continue 'outer;
                //     }
                // } else {                // try match
                //     if syn != pat {
                //         continue 'outer;
                //     }
                // }

                // // pass matching

                // if nsyn == '*' {        // don't change
                //     rnts.push(syn);
                // } else {                // change
                //     rnts.push(nsyn);
                // }

                if pat == '*' {
                    if syn == self.B {
                        continue 'outer;
                    } else if nsyn == '*' {
                        rnts.push(syn);
                    } else {
                        rnts.push(nsyn);
                    }
                } else if pat == syn {
                    if nsyn == '*' {
                        rnts.push(syn);
                    } else {
                        rnts.push(nsyn);
                    }
                } else {
                    continue 'outer;
                }
            }

            return Some((rnts, dirs.clone(), p.clone()));
        }
        None
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) enum SpecError {
    DeclItem(HashSet<String>),
    MultiCharSymbol(String),
    QChar(char),
    GChar(char),
    SChar(char),
    Type(String),
    FNotSubsetQ,
    SNotSubsetG,
    BNotInG,
    q0NotInQ,
    TLen(Vec<String>),
    TtsLen(Vec<String>),
    TInvalidState(String),
    TInvalidSymbol(char),
    TInvalidDirection,
    TGlob(String, String),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ParseError {
    Syntax(crate::parse::ParseError),
    Spec(SpecError),
}

impl std::str::FromStr for TuringMachine {
    type Err = (Position, ParseError);

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tm = TuringMachine::default();
        let mut c = match parse(s, 5) {
            Ok(c) => c,
            Err(e) => return Err((e.0, ParseError::Syntax(e.1))),
        };

        let decl_items_ref = HashSet::from(["N", "Q", "S", "G", "q0", "B", "F"]);
        let decl_items_dut = c
            .store
            .iter()
            .map(|kv| kv.0.as_str())
            .collect::<HashSet<_>>();

        let pos = Position::default();

        if decl_items_dut != decl_items_ref {
            return Err((
                pos,
                ParseError::Spec(SpecError::DeclItem(
                    decl_items_dut
                        .symmetric_difference(&decl_items_ref)
                        .map(|s| (*s).to_owned())
                        .collect(),
                )),
            ));
        }

        for k in decl_items_ref {
            let (k, (pos, v)) = c.store.remove_entry(k).unwrap();
            fn valid_states(states: &HashSet<String>) -> Result<(), ParseError> {
                for state in states.iter() {
                    for ch in state.chars() {
                        if !valid_state_char(ch) {
                            return Err(ParseError::Spec(SpecError::QChar(ch)));
                        }
                    }
                }
                Ok(())
            }
            match k.as_str() {
                "N" | "B" | "q0" => {
                    if let Value::Str(v) = v {
                        match k.as_str() {
                            "N" => {
                                tm.N = match v.parse::<usize>() {
                                    Ok(v) => v,
                                    Err(_) => {
                                        return Err((pos, ParseError::Spec(SpecError::Type(v))))
                                    }
                                };
                            }
                            "B" => {
                                if v.len() != 1 {
                                    return Err((
                                        pos,
                                        ParseError::Spec(SpecError::MultiCharSymbol(v)),
                                    ));
                                }
                                let ch = v.chars().nth(0).unwrap();
                                if !valid_symbol_char(ch) {
                                    return Err((pos, ParseError::Spec(SpecError::GChar(ch))));
                                }
                                tm.B = ch;
                            }
                            "q0" => {
                                tm.q0 = v;
                            }
                            _ => panic!(),
                        }
                    } else {
                        return Err((pos, ParseError::Spec(SpecError::Type(k.to_owned()))));
                    }
                }
                "Q" | "S" | "G" | "F" => {
                    if let Value::Set(v) = v {
                        match k.as_str() {
                            "Q" => {
                                if let Err(e) = valid_states(&v) {
                                    return Err((pos, e));
                                }
                                tm.Q = v;
                            }
                            "S" => {
                                for symbol in v.iter() {
                                    if symbol.len() != 1 {
                                        return Err((
                                            pos,
                                            ParseError::Spec(SpecError::MultiCharSymbol(
                                                symbol.to_owned(),
                                            )),
                                        ));
                                    }
                                    let ch = symbol.chars().nth(0).unwrap();
                                    if !valid_symbol_char(ch) || ch == '_' {
                                        return Err((pos, ParseError::Spec(SpecError::SChar(ch))));
                                    }
                                    tm.S.insert(ch);
                                }
                            }
                            "G" => {
                                for symbol in v.iter() {
                                    if symbol.len() != 1 {
                                        return Err((
                                            pos,
                                            ParseError::Spec(SpecError::MultiCharSymbol(
                                                symbol.to_owned(),
                                            )),
                                        ));
                                    }
                                    let B = symbol.chars().nth(0).unwrap();
                                    if !valid_symbol_char(B) {
                                        return Err((pos, ParseError::Spec(SpecError::GChar(B))));
                                    }
                                    tm.G.insert(B);
                                }
                            }
                            "F" => {
                                if let Err(e) = valid_states(&v) {
                                    return Err((pos, e));
                                }
                                tm.F = v;
                            }
                            _ => panic!(),
                        }
                    } else {
                        return Err((pos, ParseError::Spec(SpecError::Type(k.to_owned()))));
                    }
                }
                _ => panic!(),
            }
        }

        if tm.B != '_' {
            log::warn!("The blank character B is '{}', not '_'!", tm.B);
        }

        if !tm.Q.contains(&tm.q0) {
            return Err((pos, ParseError::Spec(SpecError::q0NotInQ)));
        }

        if !tm.G.contains(&tm.B) {
            return Err((pos, ParseError::Spec(SpecError::BNotInG)));
        }

        if !tm.F.is_subset(&tm.Q) {
            return Err((pos, ParseError::Spec(SpecError::FNotSubsetQ)));
        }

        if !tm.S.is_subset(&tm.G) {
            return Err((pos, ParseError::Spec(SpecError::SNotSubsetG)));
        }

        for (pos, t) in c.trans {
            if let [q, X_str, Y_str, D, p] = &t[..] {
                if X_str.len() != tm.N || Y_str.len() != tm.N || D.len() != tm.N {
                    return Err((pos, ParseError::Spec(SpecError::TtsLen(t))));
                }
                if !tm.Q.contains(q) {
                    return Err((
                        pos,
                        ParseError::Spec(SpecError::TInvalidState(q.to_owned())),
                    ));
                }
                if !tm.Q.contains(p) {
                    return Err((
                        pos,
                        ParseError::Spec(SpecError::TInvalidState(p.to_owned())),
                    ));
                }

                let direction: Vec<Direction>;
                if let Ok(d) = D.chars().map(|d| d.try_into()).collect() {
                    direction = d;
                } else {
                    return Err((pos, ParseError::Spec(SpecError::TInvalidDirection)));
                }

                let mut X_vec = Vec::new();
                let mut Y_vec = Vec::new();
                let X = X_str.chars();
                let Y = Y_str.chars();

                for (X, Y) in X.zip(Y) {
                    if X != '*' && Y == '*' {
                        return Err((
                            pos,
                            ParseError::Spec(SpecError::TGlob(X_str.to_owned(), Y_str.to_owned())),
                        ));
                    }
                    for ch in [X, Y] {
                        if !tm.G.contains(&ch) && ch != '*' {
                            return Err((pos, ParseError::Spec(SpecError::TInvalidSymbol(ch))));
                        }
                    }
                    X_vec.push(X);
                    Y_vec.push(Y);
                }

                tm.delta
                    .push(((q.to_owned(), X_vec), (Y_vec, direction, p.to_owned())));
            } else {
                return Err((pos, ParseError::Spec(SpecError::TLen(t))));
            }
        }

        Ok(tm)
    }
}

#[derive(Clone, Debug)]
pub struct ArchState {
    tm: TuringMachine,
    step: usize,
    state: String,
    tapes: Vec<VecDeque<TapeSymbol>>,
    /// (index (on abstarct tape), offset (on VecDeque))
    heads: Vec<(isize, usize)>,
    halt: bool,
}

#[derive(Debug, Clone)]
pub(crate) enum Exception {
    InvalidInput { input: String, offset: usize },
    Reject(String),
    Accept(String),
}

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
            halt: false,
        }
    }

    pub fn result(&self) -> Option<String> {
        match self.halt {
            true => {
                let mut tape = self.tapes[0].clone();
                while let Some(c) = tape.front() {
                    if *c == self.tm.B() {
                        tape.pop_front();
                    } else {
                        break;
                    }
                }
                while let Some(c) = tape.back() {
                    if *c == self.tm.B() {
                        tape.pop_back();
                    } else {
                        break;
                    }
                }
                Some(tape.iter().collect())
            }
            false => None,
        }
    }
}

impl super::ArchState for ArchState {
    fn input(&mut self, s: &str) -> Result<(), super::Exception> {
        match self.tm.input_valid(s) {
            Ok(()) => {
                self.tapes[0] = VecDeque::from_iter(s.to_owned().chars());
                if self.tapes[0].is_empty() {
                    self.tapes[0] = VecDeque::from([self.tm.B()])
                }
            }
            Err(offset) => {
                return Err(super::Exception::Tm(Exception::InvalidInput {
                    input: s.to_owned(),
                    offset,
                }))
            }
        }
        Ok(())
    }

    fn step(&mut self) -> Result<(), super::Exception> {
        if self.tm.F().contains(&self.state) {
            self.halt = true;
            return Err(super::Exception::Tm(Exception::Accept(
                self.result().unwrap(),
            )));
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
                self.state = new_state;
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
                            }

                            head.1 += 1;
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
            None => {
                self.halt = true;
                Err(super::Exception::Tm(Exception::Reject(
                    self.result().unwrap(),
                )))
            }
        }
    }
}

impl std::fmt::Display for ArchState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Step   : {}", self.step)?;
        for i in 0..self.tm.N() {
            let head = &self.heads[i];
            let tape = &self.tapes[i];
            let indices: Vec<_> = (0..tape.len())
                .map(|pos| (pos as isize + head.0 - head.1 as isize).abs())
                .collect();
            let widths: Vec<_> = indices
                .iter()
                .map(|idx| idx.to_string().len() + 1)
                .collect();
            write!(f, "Index{:<2}: ", i)?;
            for (pos, width) in indices.iter().zip(widths.iter()) {
                write!(f, "{:<width$}", pos, width = width)?;
            }
            writeln!(f)?;
            write!(f, "Tape{:<3}: ", i)?;
            for (pos, width) in (0..tape.len()).zip(widths.iter()) {
                write!(f, "{:<width$}", tape[pos], width = width)?;
            }
            writeln!(f)?;
            writeln!(
                f,
                "Head{:<3}: {}",
                i,
                " ".repeat(widths[..head.1].iter().sum()) + "^"
            )?;
        }
        writeln!(f, "State  : {}", self.state)?;
        Ok(())
    }
}
