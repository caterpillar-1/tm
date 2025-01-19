use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub enum Value {
    Str(String),
    Set(HashSet<String>),
}

impl Default for Value {
    fn default() -> Self {
        Value::Str(String::new())
    }
}

pub type KVStore = HashMap<String, (Position, Value)>;

#[derive(Default)]
pub struct Config {
    pub store: KVStore,
    pub trans: Vec<(Position, Vec<String>)>,
}

#[derive(Default, Debug, Clone)]
pub enum ParseErrorKV {
    #[default]
    Unknown,
    Str,
    Set,
}

#[derive(Debug, Clone)]
pub enum ParseError {
    KV(ParseErrorKV),
    Trans,
}

#[derive(Clone, Debug, Default)]
pub struct Position {
    pub inst: String,
    pub row: usize,
    pub col: usize,
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "line {}, col {}:", self.row, self.col)?;
        writeln!(f, "{}", self.inst)?;
        writeln!(f, "{}", " ".repeat(self.col) + "^")
    }
}

fn parse_set(line: &str) -> Result<HashSet<String>, (bool, usize)> /* Err(confident, col) */ {
    fn valid_item_char(c: char) -> bool {
        c.is_ascii_graphic() && ![',', ';', '{', '}'].contains(&c)
    }

    let s = line.trim();
    if s.is_empty() {
        return Err((false, 0));
    }

    if !s.starts_with('{') || !s.ends_with('}') {
        return Err((false, 0));
    }

    let items: HashSet<String> = s
        .strip_prefix('{')
        .unwrap()
        .strip_suffix('}')
        .unwrap()
        .split(',')
        .map(|item| item.trim().to_owned())
        .collect();
    match items.iter().all(|item| item.chars().all(valid_item_char)) {
        true => Ok(items),
        false => Err((true, 0)),
    }
}

fn parse_str(s: &str) -> Result<String, usize> {
    fn valid_str_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    let s = s.trim();

    if s.is_empty() {
        return Err(0);
    }

    for (col, c) in s.chars().enumerate() {
        if !valid_str_char(c) {
            return Err(col);
        }
    }
    Ok(s.to_owned())
}

fn parse_trans(s: &str, nr_trans_item: usize) -> Option<Vec<String>> {
    let segs: Vec<_> = s.split_whitespace().map(|s| s.to_owned()).collect();
    if segs.len() != nr_trans_item {
        return None;
    }
    Some(segs)
}

pub fn parse(s: &str, nr_trans_item: usize) -> Result<Config, (Position, ParseError)> {
    let mut c = Config::default();
    for (row, line) in s.lines().enumerate() {
        let inst = match line.split_once(';') {
            Some((code, _)) => code,
            None => line,
        }
        .trim();

        if inst.is_empty() {
            continue;
        }

        let pos = Position {
            inst: line.to_owned(),
            row,
            ..Default::default()
        };
        if inst.starts_with('#') {
            // KVStore
            let inst = inst.strip_prefix('#').unwrap();
            if let Some((ks, vs)) = inst.split_once('=') {
                let margin = ks.len() + 2;
                let ks = ks.trim();
                let vs = vs.trim();
                match parse_set(vs) {
                    Ok(s) => {
                        c.store.insert(ks.to_owned(), (pos, Value::Set(s)));
                        continue;
                    }
                    Err((confident, off)) => {
                        if confident {
                            return Err((
                                Position {
                                    col: margin + off,
                                    ..pos
                                },
                                ParseError::KV(ParseErrorKV::Set),
                            ));
                        }
                    }
                }
                match parse_str(vs) {
                    Ok(s) => {
                        c.store.insert(ks.to_owned(), (pos, Value::Str(s)));
                        continue;
                    }
                    Err(off) => {
                        return Err((
                            Position {
                                col: margin + off,
                                ..pos
                            },
                            ParseError::KV(ParseErrorKV::Str),
                        ))
                    }
                }
            } else {
                return Err((
                    Position { col: 3, ..pos },
                    ParseError::KV(ParseErrorKV::Unknown),
                ));
            }
        } else {
            let t = match parse_trans(inst, nr_trans_item) {
                None => return Err((pos, ParseError::Trans)),
                Some(t) => t,
            };
            // Transition
            c.trans.push((pos, t))
        }
    }

    Ok(c)
}

pub fn valid_state_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

pub fn valid_symbol_char(c: char) -> bool {
    c.is_ascii_graphic() && ![' ', ',', ';', '{', '}', '*'].contains(&c)
}
