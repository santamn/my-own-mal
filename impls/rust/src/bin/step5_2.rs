use itertools::Itertools;
use rustymal::env::Env;
use rustymal::printer;
use rustymal::types::{Arity, Closure, MalError, MalResult, MalVal};

fn main() {}

#[allow(non_snake_case)]
fn EVAL(input: MalVal, env: &mut Env) -> MalResult {
    loop {
        todo!()
    }
}

fn special_do(mut list: Vec<MalVal>, env: &mut Env) -> MalResult {
    let last = list.pop().unwrap_or(MalVal::Nil);
    list.into_iter().try_for_each(|x| {
        EVAL(x, env)?;
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

fn special_let(list: Vec<MalVal>, env: &mut Env) -> MalResult {
    if list.len() != 3 {
        return Err(MalError::WrongArity(
            "let*".to_string(),
            Arity::Fixed(2),
            list.len() - 1,
        ));
    }

    let [_, bindings, body] = unsafe { list.try_into().unwrap_unchecked() };
    if let MalVal::List(bindings, _) | MalVal::Vector(bindings, _) = bindings {
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
        Ok(body)
    } else {
        Err(MalError::InvalidType(
            printer::pr_str(&bindings, true),
            "list or vec".to_string(),
            bindings.type_str(),
        ))
    }
}
