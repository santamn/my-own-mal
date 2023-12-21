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
    pub fn with_bind<I, J>(
        outer: Option<&Env>,
        params: I,
        variadic: Option<String>, // 可変長引数
        args: J,
    ) -> Self
    where
        I: IntoIterator<Item = String> + ExactSizeIterator,
        J: Iterator<Item = MalVal> + ExactSizeIterator + Clone,
    {
        if let Some(var) = variadic
            && args.len() > params.len()
        {
            let len = params.len();
            Env(Rc::new(RefCell::new(EnvEntity {
                outer: outer.cloned(),
                data: iter::zip(
                    params.into_iter().chain(iter::once(var)),
                    args.clone()
                        .take(len)
                        .chain(iter::once(MalVal::vec(args.skip(len).collect()))),
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
