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

pub type KVStore = HashMap<String, Value>;

#[derive(Default)]
pub struct Config {
    pub store: KVStore,
    pub trans: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Default)]
pub enum ParseErrorType {
    #[default]
    KV,
    Trans,
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ParseError {
    error: ParseErrorType,
    inst: String,
    row: usize,
    col: usize,
    msg: String,
}

fn parse_set(s: &str) -> Option<HashSet<String>> {
    fn valid_item_char(c: char) -> bool {
        c.is_ascii_graphic() && ![',', ';', '{', '}'].contains(&c)
    }

    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    if s.chars().next()? != '{' || s.chars().next_back()? != '}' {
        return None;
    }

    let items: HashSet<String> = s
        .strip_prefix('{')?
        .strip_suffix('}')?
        .split(',')
        .map(|item| item.trim().to_owned())
        .collect();
    match items.iter().all(|item| item.chars().all(valid_item_char)) {
        true => Some(items),
        false => None,
    }
}

fn parse_str(s: &str) -> Option<String> {
    fn valid_str_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    let s = s.trim();

    if s.is_empty() {
        return None;
    }

    match s.chars().all(valid_str_char) {
        true => Some(s.to_owned()),
        false => None,
    }
}

fn parse_trans(s: &str) -> Option<Vec<String>> {
    let segs = s.trim().split_whitespace().map(|s| s.to_owned()).collect();
    Some(segs)
}

pub fn parse(s: &str, nr_trans_item: usize) -> Result<Config, ParseError> {
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

        if inst.chars().nth(0).unwrap() == '#' {
            // KVStore
            let err = Err(ParseError {
                error: ParseErrorType::KV,
                inst: line.to_owned(),
                row,
                ..Default::default()
            });
            let inst = inst.strip_prefix('#').unwrap();
            if let Some((ks, vs)) = inst.split_once('=') {
                let ks = ks.trim();
                let vs = vs.trim();
                let v;
                if let Some(s) = parse_set(vs) {
                    v = Value::Set(s);
                } else if let Some(str) = parse_str(vs) {
                    v = Value::Str(str);
                } else {
                    return err;
                }
                c.store.insert(ks.to_owned(), v);
            } else {
                return err;
            }
        } else {
            // Transition
            let err = Err(ParseError {
                error: ParseErrorType::Trans,
                inst: line.to_owned(),
                row,
                ..Default::default()
            });
            c.trans.push(match parse_trans(inst) {
                None => return err,
                Some(t) => {
                    if t.len() != nr_trans_item {
                        return err;
                    }
                    t
                }
            });
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
