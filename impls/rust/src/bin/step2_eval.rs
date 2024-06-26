use fnv::FnvHashMap;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use rustymal::printer;
use rustymal::reader;
use rustymal::types::MalError;
use rustymal::types::{MalResult, MalVal};

type ReplEnv = FnvHashMap<String, MalVal>;

fn main() {
    let env = ReplEnv::default();

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
                Ok(MalVal::BuiltinFn(f)) => f(list[1..]
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
    printer::pr_str(&input, false)
}

fn rep(input: String, env: &ReplEnv) -> Result<String, MalError> {
    Ok(PRINT(EVAL(READ(input)?, env)?))
}

fn eval_ast(ast: MalVal, env: &ReplEnv) -> MalResult {
    match ast {
        MalVal::Symbol(s) => env
            .get(&(*s))
            .ok_or(MalError::NotFound(s.to_string()))
            .cloned(),
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
