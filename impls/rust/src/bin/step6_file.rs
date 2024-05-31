use std::hint::unreachable_unchecked;

use itertools::Itertools;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use rustymal::core;
use rustymal::env::Env;
use rustymal::printer;
use rustymal::reader;
use rustymal::types::{Arity, Closure, MalError, MalResult, MalVal};

fn main() {
    let mut env = core::env();
    rep(
        "(def! not (fn* [a] (if a false true)))".to_string(),
        &mut env,
    )
    .unwrap();
    rep(
        "(def! load-file (fn* [filename] (eval (read-string (str \"(do \" (slurp filename) \")\")))))"
            .to_string(),
        &mut env,
    )
    .unwrap();
    rep(
        "(def! apply (fn* [f args] (eval (cons f args))))".to_string(),
        &mut env,
    )
    .unwrap();
    rep(
        "(def! swap! (fn* [a f & args] (apply reset! a (f (deref a) args))))".to_string(),
        &mut env,
    )
    .unwrap();

    loop {
        let mut editor = DefaultEditor::new().unwrap();
        let line = editor.readline("user=> ");
        match line {
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
fn EVAL(mut input: MalVal, env: &mut Env) -> MalResult {
    loop {
        if let MalVal::List(ref list, _) = input {
            if list.is_empty() {
                return Ok(input);
            }

            // 特殊フォームの処理
            if let MalVal::Symbol(s) = &list[0] {
                match s.as_str() {
                    "def!" => return special_def(list, env),
                    "fn*" => return special_fn(list, env),
                    "do" => {
                        input = special_do(list, env)?;
                        continue;
                    }
                    "if" => {
                        input = special_if(list, env)?;
                        continue;
                    }
                    "let*" => {
                        input = special_let(list, env)?;
                        continue;
                    }
                    "eval" => {
                        if list.len() != 2 {
                            return Err(MalError::WrongArity(
                                "eval".to_string(),
                                Arity::Fixed(1),
                                list.len() - 1,
                            ));
                        }
                        input = EVAL(list[1].clone(), env)?;
                        continue;
                    }
                    _ => {}
                };
            }

            let MalVal::List(list, _) = eval_ast(input, env)? else {
                // SAFETY: Listの場合はeval_astで必ずMalVal::Listが返る
                unsafe { unreachable_unchecked() }
            };
            // TODO: vecやhashmapも関数のように扱えるようにする
            match &list[0] {
                MalVal::BuiltinFn(f) => return f(list[1..].to_vec()),
                MalVal::Func(f, _) => {
                    let (rev_p, v) = f.rev_params.clone();
                    if v.is_some() {
                        if rev_p.len() > list.len() - 1 {
                            return Err(MalError::WrongArity(
                                "function".to_string(),
                                Arity::Variadic(rev_p.len()),
                                list.len() - 1,
                            ));
                        }
                    } else if rev_p.len() != list.len() - 1 {
                        return Err(MalError::WrongArity(
                            "function".to_string(),
                            Arity::Fixed(rev_p.len()),
                            list.len() - 1,
                        ));
                    }

                    *env = Env::with_bind(
                        Some(&f.env),
                        rev_p.into_iter().rev(),
                        v,
                        list[1..].iter().cloned(),
                    );
                    input = f.body.clone();
                }
                not_func => {
                    return Err(MalError::InvalidType(
                        printer::pr_str(not_func, true),
                        "function".to_string(),
                        not_func.type_str(),
                    ))
                }
            }
        } else {
            return eval_ast(input, env);
        }
    }
}

#[allow(non_snake_case)]
fn PRINT(input: &MalVal) -> String {
    printer::pr_str(input, true)
}

// READ -> EVAL -> PRINT
fn rep(input: String, env: &mut Env) -> Result<String, MalError> {
    Ok(PRINT(&EVAL(READ(input)?, env)?))
}

fn eval_ast(ast: MalVal, env: &mut Env) -> MalResult {
    match ast {
        MalVal::Symbol(s) => env.get(s.as_ref()).ok_or(MalError::NotFound(s.to_string())),
        MalVal::List(l, _) => Ok(MalVal::list(
            l.iter().map(|item| EVAL(item.clone(), env)).try_collect()?,
        )),
        MalVal::Vector(l, _) => Ok(MalVal::vec(
            l.iter().map(|item| EVAL(item.clone(), env)).try_collect()?,
        )),
        MalVal::HashMap(m, _) => Ok(MalVal::hashmap(
            m.iter()
                .map(|(k, v)| Ok((k.clone(), EVAL(v.clone(), env)?)))
                .try_collect()?,
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
        let val = EVAL(list[2].clone(), env)?;
        env.set(s.to_string(), val.clone());
        Ok(val)
    } else {
        Err(MalError::InvalidType(
            printer::pr_str(&list[1], true),
            "symbol".to_string(),
            list[1].type_str(),
        ))
    }
}

fn special_do(list: &[MalVal], env: &mut Env) -> MalResult {
    let mut iter = list[1..].iter();
    let last = iter.next_back().unwrap_or(&MalVal::Nil).clone();
    iter.try_for_each(|x| {
        EVAL(x.clone(), env)?;
        Ok(())
    })?;
    Ok(last)
}

fn special_if(list: &[MalVal], env: &mut Env) -> MalResult {
    if list.len() < 3 || list.len() > 4 {
        return Err(MalError::WrongArity(
            "if".to_string(),
            Arity::JustOrOneLess(4),
            list.len() - 1,
        ));
    }

    match EVAL(list[1].clone(), env)? {
        MalVal::Bool(false) | MalVal::Nil => {
            if list.len() == 4 {
                Ok(list[3].clone())
            } else {
                Ok(MalVal::Nil)
            }
        }
        _ => Ok(list[2].clone()),
    }
}

fn special_let(list: &[MalVal], env: &mut Env) -> MalResult {
    if list.len() != 3 {
        return Err(MalError::WrongArity(
            "let*".to_string(),
            Arity::Fixed(2),
            list.len() - 1,
        ));
    }

    if let MalVal::List(bindings, _) | MalVal::Vector(bindings, _) = &list[1] {
        let mut new_env = Env::new(Some(env));
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
                        printer::pr_str(k, true),
                        "symbol".to_string(),
                        k.type_str(),
                    ))
                }
            })?;

        *env = new_env;
        Ok(list[2].clone())
    } else {
        Err(MalError::InvalidType(
            printer::pr_str(&list[1], true),
            "list or vec".to_string(),
            list[1].type_str(),
        ))
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
        let ampersand_error = "invalid function definition: & in incorrect position";
        Ok(MalVal::func(Closure {
            // 逆順で引数をチェックする
            // [a b & c] => (& c), (a b)
            rev_params: params.rchunks(2).enumerate().try_fold(
                (Vec::with_capacity(params.len()), None),
                // SAFETY: 常にc.len() >= 1
                |(mut vec, v), (i, c)| match (i, unsafe { c.get_unchecked(0) }, c.get(1)) {
                    (0, MalVal::Symbol(s), Some(MalVal::Symbol(t)))
                        if s.as_str() == "&" && t.as_str() != "&" =>
                    {
                        Ok((vec, Some(t.to_string())))
                    }
                    (_, MalVal::Symbol(s), Some(MalVal::Symbol(t)))
                        if s.as_str() == "&" || t.as_str() == "&" =>
                    {
                        Err(MalError::InvalidSyntax(ampersand_error.to_string()))
                    }
                    (_, MalVal::Symbol(s), Some(MalVal::Symbol(t))) => Ok((
                        {
                            vec.push(t.to_string());
                            vec.push(s.to_string());
                            vec
                        },
                        v,
                    )),
                    (_, MalVal::Symbol(s), None) => {
                        if s.as_str() != "&" {
                            Ok((
                                {
                                    vec.push(s.to_string());
                                    vec
                                },
                                v,
                            ))
                        } else {
                            Err(MalError::InvalidSyntax(ampersand_error.to_string()))
                        }
                    }
                    (_, x, Some(MalVal::Symbol(_))) | (_, _, Some(x)) | (_, x, None) => {
                        Err(MalError::InvalidType(
                            printer::pr_str(x, true),
                            "symbol".to_string(),
                            x.type_str(),
                        ))
                    }
                },
            )?, // NOTE: vecは逆順になっている
            body: list[2].clone(),
            env: env.clone(),
        }))
    } else {
        Err(MalError::InvalidType(
            printer::pr_str(&list[1], true),
            "list or vec".to_string(),
            list[1].type_str(),
        ))
    }
}
