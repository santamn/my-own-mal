#![feature(iterator_try_reduce)]

use itertools::Itertools;
use rusty_mal::env::Env;
use rusty_mal::printer;
use rusty_mal::reader;
use rusty_mal::types::{Arity, Closure, MalError, MalResult, MalVal};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

fn main() {
    // 将来的にマクロにしたい
    let mut env = Env::new(None);
    env.set(
        "+",
        MalVal::BuiltinFn(|args| {
            args.into_iter()
                .try_reduce(|acc, x| match (acc, x) {
                    (MalVal::Number(acc), MalVal::Number(x)) => Ok(MalVal::Number(acc + x)),
                    (z, MalVal::Number(_)) | (_, z) => Err(MalError::InvalidType(
                        printer::pr_str(&z),
                        "number".to_string(),
                        z.type_str(),
                    )),
                })?
                .ok_or(MalError::WrongArity("+".to_string(), Arity::Variadic(1), 0))
        }),
    );
    env.set(
        "-",
        MalVal::BuiltinFn(|args| {
            args.into_iter()
                .try_reduce(|acc, x| match (acc, x) {
                    (MalVal::Number(acc), MalVal::Number(x)) => Ok(MalVal::Number(acc - x)),
                    (z, MalVal::Number(_)) | (_, z) => Err(MalError::InvalidType(
                        printer::pr_str(&z),
                        "number".to_string(),
                        z.type_str(),
                    )),
                })?
                .ok_or(MalError::WrongArity("-".to_string(), Arity::Variadic(1), 0))
        }),
    );
    env.set(
        "*",
        MalVal::BuiltinFn(|args| {
            args.into_iter()
                .try_reduce(|acc, x| match (acc, x) {
                    (MalVal::Number(acc), MalVal::Number(x)) => Ok(MalVal::Number(acc * x)),
                    (z, MalVal::Number(_)) | (_, z) => Err(MalError::InvalidType(
                        printer::pr_str(&z),
                        "number".to_string(),
                        z.type_str(),
                    )),
                })?
                .ok_or(MalError::WrongArity("*".to_string(), Arity::Variadic(1), 0))
        }),
    );
    env.set(
        "/",
        MalVal::BuiltinFn(|args| {
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
                .ok_or(MalError::WrongArity("/".to_string(), Arity::Variadic(1), 0))
        }),
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
fn EVAL(input: &MalVal, env: &mut Env) -> MalResult {
    match input {
        MalVal::List(ref list, _) => {
            if list.is_empty() {
                return Ok(input.clone());
            }

            // 特殊フォームの処理
            if let MalVal::Symbol(s) = &list[0] {
                match s.as_str() {
                    "def!" => return special_def(list, env),
                    "let*" => return special_let(list, env),
                    "do" => return special_do(list, env),
                    "if" => return special_if(list, env),
                    _ => {}
                };
            }

            match eval_ast(&list[0], env) {
                Ok(MalVal::BuiltinFn(f)) => f(list[1..]
                    .iter()
                    .map(|item| EVAL(item, env))
                    .collect::<Result<_, _>>()?),
                Ok(t) => Err(MalError::InvalidType(
                    printer::pr_str(&t),
                    "function".to_string(),
                    t.type_str(),
                )),
                err => err,
            }
        }
        _ => eval_ast(input, env),
    }
}

#[allow(non_snake_case)]
fn PRINT(input: &MalVal) -> String {
    printer::pr_str(input)
}

fn rep(input: String, env: &mut Env) -> Result<String, MalError> {
    Ok(PRINT(&EVAL(&READ(input)?, env)?))
}

fn eval_ast(ast: &MalVal, env: &mut Env) -> MalResult {
    match ast {
        MalVal::Symbol(s) => env
            .get(s.as_ref())
            .ok_or(MalError::NotFound(s.to_string()))
            .map(|v| v.clone()),
        MalVal::List(l, _) => Ok(MalVal::list(
            l.iter()
                .map(|item| EVAL(item, env))
                .collect::<Result<_, _>>()?,
        )),
        MalVal::Vector(l, _) => Ok(MalVal::vec(
            l.iter()
                .map(|item| EVAL(item, env))
                .collect::<Result<_, _>>()?,
        )),
        MalVal::HashMap(m, _) => Ok(MalVal::hashmap(
            m.iter()
                .map(|(k, v)| Ok((k.clone(), EVAL(v, env)?)))
                .collect::<Result<_, _>>()?,
        )),
        _ => Ok(ast.clone()),
    }
}

fn special_def(list: &[MalVal], env: &mut Env) -> MalResult {
    if list.len() != 3 {
        return Err(MalError::WrongArity(
            "def!".to_string(),
            Arity::Fixed(2),
            list.len() - 1,
        ));
    }

    if let MalVal::Symbol(s) = &list[1] {
        let val = EVAL(&list[2], env)?;
        env.set(s.to_string(), val.clone());
        Ok(val)
    } else {
        Err(MalError::InvalidType(
            printer::pr_str(&list[1]),
            "symbol".to_string(),
            list[1].type_str(),
        ))
    }
}

fn special_let(list: &[MalVal], env: &Env) -> MalResult {
    if list.len() != 3 {
        return Err(MalError::WrongArity(
            "let*".to_string(),
            Arity::Fixed(2),
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
                    let val = EVAL(v, &mut new_env)?;
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
            "list or vec".to_string(),
            list[1].type_str(),
        ));
    }

    EVAL(&list[2], &mut new_env)
}

fn special_do(list: &[MalVal], env: &mut Env) -> MalResult {
    if list.len() < 2 {
        return Err(MalError::WrongArity(
            "do".to_string(),
            Arity::Variadic(1),
            list.len() - 1,
        ));
    }

    list[1..]
        .iter()
        .cloned()
        .try_reduce(|_, ref item| EVAL(item, env))
        .map(|v| v.unwrap())
}

fn special_if(list: &[MalVal], env: &mut Env) -> MalResult {
    if list.len() < 3 || list.len() > 4 {
        return Err(MalError::WrongArity(
            "if".to_string(),
            Arity::JustOrOneLess(4),
            list.len() - 1,
        ));
    }

    match EVAL(&list[1], env)? {
        MalVal::Bool(false) | MalVal::Nil => {
            if list.len() == 4 {
                EVAL(&list[3], env)
            } else {
                Ok(MalVal::Nil)
            }
        }
        _ => EVAL(&list[2], env),
    }
}

fn special_fn(list: &[MalVal], env: &Env) -> MalResult {
    if list.len() != 3 {
        return Err(MalError::WrongArity(
            "fn*".to_string(),
            Arity::Fixed(2),
            list.len() - 1,
        ));
    }

    if let MalVal::List(params, _) | MalVal::Vector(params, _) = &list[1] {
        Ok(MalVal::func(Closure {
            // TODO: '&'が2つ以上ある場合はエラーにする
            params: {
                params
                    .iter()
                    .map(|item| match item {
                        MalVal::Symbol(s) => Ok(s.to_string()),
                        _ => Err(MalError::InvalidType(
                            printer::pr_str(item),
                            "symbol".to_string(),
                            item.type_str(),
                        )),
                    })
                    // try_foldを使う
                    .collect::<Result<_, _>>()?
            },
            body: list[2].clone(),
            env: env.clone(),
        }))
    } else {
        Err(MalError::InvalidType(
            printer::pr_str(&list[1]),
            "list or vec".to_string(),
            list[1].type_str(),
        ))
    }
}
