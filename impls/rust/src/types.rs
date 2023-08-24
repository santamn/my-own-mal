use std::collections::{HashMap, HashSet, LinkedList};
use std::hash::Hash;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum MalVal {
    Nil,
    Bool(bool),
    Number(i64),
    String(String),
    Symbol(String),
    List(Rc<LinkedList<MalVal>>, Rc<MalVal>),
    Vector(Rc<Vec<MalVal>>, Rc<MalVal>),
    HashMap(Rc<HashMap<MalVal, MalVal>>, Rc<MalVal>),
    HashSet(Rc<HashSet<MalVal>>, Rc<MalVal>),
}

impl PartialEq for MalVal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MalVal::Nil, MalVal::Nil) => true,
            (MalVal::Bool(a), MalVal::Bool(b)) => a == b,
            (MalVal::Number(a), MalVal::Number(b)) => a == b,
            (MalVal::String(a), MalVal::String(b)) => a == b,
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
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum MalError {
    ErrString(String),
    Parse(String),
}

pub type MalResult = Result<MalVal, MalError>;
