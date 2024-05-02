use rustymal::env::Env;
use rustymal::types::{MalResult, MalVal};

fn main() {}

#[allow(non_snake_case)]
fn EVAL(input: MalVal, env: &mut Env) -> MalResult {
    loop {}
}

fn special_do(mut list: Vec<MalVal>, env: &mut Env) -> MalResult {
    let last = list.pop().unwrap_or(MalVal::Nil);
    list.into_iter().try_for_each(|x| {
        EVAL(x, env)?;
        Ok(())
    })?;
    Ok(last)
}

fn special_if(mut list: Vec<MalVal>, env: &mut Env) -> MalResult {
    if list.len() < 3 || list.len() > 4 {
        return Err(MalError::WrongArity(
            "if".to_string(),
            Arity::JustOrOneLess(4),
            list.len() - 1,
        ));
    }

    match EVAL(list.first().unwrap(), env)? {
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
