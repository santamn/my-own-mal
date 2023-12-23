use crate::env::Env;
use crate::printer;
use crate::types::{MalError, MalVal};
use itertools::Itertools;

// TODO: 比較演算子でも使えるように抽象化したい
macro_rules! int_op {
    ($name:expr, $func:expr) => {
        crate::types::MalVal::BuiltinFn(|args| {
            args.into_iter()
                .try_reduce(|acc, x| match (acc, x) {
                    (crate::types::MalVal::Number(acc), crate::types::MalVal::Number(x)) => {
                        $func(acc, x)
                    }
                    (z, crate::types::MalVal::Number(_)) | (_, z) => {
                        Err(crate::types::MalError::InvalidType(
                            crate::printer::pr_str(&z, true),
                            "number".to_string(),
                            z.type_str(),
                        ))
                    }
                })?
                .ok_or(crate::types::MalError::WrongArity(
                    $name.to_string(),
                    crate::types::Arity::Variadic(1),
                    0,
                ))
        })
    };
}

macro_rules! int_cmp {
    ($cmp:expr) => {
        crate::types::MalVal::BuiltinFn(|args| {
            Ok(crate::types::MalVal::Bool(
                args.into_iter()
                    .tuple_windows()
                    .try_fold(true, |acc, (a, b)| match (a, b) {
                        (crate::types::MalVal::Number(a), crate::types::MalVal::Number(b)) => {
                            Ok(acc && $cmp(a, b))
                        }
                        (z, MalVal::Number(_)) | (_, z) => {
                            Err(crate::types::MalError::InvalidType(
                                crate::printer::pr_str(&z, true),
                                "number".to_string(),
                                z.type_str(),
                            ))
                        }
                    })?,
            ))
        })
    };
}

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
                    Some(MalVal::List(list, _)) if list.is_empty()
                )))
            }),
        ),
        (
            "count".to_string(),
            MalVal::BuiltinFn(|args| {
                if let Some(MalVal::List(list, _)) = args.get(0) {
                    Ok(MalVal::Number(list.len() as i64))
                } else {
                    Err(MalError::InvalidType(
                        crate::printer::pr_str(&args[0], true),
                        "list".to_string(),
                        args[0].type_str(),
                    ))
                }
            }),
        ),
        (
            "=".to_string(),
            MalVal::BuiltinFn(|args| {
                let first = args[0].clone();
                Ok(MalVal::Bool(
                    args.into_iter().fold(true, |acc, x| acc && first == x),
                ))
            }),
        ),
        ("<".to_string(), int_cmp!(|a, b| a < b)),
        ("<=".to_string(), int_cmp!(|a, b| a <= b)),
        (">".to_string(), int_cmp!(|a, b| a > b)),
        (">=".to_string(), int_cmp!(|a, b| a >= b)),
        (
            "pr-str".to_string(),
            MalVal::BuiltinFn(|args| {
                Ok(MalVal::string(
                    args.into_iter()
                        .map(|x| printer::pr_str(&x, true))
                        .collect::<String>(),
                ))
            }),
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
                    .for_each(|x| print!("{} ", printer::pr_str(&x, true)));
                Ok(MalVal::Nil)
            }),
        ),
        (
            "println".to_string(),
            MalVal::BuiltinFn(|args| {
                args.into_iter()
                    .for_each(|x| print!("{} ", printer::pr_str(&x, false)));
                Ok(MalVal::Nil)
            }),
        ),
    ]
    .into()
}
