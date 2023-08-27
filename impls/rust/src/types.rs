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

impl MalVal {
    pub fn nil() -> Self {
        MalVal::Nil
    }

    pub fn bool(b: bool) -> Self {
        MalVal::Bool(b)
    }

    pub fn number(n: i64) -> Self {
        MalVal::Number(n)
    }

    pub fn string<T: Into<String>>(str: T) -> Self {
        MalVal::String(str.into())
    }

    pub fn keyword<T: Into<String>>(str: T) -> Self {
        MalVal::Keyword(str.into())
    }

    pub fn symbol<T: Into<String>>(str: T) -> Self {
        MalVal::Symbol(str.into())
    }

    pub fn list(list: LinkedList<MalVal>) -> Self {
        MalVal::list_with_meta(list, MalVal::Nil)
    }

    pub fn list_with_meta(list: LinkedList<MalVal>, meta: MalVal) -> Self {
        MalVal::List(Rc::new(list), Rc::new(meta))
    }

    pub fn vec(vec: Vec<Self>) -> Self {
        Self::vec_with_meta(vec, MalVal::Nil)
    }

    pub fn vec_with_meta(vec: Vec<Self>, meta: Self) -> Self {
        MalVal::Vector(Rc::new(vec), Rc::new(meta))
    }

    pub fn hashmap(hashmap: FnvHashMap<MalVal, MalVal>) -> Self {
        MalVal::hashmap_with_meta(hashmap, MalVal::Nil)
    }

    pub fn hashmap_with_meta(hashmap: FnvHashMap<MalVal, MalVal>, meta: MalVal) -> Self {
        MalVal::HashMap(Rc::new(hashmap), Rc::new(meta))
    }

    pub fn hashset(hashset: FnvHashSet<MalVal>) -> Self {
        MalVal::hashset_with_meta(hashset, MalVal::Nil)
    }

    pub fn hashset_with_meta(hashset: FnvHashSet<MalVal>, meta: MalVal) -> Self {
        MalVal::HashSet(Rc::new(hashset), Rc::new(meta))
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
