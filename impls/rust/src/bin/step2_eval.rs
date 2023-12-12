use fnv::FnvHashMap;
use rusty_mal::printer;
use rusty_mal::reader;
use rusty_mal::types::MalError;
use rusty_mal::types::{MalResult, MalVal};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::rc::Rc;

use rusty_mal::printer::pr_str;

type ReplEnv = FnvHashMap<String, MalVal>;

fn main() {
    let mut env = ReplEnv::default();
    env.insert(
        "+".to_string(),
        MalVal::Func(
            |args| {
                args.iter()
                    .try_fold(MalVal::Number(0), |acc, x| match (acc, x) {
                        (MalVal::Number(acc), MalVal::Number(x)) => Ok(MalVal::Number(acc + x)),
                        _ => Err(MalError::InvalidType(
                            pr_str(x),
                            "number".to_string(),
                            x.type_str(),
                        )),
                    })
            },
            Rc::new(MalVal::Nil),
        ),
    );
    env.insert(
        "-".to_string(),
        MalVal::Func(
            |args| {
                args.iter()
                    .skip(1)
                    .try_fold(args[0].clone(), |acc, x| match (acc, x) {
                        (MalVal::Number(acc), MalVal::Number(x)) => Ok(MalVal::Number(acc - x)),
                        _ => Err(MalError::InvalidType(
                            pr_str(x),
                            "number".to_string(),
                            x.type_str(),
                        )),
                    })
            },
            Rc::new(MalVal::Nil),
        ),
    );
    env.insert(
        "*".to_string(),
        MalVal::Func(
            |args| {
                args.iter()
                    .try_fold(MalVal::Number(1), |acc, x| match (acc, x) {
                        (MalVal::Number(acc), MalVal::Number(x)) => Ok(MalVal::Number(acc * x)),
                        _ => Err(MalError::InvalidType(
                            pr_str(x),
                            "number".to_string(),
                            x.type_str(),
                        )),
                    })
            },
            Rc::new(MalVal::Nil),
        ),
    );
    env.insert(
        "/".to_string(),
        MalVal::Func(
            |args| {
                args.iter()
                    .skip(1)
                    .try_fold(args[0].clone(), |acc, x| match (acc, x) {
                        (MalVal::Number(acc), MalVal::Number(x)) => {
                            if *x == 0 {
                                Err(MalError::DividedByZero)
                            } else {
                                Ok(MalVal::Number(acc / x))
                            }
                        }
                        _ => Err(MalError::InvalidType(
                            pr_str(x),
                            "number".to_string(),
                            x.type_str(),
                        )),
                    })
            },
            Rc::new(MalVal::Nil),
        ),
    );

    loop {
        let mut editor = DefaultEditor::new().unwrap();
        let readline = editor.readline("user> ");
        match readline {
            Ok(line) => println!("{}", rep(line, &env).unwrap_or_else(|e| e.to_string())),
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
fn EVAL(input: MalVal, env: &ReplEnv) -> MalResult {
    match input {
        MalVal::List(ref list, _) => {
            if list.is_empty() {
                return Ok(input);
            }

            match eval_ast(list[0].clone(), env) {
                Ok(MalVal::Func(f, _)) => f(list[1..]
                    .iter()
                    .map(|item| EVAL(item.clone(), env))
                    .collect::<Result<_, _>>()?),
                Ok(_) => Err(MalError::InvalidType(
                    "eliminated".to_string(),
                    "symbol".to_string(),
                    list[1].type_str(),
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

fn rep(input: String, env: &ReplEnv) -> Result<String, MalError> {
    Ok(PRINT(EVAL(READ(input)?, env)?))
}

fn eval_ast(ast: MalVal, env: &ReplEnv) -> MalResult {
    match ast {
        MalVal::Symbol(s) => env
            .get(&(*s))
            .ok_or(MalError::NotFound(s.to_string()))
            .map(|v| v.clone()),
        MalVal::List(l, _) => Ok(MalVal::list(
            l.iter()
                .map(|item| EVAL(item.clone(), env))
                .collect::<Result<_, _>>()?,
        )),
        MalVal::Vector(v, _) => Ok(MalVal::vec(
            v.iter()
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
