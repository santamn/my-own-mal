use fnv::FnvBuildHasher;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::fmt::{Formatter, Result as FmtResult};
use std::hash::{BuildHasher, Hash, Hasher};
use std::rc::Rc;

use crate::env::Env;
use crate::printer::pr_str;

#[derive(Debug, Clone)]
pub enum MalVal<S = FnvBuildHasher> {
    Nil,
    Bool(bool),
    Number(i64),
    String(Rc<String>),
    Keyword(Rc<String>),
    Symbol(Rc<String>),
    List(Rc<Vec<MalVal>>, Rc<MalVal>),
    Vector(Rc<Vec<MalVal>>, Rc<MalVal>),
    HashMap(Rc<HashMap<MalVal, MalVal, S>>, Rc<MalVal>),
    HashSet(Rc<HashSet<MalVal, S>>, Rc<MalVal>),
    BuiltinFn(fn(Vec<MalVal>) -> MalResult),
    Func(Rc<Closure<S>>, Rc<MalVal>),
}

#[derive(Debug, Clone)]
pub struct Closure<S = FnvBuildHasher> {
    pub params: (Vec<String>, Option<String>),
    pub body: MalVal<S>,
    pub env: Env,
}

pub struct Clojure<I, S>
where
    I: IntoIterator<Item = String>, // MalValにこのジェネリックスが伝播してしまう
    S: BuildHasher + Clone,
{
    pub params: (I, Option<String>, usize),
    pub body: MalVal<S>,
    pub env: Env,
}

impl<S> MalVal<S>
where
    S: BuildHasher + Clone,
{
    pub fn string<T: Into<String>>(str: T) -> Self {
        MalVal::String(Rc::new(str.into()))
    }

    pub fn keyword<T: Into<String>>(str: T) -> Self {
        MalVal::Keyword(Rc::new(str.into()))
    }

    pub fn symbol<T: Into<String>>(str: T) -> Self {
        MalVal::Symbol(Rc::new(str.into()))
    }

    pub fn list(list: Vec<MalVal>) -> Self {
        MalVal::list_with_meta(list, MalVal::Nil)
    }

    pub fn list_with_meta(list: Vec<MalVal>, meta: MalVal) -> Self {
        MalVal::List(Rc::new(list), Rc::new(meta))
    }

    pub fn vec(vec: Vec<MalVal>) -> Self {
        MalVal::vec_with_meta(vec, MalVal::Nil)
    }

    pub fn vec_with_meta(vec: Vec<MalVal>, meta: MalVal) -> Self {
        MalVal::Vector(Rc::new(vec), Rc::new(meta))
    }

    pub fn hashmap(hashmap: HashMap<MalVal, MalVal, S>) -> Self {
        MalVal::hashmap_with_meta(hashmap, MalVal::Nil)
    }

    pub fn hashmap_with_meta(hashmap: HashMap<MalVal, MalVal, S>, meta: MalVal) -> Self {
        MalVal::HashMap(Rc::new(hashmap), Rc::new(meta))
    }

    pub fn hashset(hashset: HashSet<MalVal, S>) -> Self {
        MalVal::hashset_with_meta(hashset, MalVal::Nil)
    }

    pub fn hashset_with_meta(hashset: HashSet<MalVal, S>, meta: MalVal) -> Self {
        MalVal::HashSet(Rc::new(hashset), Rc::new(meta))
    }

    pub fn func_with_meta(closure: Closure<S>, meta: MalVal) -> Self {
        MalVal::Func(Rc::new(closure), Rc::new(meta))
    }

    pub fn func(closure: Closure<S>) -> Self {
        MalVal::func_with_meta(closure, MalVal::Nil)
    }

    pub fn type_str(&self) -> String {
        match self {
            MalVal::Nil => "nil".to_string(),
            MalVal::Bool(_) => "bool".to_string(),
            MalVal::Number(_) => "number".to_string(),
            MalVal::String(_) => "string".to_string(),
            MalVal::Keyword(_) => "keyword".to_string(),
            MalVal::Symbol(_) => "symbol".to_string(),
            MalVal::List(_, _) => "list".to_string(),
            MalVal::Vector(_, _) => "vector".to_string(),
            MalVal::HashMap(_, _) => "hash-map".to_string(),
            MalVal::HashSet(_, _) => "hash-set".to_string(),
            MalVal::BuiltinFn(_) => "function".to_string(),
            MalVal::Func(_, _) => "function".to_string(),
        }
    }
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
            (MalVal::BuiltinFn(a), MalVal::BuiltinFn(b)) => a as *const _ == b as *const _,
            _ => false, // Func同士は常にfalse
        }
    }
}

impl Eq for MalVal {}

impl Hash for MalVal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            MalVal::Nil => 0.hash(state),
            MalVal::Bool(b) => b.hash(state),
            MalVal::Number(n) => n.hash(state),
            MalVal::String(s) => {
                state.write_u8(1);
                s.hash(state)
            }
            MalVal::Keyword(s) => {
                state.write_u8(2);
                s.hash(state)
            }
            MalVal::Symbol(s) => {
                state.write_u8(3);
                s.hash(state)
            }
            MalVal::List(l, _) => l.hash(state),
            MalVal::Vector(v, _) => v.hash(state),
            // ref: [集合をハッシュする (Zobrist hashing)](https://trap.jp/post/1594/)
            MalVal::HashMap(m, _) => {
                state.write_usize(m.len());
                state.write_u64(
                    m.iter()
                        .map(|e| m.hasher().hash_one(e))
                        .reduce(|a, b| a ^ b)
                        .unwrap_or(4),
                );
            }
            MalVal::HashSet(s, _) => {
                state.write_usize(s.len());
                state.write_u64(
                    s.iter()
                        .map(|e| s.hasher().hash_one(e))
                        .reduce(|a, b| a ^ b)
                        .unwrap_or(5),
                );
            }
            MalVal::BuiltinFn(f) => state.write_usize(f as *const _ as usize),
            MalVal::Func(f, _) => {
                state.write_usize(f as *const _ as usize);
                f.params.hash(state);
                f.body.hash(state);
            }
        }
    }
}

impl Display for MalVal {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", pr_str(self))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Paren {
    Round,  // ()
    Square, // []
    Curly,  // {}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Arity {
    Fixed(usize),
    Variadic(usize),
    JustOrOneLess(usize),
}

impl Display for Arity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}",
            match self {
                Arity::Fixed(n) => n.to_string(),
                Arity::Variadic(n) => format!("{}+", n),
                Arity::JustOrOneLess(n) => format!("{} or {}", n - 1, n),
            }
        )
    }
}

pub type MalResult = Result<MalVal, MalError>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MalError {
    // Read時のエラー
    NoInput,
    Unbalanced(Paren),
    UncloedQuote,
    // Eval時のエラー
    DividedByZero,
    NotFound(String),
    InvalidType(String, String, String),
    WrongArity(String, Arity, usize),
    InvalidSyntax(String),
}

impl Display for MalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
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
            MalError::DividedByZero => write!(f, "divided by zero"),
            MalError::NotFound(s) => write!(f, "symbol \'{}\' not found", s),
            MalError::InvalidType(name, expected, got) => {
                write!(f, "expected {} for {}, got {}", expected, name, got)
            }
            MalError::WrongArity(name, expected, got) => write!(
                f,
                "wrong number of arguments for {}: expected {}, got {}",
                name, expected, got
            ),
            MalError::InvalidSyntax(s) => write!(f, "invalid syntax: {}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::hash::{Hash, Hasher};

    use super::MalVal;

    use rand::seq::SliceRandom;
    use rand::thread_rng;

    #[test]
    fn test_hash() {
        let v = [
            (MalVal::Nil, MalVal::Bool(false)),
            (MalVal::Bool(false), MalVal::Number(1)),
            (MalVal::Number(1), MalVal::string("hello")),
            (MalVal::string("hello"), MalVal::symbol("+")),
            (MalVal::symbol("+"), MalVal::keyword("key")),
            (MalVal::keyword("key"), MalVal::Nil),
        ];

        for _ in 0..10 {
            let mut c = v.clone();
            let mut rng = thread_rng();
            c.shuffle(&mut rng);
            let rand_map = MalVal::hashmap(c.into_iter().collect());
            let expected_map = MalVal::hashmap(v.clone().into_iter().collect());
            assert_eq!(expected_map, rand_map);

            let mut random_hash = fnv::FnvHasher::default();
            let mut expected_hash = fnv::FnvHasher::default();
            rand_map.hash(&mut expected_hash);
            expected_map.hash(&mut random_hash);
            assert_eq!(
                expected_hash.finish(),
                random_hash.finish(),
                "hashes are not equal"
            );
        }
    }
}
