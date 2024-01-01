use crate::env::Env;
use crate::printer;
use crate::types::{MalError, MalVal};
use itertools::Itertools;

#[macro_export]
macro_rules! int_op {
    ($name:expr, $func:expr) => {
        $crate::types::MalVal::BuiltinFn(|args| {
            args.into_iter()
                .try_reduce(|acc, x| match (acc, x) {
                    ($crate::types::MalVal::Number(acc), $crate::types::MalVal::Number(x)) => {
                        $func(acc, x)
                    }
                    (z, $crate::types::MalVal::Number(_)) | (_, z) => {
                        Err($crate::types::MalError::InvalidType(
                            crate::printer::pr_str(&z, true),
                            "number".to_string(),
                            z.type_str(),
                        ))
                    }
                })?
                .ok_or($crate::types::MalError::WrongArity(
                    $name.to_string(),
                    $crate::types::Arity::Variadic(1),
                    0,
                ))
        })
    };
}

macro_rules! int_cmp {
    ($cmp:expr) => {
        $crate::types::MalVal::BuiltinFn(|args| {
            args.into_iter()
                .tuple_windows()
                .try_fold(true, |acc, (a, b)| match (a, b) {
                    ($crate::types::MalVal::Number(a), $crate::types::MalVal::Number(b)) => {
                        Ok(acc && $cmp(a, b))
                    }
                    (z, MalVal::Number(_)) | (_, z) => Err($crate::types::MalError::InvalidType(
                        $crate::printer::pr_str(&z, true),
                        "number".to_string(),
                        z.type_str(),
                    )),
                })
                .map($crate::types::MalVal::Bool)
        })
    };
}

// 一回しか呼ばれないのでinlineにしておく
#[inline]
pub fn env() -> Env {
    [
        (
            "+".to_string(),
            int_op!("+", |a, b| Ok(MalVal::Number(a + b))),
        ),
        (
            "-".to_string(),
            int_op!("-", |a, b| Ok(MalVal::Number(a - b))),
        ),
        (
            "*".to_string(),
            int_op!("*", |a, b| Ok(MalVal::Number(a * b))),
        ),
        (
            "/".to_string(),
            int_op!("/", |a: i64, b: i64| a
                .checked_div(b)
                .map(MalVal::Number)
                .ok_or(MalError::DividedByZero)),
        ),
        (
            "list".to_string(),
            MalVal::BuiltinFn(|args| Ok(MalVal::list(args))),
        ),
        (
            "list?".to_string(),
            MalVal::BuiltinFn(|args| {
                Ok(MalVal::Bool(matches!(
                    args.get(0),
                    Some(MalVal::List(_, _))
                )))
            }),
        ),
        (
            "empty?".to_string(),
            MalVal::BuiltinFn(|args| {
                Ok(MalVal::Bool(matches!(
                    args.get(0),
                    Some(MalVal::List(v, _)) | Some(MalVal::Vector(v, _)) if v.is_empty()
                )))
            }),
        ),
        (
            "count".to_string(),
            MalVal::BuiltinFn(|args| match args.get(0) {
                None | Some(MalVal::Nil) => Ok(MalVal::Number(0)),
                Some(MalVal::List(list, _)) => Ok(MalVal::Number(list.len() as i64)),
                Some(MalVal::Vector(vec, _)) => Ok(MalVal::Number(vec.len() as i64)),
                Some(MalVal::HashMap(map, _)) => Ok(MalVal::Number(map.len() as i64)),
                Some(MalVal::HashSet(set, _)) => Ok(MalVal::Number(set.len() as i64)),
                Some(z) => Err(MalError::InvalidType(
                    printer::pr_str(z, true),
                    "nil, list, vector, hashmap or hashset".to_string(),
                    z.type_str(),
                )),
            }),
        ),
        (
            "=".to_string(),
            MalVal::BuiltinFn(|args| Ok(MalVal::Bool(args.into_iter().all_equal()))),
        ),
        ("<".to_string(), int_cmp!(|a, b| a < b)),
        ("<=".to_string(), int_cmp!(|a, b| a <= b)),
        (">".to_string(), int_cmp!(|a, b| a > b)),
        (">=".to_string(), int_cmp!(|a, b| a >= b)),
        (
            "pr-str".to_string(),
            // NOTE: joinはDisplay実装を用いてString化を行う
            MalVal::BuiltinFn(|args| Ok(MalVal::string(args.into_iter().join(" ")))),
        ),
        (
            "str".to_string(),
            MalVal::BuiltinFn(|args| {
                Ok(MalVal::string(
                    args.into_iter()
                        .map(|x| printer::pr_str(&x, false))
                        .collect::<String>(),
                ))
            }),
        ),
        (
            "prn".to_string(),
            MalVal::BuiltinFn(|args| {
                args.into_iter()
                    .for_each(|x| println!("{}", printer::pr_str(&x, true)));
                Ok(MalVal::Nil)
            }),
        ),
        (
            "println".to_string(),
            MalVal::BuiltinFn(|args| {
                args.into_iter()
                    .for_each(|x| println!("{}", printer::pr_str(&x, false)));
                Ok(MalVal::Nil)
            }),
        ),
    ]
    .into()
}
