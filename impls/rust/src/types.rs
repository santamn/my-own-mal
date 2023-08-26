use fnv::{FnvHashMap, FnvHashSet};
use std::collections::LinkedList;
use std::fmt::Display;
use std::hash::Hash;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum MalVal {
    Nil,
    Bool(bool),
    Number(i64),
    String(String),
    Keyword(String),
    Symbol(String),
    List(Rc<LinkedList<MalVal>>, Rc<MalVal>),
    Vector(Rc<Vec<MalVal>>, Rc<MalVal>),
    HashMap(Rc<FnvHashMap<MalVal, MalVal>>, Rc<MalVal>),
    HashSet(Rc<FnvHashSet<MalVal>>, Rc<MalVal>),
}

impl PartialEq for MalVal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MalVal::Nil, MalVal::Nil) => true,
            (MalVal::Bool(a), MalVal::Bool(b)) => a == b,
            (MalVal::Number(a), MalVal::Number(b)) => a == b,
            (MalVal::String(a), MalVal::String(b)) => a == b,
            (MalVal::Keyword(a), MalVal::Keyword(b)) => a == b,
            (MalVal::Symbol(a), MalVal::Symbol(b)) => a == b,
            (MalVal::List(a, _), MalVal::List(b, _)) => a == b,
            (MalVal::Vector(a, _), MalVal::Vector(b, _)) => a == b,
            (MalVal::HashMap(a, _), MalVal::HashMap(b, _)) => a == b,
            (MalVal::HashSet(a, _), MalVal::HashSet(b, _)) => a == b,
            _ => false,
        }
    }
}

impl Eq for MalVal {}

impl Hash for MalVal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            MalVal::Nil => 0.hash(state),
            MalVal::Bool(b) => b.hash(state),
            MalVal::Number(n) => n.hash(state),
            MalVal::String(s) => s.hash(state),
            MalVal::Keyword(s) => s.hash(state),
            MalVal::Symbol(s) => s.hash(state),
            MalVal::List(l, _) => l.hash(state),
            MalVal::Vector(v, _) => v.hash(state),
            MalVal::HashMap(m, _) => {
                state.write_usize(m.len());
                for (k, v) in m.iter() {
                    k.hash(state);
                    v.hash(state);
                }
            }
            MalVal::HashSet(s, _) => {
                state.write_usize(s.len());
                for v in s.iter() {
                    v.hash(state);
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Paren {
    Round,  // ()
    Square, // []
    Curly,  // {}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MalError {
    NoInput,
    Unbalanced(Paren),
    UncloedQuote,
    OddMap(usize),
}

impl Display for MalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MalError::NoInput => write!(f, "no input"),
            MalError::Unbalanced(p) => write!(
                f,
                "expected {}, got EOF",
                match p {
                    Paren::Round => ")",
                    Paren::Square => "]",
                    Paren::Curly => "}",
                }
            ),
            MalError::UncloedQuote => write!(f, "expected \", got EOF"),
            MalError::OddMap(n) => write!(f, "odd number of map items: {}", n),
        }
    }
}

pub type MalResult = Result<MalVal, MalError>;
