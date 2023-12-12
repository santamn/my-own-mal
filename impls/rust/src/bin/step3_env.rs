#![feature(iterator_try_reduce)]

use itertools::Itertools;
use rust::env::Env;
use rust::printer;
use rust::reader;
use rust::types::{Arity, MalError, MalResult, MalVal};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::rc::Rc;

fn main() {
    // 将来的にマクロにしたい
    let mut env = Env::new(None);
    env.set(
        "+",
        MalVal::Func(
            |args| {
                let length = args.len();
                args.into_iter()
                    .try_reduce(|acc, x| match (acc, x) {
                        (MalVal::Number(acc), MalVal::Number(x)) => Ok(MalVal::Number(acc + x)),
                        (z, MalVal::Number(_)) | (_, z) => Err(MalError::InvalidType(
                            printer::pr_str(&z),
                            "number".to_string(),
                            z.type_str(),
                        )),
                    })?
                    .ok_or(MalError::WrongArity(
                        "+".to_string(),
                        Arity(1, true),
                        length,
                    ))
            },
            Rc::new(MalVal::Nil),
        ),
    );
    env.set(
        "-",
        MalVal::Func(
            |args| {
                let length = args.len();
                args.into_iter()
                    .try_reduce(|acc, x| match (acc, x) {
                        (MalVal::Number(acc), MalVal::Number(x)) => Ok(MalVal::Number(acc - x)),
                        (z, MalVal::Number(_)) | (_, z) => Err(MalError::InvalidType(
                            printer::pr_str(&z),
                            "number".to_string(),
                            z.type_str(),
                        )),
                    })?
                    .ok_or(MalError::WrongArity(
                        "-".to_string(),
                        Arity(1, true),
                        length,
                    ))
            },
            Rc::new(MalVal::Nil),
        ),
    );
    env.set(
        "*",
        MalVal::Func(
            |args| {
                let length = args.len();
                args.into_iter()
                    .try_reduce(|acc, x| match (acc, x) {
                        (MalVal::Number(acc), MalVal::Number(x)) => Ok(MalVal::Number(acc * x)),
                        (z, MalVal::Number(_)) | (_, z) => Err(MalError::InvalidType(
                            printer::pr_str(&z),
                            "number".to_string(),
                            z.type_str(),
                        )),
                    })?
                    .ok_or(MalError::WrongArity(
                        "*".to_string(),
                        Arity(1, true),
                        length,
                    ))
            },
            Rc::new(MalVal::Nil),
        ),
    );
    env.set(
        "/",
        MalVal::Func(
            |args| {
                let length = args.len();
                args.into_iter()
                    .try_reduce(|acc, x| match (acc, x) {
                        (MalVal::Number(acc), MalVal::Number(x)) => acc
                            .checked_div(x)
                            .map(MalVal::Number)
                            .ok_or(MalError::DividedByZero),
                        (z, MalVal::Number(_)) | (_, z) => Err(MalError::InvalidType(
                            printer::pr_str(&z),
                            "number".to_string(),
                            z.type_str(),
                        )),
                    })?
                    .ok_or(MalError::WrongArity(
                        "/".to_string(),
                        Arity(1, true),
                        length,
                    ))
            },
            Rc::new(MalVal::Nil),
        ),
    );

    loop {
        let mut editor = DefaultEditor::new().unwrap();
        let readline = editor.readline("user> ");
        match readline {
            Ok(line) => println!("{}", rep(line, &mut env).unwrap_or_else(|e| e.to_string())),
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
    }
}

#[allow(non_snake_case)]
fn READ(input: String) -> MalResult {
    reader::read_str(input)
}

#[allow(non_snake_case)]
fn EVAL(input: MalVal, env: &mut Env) -> MalResult {
    match input {
        MalVal::List(ref list, _) => {
            if list.is_empty() {
                return Ok(input);
            }

            // 特殊フォームの処理
            if let MalVal::Symbol(s) = &list[0] {
                match s.as_str() {
                    "def!" => {
                        if list.len() != 3 {
                            return Err(MalError::WrongArity(
                                "def!".to_string(),
                                Arity(2, false),
                                list.len() - 1,
                            ));
                        }

                        if let MalVal::Symbol(s) = &list[1] {
                            let val = EVAL(list[2].clone(), env)?;
                            env.set(s.to_string(), val.clone());
                            return Ok(val);
                        } else {
                            return Err(MalError::InvalidType(
                                s.to_string(),
                                "symbol".to_string(),
                                list[1].type_str(),
                            ));
                        }
                    }
                    "let*" => {
                        if list.len() != 3 {
                            return Err(MalError::WrongArity(
                                "let*".to_string(),
                                Arity(2, false),
                                list.len() - 1,
                            ));
                        }

                        let mut new_env = Env::new(Some(env));
                        if let MalVal::List(bindings, _) | MalVal::Vector(bindings, _) = &list[1] {
                            bindings
                                .iter()
                                .chain(std::iter::once(&MalVal::Nil)) // 奇数個の場合に対応するため
                                .tuples()
                                .try_for_each(|(k, v)| {
                                    if let MalVal::Symbol(s) = k {
                                        let val = EVAL(v.clone(), &mut new_env)?;
                                        new_env.set(s.to_string(), val);
                                        Ok(())
                                    } else {
                                        Err(MalError::InvalidType(
                                            printer::pr_str(k),
                                            "symbol".to_string(),
                                            k.type_str(),
                                        ))
                                    }
                                })?;
                        } else {
                            return Err(MalError::InvalidType(
                                printer::pr_str(&list[1]),
                                "list".to_string(),
                                list[1].type_str(),
                            ));
                        }

                        return EVAL(list[2].clone(), &mut new_env);
                    }
                    _ => (),
                }
            }

            match eval_ast(list[0].clone(), env) {
                Ok(MalVal::Func(f, _)) => f(list[1..]
                    .iter()
                    .map(|item| EVAL(item.clone(), env))
                    .collect::<Result<_, _>>()?),
                Ok(f) => Err(MalError::InvalidType(
                    printer::pr_str(&f),
                    "function".to_string(),
                    f.type_str(),
                )),
                Err(e) => Err(e),
            }
        }
        _ => eval_ast(input, env),
    }
}

#[allow(non_snake_case)]
fn PRINT(input: MalVal) -> String {
    printer::pr_str(&input)
}

fn rep(input: String, env: &mut Env) -> Result<String, MalError> {
    Ok(PRINT(EVAL(READ(input)?, env)?))
}

fn eval_ast(ast: MalVal, env: &mut Env) -> MalResult {
    match ast {
        MalVal::Symbol(s) => env
            .get(&(*s))
            .ok_or(MalError::NotFound(s.to_string()))
            .map(|v| v.clone()),
        MalVal::List(l, _) | MalVal::Vector(l, _) => Ok(MalVal::list(
            l.iter()
                .map(|item| EVAL(item.clone(), env))
                .collect::<Result<_, _>>()?,
        )),
        MalVal::HashMap(m, _) => Ok(MalVal::hashmap(
            m.iter()
                .map(|(k, v)| Ok((k.clone(), EVAL(v.clone(), env)?)))
                .collect::<Result<_, _>>()?,
        )),
        _ => Ok(ast),
    }
}
