use fnv::FnvHashMap;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::iter;
use std::rc::Rc;

use crate::types::MalVal;

#[derive(Debug, Clone)]
struct EnvEntity {
    outer: Option<Env>,
    data: FnvHashMap<String, MalVal>,
}

// TODO: RefCellいらない?
#[derive(Debug, Clone)]
pub struct Env(Rc<RefCell<EnvEntity>>);

impl Env {
    pub fn new(outer: Option<&Env>) -> Self {
        Env(Rc::new(RefCell::new(EnvEntity {
            outer: outer.cloned(),
            data: FnvHashMap::default(),
        })))
    }

    // 関数の仮引数と実引数を受け取り、新たな環境を作成する
    // TODO: paramsとargsをIteratorで受け取る
    pub fn with_bind(
        outer: Option<&Env>,
        params: Vec<String>,
        variadic: Option<String>, // 可変長引数
        mut args: Vec<MalVal>,
    ) -> Self {
        if let Some(var) = variadic
            && args.len() > params.len()
        {
            let rest_args = args.split_off(params.len());
            Env(Rc::new(RefCell::new(EnvEntity {
                outer: outer.cloned(),
                data: iter::zip(
                    params.into_iter().chain(iter::once(var)),
                    args.into_iter().chain(iter::once(MalVal::vec(rest_args))),
                )
                .collect(),
            })))
        } else {
            Env(Rc::new(RefCell::new(EnvEntity {
                outer: outer.cloned(),
                data: iter::zip(params, args).collect(),
            })))
        }
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
