use std::collections::{HashMap, HashSet, VecDeque};

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
        return Ok(());
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

    pub fn get<'a>(&'a self, q: &State, content: &[TapeSymbol]) -> Option<TransR> {
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
                    } else {
                        if nsyn == '*' {
                            rnts.push(syn);
                        } else {
                            rnts.push(nsyn);
                        }
                    }
                } else {
                    if pat == syn {
                        if nsyn == '*' {
                            rnts.push(syn);
                        } else {
                            rnts.push(nsyn);
                        }
                    } else {
                        continue 'outer;
                    }
                }
            }

            return Some((rnts, dirs.clone(), p.clone()));
        }
        None
    }
}

#[derive(Debug)]
pub enum SpecError {
    TapeNumber,
    QChar,
    GChar,
    FNotSubsetQ,
    SNotSubsetG,
    BNotInG,
    q0NotInQ,
    TqNotInQ,
    TpNotInQ,
    TLen,
    TtsChar,
    TGlob,
}

impl TuringMachine {
    pub fn validate(&self) -> Result<(), SpecError> {
        fn valid_Q_state_char(c: char) -> bool {
            c.is_ascii_alphanumeric() || c == '_'
        }
        fn valid_G_symbol(c: &char) -> bool {
            c.is_ascii_graphic() && ![' ', ',', ';', '{', '}', '*'].contains(&c)
        }
        if self.N < 1 {
            return Err(SpecError::TapeNumber);
        }

        if !self.Q.is_superset(&self.F) {
            return Err(SpecError::FNotSubsetQ);
        }

        if !self.G.is_superset(&self.S) {
            return Err(SpecError::SNotSubsetG);
        }

        if !self.Q.contains(&self.q0) {
            return Err(SpecError::q0NotInQ);
        }

        if !self.G.contains(&self.B) {
            return Err(SpecError::BNotInG);
        }

        if !self.Q.iter().all(|s| s.chars().all(valid_Q_state_char)) {
            return Err(SpecError::QChar);
        }

        if !self.G.iter().all(valid_G_symbol) {
            return Err(SpecError::GChar);
        }

        for ((q, ots), (nts, dirs, p)) in self.delta.iter() {
            if !self.Q.contains(q) {
                return Err(SpecError::TqNotInQ);
            }
            if !self.Q.contains(p) {
                return Err(SpecError::TpNotInQ);
            }
            if [ots, nts].iter().any(|v| v.len() != self.N) {
                return Err(SpecError::TLen);
            }
            if dirs.len() != self.N {
                return Err(SpecError::TLen);
            }
            if !ots
                .iter()
                .chain(nts.iter())
                .all(|c| *c == '*' || self.G.contains(c))
            {
                return Err(SpecError::TtsChar);
            }
            if ots
                .iter()
                .zip(nts.iter())
                .any(|(o, n)| *o != '*' && *n == '*')
            {
                return Err(SpecError::TGlob);
            }
        }
        Ok(())
    }
}

pub enum ParseErrorType {
    FieldNameNotFound,
    FieldDeclFormat,
    TransitionDeclFormat,
    FieldNotFound,
    SetDeclFormat,
    MultiCharSymbol,
    IntDeclFormat,
    CharDeclFormat,
    Spec(SpecError),
}

pub struct ParseError {
    pub error: ParseErrorType,
    pub pc: usize,
    pub inst: String,
    pub offset: usize,
    pub msg: String,
}

impl Default for ParseError {
    fn default() -> Self {
        Self {
            error: ParseErrorType::FieldNotFound,
            pc: Default::default(),
            inst: Default::default(),
            offset: Default::default(),
            msg: Default::default(),
        }
    }
}

impl std::str::FromStr for TuringMachine {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tm = TuringMachine::default();
        let insts: Vec<&str> = s
            .lines()
            .map(|l| {
                match l.split_once(';') {
                    Some((code, _)) => code,
                    None => l,
                }
                .trim()
            })
            .collect();
        //                    name   pc (line number)  type
        let mut pcs = HashMap::new();
        let fields = ["N", "Q", "S", "G", "q0", "B", "F"];
        for field in fields {
            pcs.insert(field, None);
        }
        for (pc, inst) in insts.iter().enumerate() {
            if inst.is_empty() || inst.chars().next().unwrap() == ';' {
                continue;
            }

            if inst.chars().next().unwrap() == '#' {
                match inst.split_once('=') {
                    Some((name_part, _)) => {
                        let name = name_part.strip_prefix("#").unwrap().trim();
                        match pcs.get_mut(&name) {
                            Some(v) => *v = Some(pc),
                            None => {
                                return Err(ParseError {
                                    error: ParseErrorType::FieldNameNotFound,
                                    pc,
                                    inst: (*inst).to_owned(),
                                    offset: 1,
                                    ..Default::default()
                                })
                            }
                        }
                    }
                    None => {
                        return Err(ParseError {
                            error: ParseErrorType::FieldDeclFormat,
                            pc,
                            inst: (*inst).to_owned(),
                            offset: 0,
                            ..Default::default()
                        })
                    }
                }
            } else {
                let p: Vec<&str> = inst.split_whitespace().collect();
                if p.len() != 5 {
                    return Err(ParseError {
                        error: ParseErrorType::TransitionDeclFormat,
                        pc: pc,
                        inst: (*inst).to_owned(),
                        offset: 0,
                        ..Default::default()
                    });
                }
                fn parse_directions(s: &str) -> Result<Vec<Direction>, usize> {
                    let mut dirs = Vec::new();
                    dirs.reserve_exact(s.len());
                    for (offset, char) in s.chars().enumerate() {
                        match char.try_into() {
                            Ok(d) => dirs.push(d),
                            Err(_) => return Err(offset),
                        }
                    }
                    return Ok(dirs);
                }
                tm.delta.push((
                    (p[0].to_owned(), p[1].chars().collect()),
                    (
                        p[2].chars().collect(),
                        match parse_directions(p[3]) {
                            Ok(v) => v,
                            Err(_) => {
                                return Err(ParseError {
                                    error: ParseErrorType::TransitionDeclFormat,
                                    pc,
                                    inst: (*inst).to_owned(),
                                    offset: inst.len() - p[4].len(),
                                    ..Default::default()
                                })
                            }
                        },
                        p[4].to_owned(),
                    ),
                ));
            }
        }
        for (k, v) in pcs.iter() {
            match v {
                None => {
                    return Err(ParseError {
                        error: ParseErrorType::FieldNotFound,
                        msg: format!("{} is not found", k),
                        ..Default::default()
                    })
                }
                Some(pc) => {
                    let inst = insts[*pc];
                    let value_part = inst.trim().split_once('=').unwrap().1.trim();

                    fn parse_set(s: &str) -> Result<HashSet<&str>, ()> {
                        if let Some(s) = s.trim().strip_prefix('{') {
                            if let Some(s) = s.strip_suffix('}') {
                                return Ok(s.split(',').map(|item| item.trim()).collect());
                            }
                        }
                        Err(())
                    }

                    match *k {
                        "Q" | "F" | "G" | "S" => {
                            let set = match parse_set(value_part) {
                                Ok(s) => s,
                                Err(_) => {
                                    return Err(ParseError {
                                        error: ParseErrorType::SetDeclFormat,
                                        pc: *pc,
                                        inst: inst.to_owned(),
                                        offset: 5,
                                        ..Default::default()
                                    })
                                }
                            };
                            match *k {
                                "Q" | "F" => {
                                    let set: HashSet<String> =
                                        set.into_iter().map(|s| s.to_owned()).collect();
                                    if *k == "Q" {
                                        tm.Q = set;
                                    } else {
                                        tm.F = set;
                                    }
                                }
                                "G" | "S" => {
                                    let set: Option<HashSet<char>> = set
                                        .into_iter()
                                        .map(|s| match s.len() {
                                            1 => Some(s.chars().next().unwrap()),
                                            _ => None,
                                        })
                                        .collect();
                                    match set {
                                        Some(s) => {
                                            if *k == "G" {
                                                tm.G = s;
                                            } else {
                                                tm.S = s;
                                            }
                                        }
                                        None => {
                                            return Err(ParseError {
                                                error: ParseErrorType::MultiCharSymbol,
                                                pc: *pc,
                                                inst: inst.to_owned(),
                                                offset: 5,
                                                ..Default::default()
                                            })
                                        }
                                    }
                                }
                                _ => panic!(),
                            }
                        }
                        "N" => {
                            tm.N = match value_part.parse() {
                                Ok(n) => n,
                                Err(_) => {
                                    return Err(ParseError {
                                        error: ParseErrorType::IntDeclFormat,
                                        pc: *pc,
                                        inst: inst.to_owned(),
                                        offset: 5,
                                        ..Default::default()
                                    })
                                }
                            }
                        }
                        "B" => {
                            if value_part.len() != 1 {
                                return Err(ParseError {
                                    error: ParseErrorType::CharDeclFormat,
                                    pc: *pc,
                                    inst: inst.to_owned(),
                                    offset: 5,
                                    ..Default::default()
                                });
                            }
                            tm.B = value_part.chars().next().unwrap();
                            match tm.B {
                                '_' => (),
                                _ => {
                                    log::warn!("blank symbol is not '_'");
                                }
                            }
                        }
                        "q0" => {
                            tm.q0 = value_part.to_owned();
                        }
                        _ => panic!(),
                    }
                }
            }
        }
        match tm.validate() {
            Ok(_) => Ok(tm),
            Err(e) => Err(ParseError {
                error: ParseErrorType::Spec(e),
                ..Default::default()
            }),
        }
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
pub enum Exception {
    InvalidInput { input: String, offset: usize },
    RepeatedInput,
    Reject,
    Accept,
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

    pub fn input(&mut self, s: &str) -> Result<(), Exception> {
        if self.step > 0 {
            return Err(Exception::RepeatedInput);
        }
        match self.tm.input_valid(s) {
            Ok(()) => {
                self.tapes[0] = VecDeque::from_iter(s.to_owned().chars());
                if self.tapes[0].is_empty() {
                    self.tapes[0] = VecDeque::from([self.tm.B()])
                }
            }
            Err(offset) => {
                return Err(Exception::InvalidInput {
                    input: s.to_owned(),
                    offset,
                })
            }
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), Exception> {
        if self.tm.F().contains(&self.state) {
            self.halt = true;
            return Err(Exception::Accept);
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
                Err(Exception::Reject)
            }
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
            writeln!(f, "")?;
            write!(f, "Tape{:<3}: ", i)?;
            for (pos, width) in (0..tape.len()).zip(widths.iter()) {
                write!(f, "{:<width$}", tape[pos], width = width)?;
            }
            writeln!(f, "")?;
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
