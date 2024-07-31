use fnv::FnvHashMap;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::Debug;
use std::iter;
use std::rc::Rc;

use crate::types::MalVal;

#[derive(Debug, Clone, PartialEq, Eq)]
struct EnvEntity {
    outer: Option<Env>,
    table: FnvHashMap<String, MalVal>,
}

// TODO: RefCellいらない?
#[derive(Clone, PartialEq, Eq)]
pub struct Env(Rc<RefCell<EnvEntity>>);

impl Debug for Env {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let env = RefCell::borrow(self.0.borrow());
        write!(f, "{{table: {:?}, outer: {:?}}}", env.table, env.outer)
    }
}

impl Env {
    pub fn new(outer: Option<&Env>) -> Self {
        Env(Rc::new(RefCell::new(EnvEntity {
            outer: outer.cloned(),
            table: FnvHashMap::default(),
        })))
    }

    /// 関数の仮引数と実引数を受け取り、新たな環境を作成する
    pub fn with_bind<I, J>(
        outer: Option<&Env>,
        params: I,                // 仮引数
        variadic: Option<String>, // 可変長引数
        args: J,                  //　実引数
    ) -> Self
    where
        I: Iterator<Item = String> + ExactSizeIterator,
        J: Iterator<Item = MalVal> + Clone,
    {
        if let Some(var) = variadic {
            let len = params.len();
            Env(Rc::new(RefCell::new(EnvEntity {
                outer: outer.cloned(),
                table: iter::zip(params, args.clone().take(len))
                    .chain(iter::once((var, MalVal::list(args.skip(len).collect()))))
                    .collect(),
            })))
        } else {
            Env(Rc::new(RefCell::new(EnvEntity {
                outer: outer.cloned(),
                table: iter::zip(params, args).collect(),
            })))
        }
    }

    pub fn get<K>(&self, key: &K) -> Option<MalVal>
    where
        K: Borrow<str>,
    {
        let env = RefCell::borrow(self.0.borrow());
        match env.table.get(key.borrow()) {
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
        self.0.borrow_mut().table.insert(key.into(), val);
    }

    pub fn global(&self) -> Self {
        // 最も外側の環境を返す
        if let Some(outer) = &self.0.borrow_mut().outer {
            outer.global()
        } else {
            self.clone()
        }
    }
}

impl<const N: usize> From<[(String, MalVal); N]> for Env {
    fn from(arr: [(String, MalVal); N]) -> Self {
        Env(Rc::new(RefCell::new(EnvEntity {
            outer: None,
            table: arr.into_iter().collect(),
        })))
    }
}
