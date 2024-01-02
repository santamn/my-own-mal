#![feature(iterator_try_reduce)]

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
    let _ = rep(
        "(def! not (fn* [a] (if a false true)))".to_string(),
        &mut env,
    );
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

// TODO: if letを使う
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
                    "fn*" => return special_fn(list, env),
                    _ => {}
                };
            }

            let MalVal::List(list, _) = eval_ast(input, env)? else {
                // SAFETY: Listの場合はeval_astで必ずMalVal::Listが返る
                unsafe { unreachable_unchecked() }
            };
            // TODO: vecやhashmapも関数のように扱えるようにする
            match &list[0] {
                MalVal::BuiltinFn(f) => f(list[1..].to_vec()),
                MalVal::Func(f, _) => {
                    let (rev_p, v) = f.rev_params.clone();
                    let mut new_env = Env::with_bind(
                        Some(&f.env),
                        rev_p.into_iter().rev(),
                        v,
                        list[1..].iter().cloned(),
                    );
                    EVAL(&f.body, &mut new_env)
                }
                not_func => Err(MalError::InvalidType(
                    printer::pr_str(not_func, true),
                    "function".to_string(),
                    not_func.type_str(),
                )),
            }
        }
        _ => eval_ast(input, env),
    }
}

#[allow(non_snake_case)]
fn PRINT(input: &MalVal) -> String {
    printer::pr_str(input, true)
}

// READ -> EVAL -> PRINT
fn rep(input: String, env: &mut Env) -> Result<String, MalError> {
    Ok(PRINT(&EVAL(&READ(input)?, env)?))
}

// TODO: try_collectを使う
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
            printer::pr_str(&list[1], true),
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
                        printer::pr_str(k, true),
                        "symbol".to_string(),
                        k.type_str(),
                    ))
                }
            })?;
    } else {
        return Err(MalError::InvalidType(
            printer::pr_str(&list[1], true),
            "list or vec".to_string(),
            list[1].type_str(),
        ));
    }

    EVAL(&list[2], &mut new_env)
}

fn special_do(list: &[MalVal], env: &mut Env) -> MalResult {
    list[1..]
        .iter()
        .try_fold(MalVal::Nil, |_, item| EVAL(item, env))
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

#[cfg(test)]
mod tests {
    use rustymal::int_op;
    use rustymal::{env::Env, types::MalVal};

    // TODO: funcのevalでどのenvがいつ使われるかを見る
    #[test]
    fn test_eval_nested_function() {
        // (def! nested-fn (fn* [a] (fn* [b] (+ a b))))
        let nested_fn = MalVal::list(vec![
            MalVal::symbol("fn*"),
            MalVal::list(vec![MalVal::symbol("a")]),
            MalVal::list(vec![
                MalVal::symbol("fn*"),
                MalVal::list(vec![MalVal::symbol("b")]),
                MalVal::list(vec![
                    MalVal::symbol("+"),
                    MalVal::symbol("b"),
                    MalVal::symbol("a"),
                ]),
            ]),
        ]);
        // (nested-fn 7)
        let appliy_a = MalVal::list(vec![nested_fn, MalVal::Number(7)]);
        // 以下の等号は構造的には正しいが、Closure同士の比較ができないため失敗する
        // assert_eq!(
        //     super::EVAL(&appliy_a, &mut Env::new(None)).unwrap(),
        //     MalVal::func(rustymal::types::Closure {
        //         rev_params: (vec!["b".to_string()], None),
        //         body: MalVal::list(vec![
        //             MalVal::symbol("+"),
        //             MalVal::symbol("b"),
        //             MalVal::symbol("a"),
        //         ]),
        //         env: {
        //             let outer = Env::new(None);
        //             let mut env = Env::new(Some(&outer));
        //             env.set("a", MalVal::Number(7));
        //             env
        //         },
        //     })
        // );

        let mut core_env: Env = [(
            "+".to_string(),
            int_op!("+", |a, b| Ok(MalVal::Number(a + b))),
        )]
        .into();
        // println!("{:?}", super::EVAL(&nested_fn, &mut env).unwrap());

        // ((nested-fn 7) 5)
        assert_eq!(
            super::EVAL(
                &MalVal::list(vec![appliy_a, MalVal::Number(5)]),
                &mut core_env
            )
            .unwrap(),
            MalVal::Number(12)
        );
    }

    #[test]
    fn test_eval_body() {
        // (def! body (+ a b))
        let body = MalVal::list(vec![
            MalVal::symbol("+"),
            MalVal::symbol("b"),
            MalVal::symbol("a"),
        ]);

        // Env {outer: Env {a: 5}, b: 7}
        let core_env: Env = [(
            "+".to_string(),
            int_op!("+", |a, b| Ok(MalVal::Number(a + b))),
        )]
        .into();
        let mut outer_env = Env::new(Some(&core_env));
        outer_env.set("a", MalVal::Number(5));
        let mut env = Env::new(Some(&outer_env));
        env.set("b", MalVal::Number(7));

        assert_eq!(super::EVAL(&body, &mut env).unwrap(), MalVal::Number(12));
    }

    #[test]
    fn test_eval_func_with_env() {
        // (defn! func (fn* [b] (+ b a)))
        let func = MalVal::list(vec![
            MalVal::symbol("fn*"),
            MalVal::list(vec![MalVal::symbol("b")]),
            MalVal::list(vec![
                MalVal::symbol("+"),
                MalVal::symbol("b"),
                MalVal::symbol("a"),
            ]),
        ]);
        // (func 5)
        let applied = MalVal::list(vec![func.clone(), MalVal::Number(5)]);

        // Env {a: 7}
        let core_env: Env = [(
            "+".to_string(),
            int_op!("+", |a, b| Ok(MalVal::Number(a + b))),
        )]
        .into();
        let mut env = Env::new(Some(&core_env));
        env.set("a", MalVal::Number(7));

        assert_eq!(super::EVAL(&applied, &mut env).unwrap(), MalVal::Number(12));
    }

    #[test]
    fn test_eval_variadic_func() {
        // (defn! func (fn* [& more] (count more)))
        let func = MalVal::list(vec![
            MalVal::symbol("fn*"),
            MalVal::list(vec![MalVal::symbol("&"), MalVal::symbol("more")]),
            MalVal::list(vec![MalVal::symbol("count"), MalVal::symbol("more")]),
        ]);

        // (func)
        assert_eq!(
            super::EVAL(
                &MalVal::list(vec![func]),
                &mut [(
                    "count".to_string(),
                    MalVal::BuiltinFn(|args| match args.get(0) {
                        None | Some(MalVal::Nil) => Ok(MalVal::Number(0)),
                        Some(MalVal::List(list, _)) => Ok(MalVal::Number(list.len() as i64)),
                        Some(z) => Err(rustymal::types::MalError::InvalidType(
                            rustymal::printer::pr_str(z, true),
                            "nil or list".to_string(),
                            z.type_str(),
                        )),
                    }),
                )]
                .into()
            )
            .unwrap(),
            MalVal::Number(0)
        );
    }
}
