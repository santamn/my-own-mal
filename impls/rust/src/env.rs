use fnv::FnvHashMap;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;

use crate::types::MalVal;

#[derive(Debug, Clone)]
struct EnvEntity {
    outer: Option<Env>,
    data: FnvHashMap<String, MalVal>,
}

#[derive(Debug, Clone)]
pub struct Env(Rc<RefCell<EnvEntity>>);

impl Env {
    pub fn new(outer: Option<&Env>) -> Self {
        Env(Rc::new(RefCell::new(EnvEntity {
            outer: outer.cloned(),
            data: FnvHashMap::default(),
        })))
    }

    pub fn get<K>(&self, key: &K) -> Option<MalVal>
    where
        K: Borrow<str>,
    {
        let env = RefCell::borrow(self.0.borrow());
        match env.data.get(key.borrow()) {
            Some(val) => Some(val.clone()),
            None => match &env.outer {
                Some(outer) => outer.get(key),
                None => None,
            },
        }
    }

    pub fn set<T>(&mut self, key: T, val: MalVal)
    where
        T: Into<String>,
    {
        self.0.borrow_mut().data.insert(key.into(), val);
    }
}
