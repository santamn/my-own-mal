use std::fs::File;
use std::io::{self, BufWriter, Read, Write};

use crate::env::Env;
use crate::printer;
use crate::reader;
use crate::types::{Arity, MalError, MalVal};
use itertools::Itertools;

#[macro_export]
macro_rules! int_op {
    ($name:expr, $func:expr) => {
        $crate::types::MalVal::BuiltinFn(|args| {
            args.into_iter()
                .try_reduce(|acc, x| match (acc, x) {
                    ($crate::types::MalVal::Number(acc), $crate::types::MalVal::Number(x)) =>
                    {
                        #[allow(clippy::redundant_closure_call)]
                        $func(acc, x)
                    }
                    (z, $crate::types::MalVal::Number(_)) | (_, z) => {
                        Err($crate::types::MalError::InvalidType(
                            $crate::printer::pr_str(&z, true),
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
                    ($crate::types::MalVal::Number(a), $crate::types::MalVal::Number(b)) =>
                    {
                        #[allow(clippy::redundant_closure_call)]
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
            MalVal::BuiltinFn(|mut args| {
                if args.len() != 1 {
                    return Err(MalError::WrongArity(
                        "list?".to_string(),
                        Arity::Fixed(1),
                        args.len(),
                    ));
                }
                Ok(MalVal::Bool(matches!(
                    unsafe { args.pop().unwrap_unchecked() },
                    MalVal::List(_, _)
                )))
            }),
        ),
        (
            "empty?".to_string(),
            MalVal::BuiltinFn(|args| {
                if args.len() != 1 {
                    return Err(MalError::WrongArity(
                        "list?".to_string(),
                        Arity::Fixed(1),
                        args.len(),
                    ));
                }
                let arg = unsafe { args.first().unwrap_unchecked() };
                Ok(MalVal::Bool(
                    matches!(
                        arg,
                        MalVal::List(v, _) | MalVal::Vector(v, _) if v.is_empty()
                    ) || matches!(
                        arg,
                        MalVal::HashMap(map, _) if map.is_empty()
                    ) || matches!(
                        arg,
                        MalVal::HashSet(set, _) if set.is_empty()
                    ),
                ))
            }),
        ),
        (
            "count".to_string(),
            MalVal::BuiltinFn(|args| {
                if args.len() > 1 {
                    return Err(MalError::WrongArity(
                        "count".to_string(),
                        Arity::JustOrOneLess(1),
                        args.len(),
                    ));
                }
                match args.first() {
                    None | Some(MalVal::Nil) => Ok(MalVal::Number(0)),
                    Some(MalVal::List(v, _) | MalVal::Vector(v, _)) => {
                        Ok(MalVal::Number(v.len() as i64))
                    }
                    Some(MalVal::HashMap(map, _)) => Ok(MalVal::Number(map.len() as i64)),
                    Some(MalVal::HashSet(set, _)) => Ok(MalVal::Number(set.len() as i64)),
                    Some(z) => Err(MalError::InvalidType(
                        printer::pr_str(z, true),
                        "nil, list, vector, hashmap or hashset".to_string(),
                        z.type_str(),
                    )),
                }
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
                fast_print(args.into_iter().map(|x| printer::pr_str(&x, true)));
                Ok(MalVal::Nil)
            }),
        ),
        (
            "println".to_string(),
            MalVal::BuiltinFn(|args| {
                fast_print(args.into_iter().map(|x| printer::pr_str(&x, false)));
                Ok(MalVal::Nil)
            }),
        ),
        (
            "read-string".to_string(),
            MalVal::BuiltinFn(|mut args| {
                if args.len() != 1 {
                    return Err(MalError::WrongArity(
                        "read-string".to_string(),
                        Arity::Fixed(1),
                        args.len(),
                    ));
                }
                let s = unsafe { args.pop().unwrap_unchecked() };
                if let MalVal::String(s) = s {
                    Ok(reader::read_str(s.to_string())?)
                } else {
                    Err(MalError::InvalidType(
                        printer::pr_str(&s, true),
                        "string".to_string(),
                        s.type_str(),
                    ))
                }
            }),
        ),
        (
            "slurp".to_string(),
            MalVal::BuiltinFn(|mut args| {
                if args.len() != 1 {
                    return Err(MalError::WrongArity(
                        "slurp".to_string(),
                        Arity::Fixed(1),
                        args.len(),
                    ));
                }
                let s = unsafe { args.pop().unwrap_unchecked() };
                if let MalVal::String(s) = s {
                    File::open(s.as_ref())
                        .map_err(|e| MalError::Other(e.to_string()))
                        .and_then(|mut f| {
                            let mut s = String::new();
                            f.read_to_string(&mut s)
                                .map_err(|e| MalError::Other(e.to_string()))?;
                            Ok(MalVal::string(s))
                        })
                } else {
                    Err(MalError::InvalidType(
                        printer::pr_str(&s, true),
                        "string".to_string(),
                        s.type_str(),
                    ))
                }
            }),
        ),
        (
            "atom".to_string(),
            MalVal::BuiltinFn(|mut args| {
                if args.len() != 1 {
                    return Err(MalError::WrongArity(
                        "atom".to_string(),
                        Arity::Fixed(1),
                        args.len(),
                    ));
                }
                Ok(MalVal::atom(unsafe { args.pop().unwrap_unchecked() }))
            }),
        ),
        (
            "atom?".to_string(),
            MalVal::BuiltinFn(|args| {
                if args.len() != 1 {
                    return Err(MalError::WrongArity(
                        "atom?".to_string(),
                        Arity::Fixed(1),
                        args.len(),
                    ));
                }
                Ok(MalVal::Bool(matches!(
                    unsafe { args.first().unwrap_unchecked() },
                    MalVal::Atom(_)
                )))
            }),
        ),
        (
            "deref".to_string(),
            MalVal::BuiltinFn(|mut args| {
                if args.len() != 1 {
                    return Err(MalError::WrongArity(
                        "deref".to_string(),
                        Arity::Fixed(1),
                        args.len(),
                    ));
                }
                match unsafe { args.pop().unwrap_unchecked() } {
                    MalVal::Atom(a) => Ok(a.borrow().clone()),
                    z => Err(MalError::InvalidType(
                        printer::pr_str(&z, true),
                        "atom".to_string(),
                        z.type_str(),
                    )),
                }
            }),
        ),
        (
            "reset!".to_string(),
            MalVal::BuiltinFn(|mut args| {
                if args.len() != 2 {
                    return Err(MalError::WrongArity(
                        "reset!".to_string(),
                        Arity::Fixed(2),
                        args.len(),
                    ));
                }
                match unsafe { (args.pop().unwrap_unchecked(), args.pop().unwrap_unchecked()) } {
                    (MalVal::Atom(a), v) => {
                        *a.borrow_mut() = v.clone();
                        Ok(v)
                    }
                    (z, _) => Err(MalError::InvalidType(
                        printer::pr_str(&z, true),
                        "atom".to_string(),
                        z.type_str(),
                    )),
                }
            }),
        ),
        (
            "swap!".to_string(),
            MalVal::BuiltinFn(|mut args| {
                if args.len() < 2 {
                    return Err(MalError::WrongArity(
                        "swap!".to_string(),
                        Arity::Variadic(2),
                        args.len(),
                    ));
                }
                match unsafe { (args.pop().unwrap_unchecked(), args.pop().unwrap_unchecked()) } {
                    (MalVal::Atom(a), MalVal::Func(f, _)) => {
                        let mut v = a.borrow_mut();
                        let mut args = vec![v.clone()];
                        args.extend(args);
                        f.apply(args)
                    }
                    (MalVal::Atom(a), MalVal::BuiltinFn(f)) => {
                        let mut v = a.borrow_mut();
                        let mut args = vec![v.clone()];
                        args.extend(args);
                        f(args)
                    }
                    (z, _) => Err(MalError::InvalidType(
                        printer::pr_str(&z, true),
                        "atom".to_string(),
                        z.type_str(),
                    )),
                }
            }),
        ),
    ]
    .into()
}

fn fast_print<I>(mut s: I)
where
    I: Iterator<Item = String>,
{
    let mut out = BufWriter::new(io::stdout().lock());
    if let Some(str) = s.next() {
        out.write_all(str.as_bytes()).unwrap();
        s.for_each(|str| {
            out.write_all(&[b' ']).unwrap();
            out.write_all(str.as_bytes()).unwrap();
        });
    }
    out.write_all(&[b'\n']).unwrap();
}
