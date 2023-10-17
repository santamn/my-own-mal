use rust::printer;
use rust::reader;
use rust::types::MalError;
use rust::types::{MalResult, MalVal};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

fn main() {
    loop {
        let mut editor = DefaultEditor::new().unwrap();
        let readline = editor.readline("user> ");
        match readline {
            Ok(line) => println!("{}", rep(line).unwrap_or_else(|e| e.to_string())),
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
fn EVAL(input: MalVal) -> MalVal {
    input
}

#[allow(non_snake_case)]
fn PRINT(input: MalVal) -> String {
    printer::pr_str(&input)
}

fn rep(input: String) -> Result<String, MalError> {
    Ok(PRINT(EVAL(READ(input)?)))
}

// 追加課題
