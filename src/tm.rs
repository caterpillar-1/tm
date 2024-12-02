use std::{collections::{HashMap, HashSet}, ops::Index};

pub type State = String;
pub type InputSymbol = char;
pub type TapeSymbol = char;
pub type TransL = (State, Vec<TapeSymbol>);
pub type TransR = (Vec<TapeSymbol>, Vec<Direction>, State);
pub type DeltaType = HashMap<TransL, TransR>;

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
        return Ok(())
    }
    pub fn N(&self) -> usize { self.N }
    pub fn Q(&self) -> &HashSet<State> { &self.Q }
    pub fn S(&self) -> &HashSet<InputSymbol> { &self.S }
    pub fn G(&self) -> &HashSet<TapeSymbol> { &self.G }
    pub fn q0(&self) -> &State { &self.q0 }
    pub fn B(&self) -> TapeSymbol { self.B }
    pub fn F(&self) -> &HashSet<State> { &self.F }
    pub fn delta(&self) -> &DeltaType { &self.delta }

    pub fn get<'a>(&'a self, q: &State, content: &[TapeSymbol]) -> Option<&'a TransR> {
        self.delta.get(&(q.clone(), content.to_owned()))        
    }
}

pub enum ParseErrorType {
    FieldNameNotFound,
    FieldDeclFormatError,
    TransitionDeclFormatError,
    FieldNotFound,
    SetDeclFormatError,
    MultiCharSymbol,
    IntDeclFormatError,
    CharDeclFormatError,
    SpecError,
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
        let insts: Vec<&str> = s.lines().map(|l| {
            match l.split_once(';') {
                Some((code, _)) => code,
                None => l
            }.trim()
        }).collect();
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
                            error: ParseErrorType::FieldDeclFormatError,
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
                        error: ParseErrorType::TransitionDeclFormatError,
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
                tm.delta.insert(
                    (p[0].to_owned(), p[1].chars().collect()),
                    (
                        p[2].chars().collect(),
                        match parse_directions(p[3]) {
                            Ok(v) => v,
                            Err(e) => {
                                return Err(ParseError {
                                    error: ParseErrorType::TransitionDeclFormatError,
                                    pc,
                                    inst: (*inst).to_owned(),
                                    offset: inst.len() - p[4].len(),
                                    ..Default::default()
                                })
                            }
                        },
                        p[4].to_owned(),
                    ),
                );
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
                                        error: ParseErrorType::SetDeclFormatError,
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
                                        error: ParseErrorType::IntDeclFormatError,
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
                                    error: ParseErrorType::CharDeclFormatError,
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
        fn valid(tm: &TuringMachine) -> bool {
            fn valid_Q_state_char(c: char) -> bool {
                c.is_ascii_alphanumeric() || c == '_'
            }
            fn valid_G_symbol(c: &char) -> bool {
                c.is_ascii_graphic() && ![' ', ',', ';', '{', '}', '*', '_'].contains(&c)
            }
            tm.N >= 1
                && tm.Q.is_superset(&tm.F)
                && tm.G.is_superset(&tm.S)
                && tm.Q.contains(&tm.q0)
                && tm.G.contains(&tm.B)
                && tm.Q.iter().all(|s| s.chars().all(valid_Q_state_char))
                && tm.G.iter().all(valid_G_symbol)
                && tm.delta.iter().all(|((q, ots), (nts, dirs, p))| {
                    tm.Q.contains(q)
                        && tm.Q.contains(p)
                        && ots.len() == tm.N
                        && nts.len() == tm.N
                        && dirs.len() == tm.N
                        && ots.iter().all(|c| tm.G.contains(c))
                        && nts.iter().all(|c| tm.G.contains(c))
                })
        }
        match valid(&tm) {
            true => Ok(tm),
            false => Err(ParseError { error: ParseErrorType::SpecError, ..Default::default() })
        }
    }
}
