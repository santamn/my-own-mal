use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    loop {
        let mut editor = DefaultEditor::new()?;
        let readline = editor.readline("user> ");
        match readline {
            Ok(line) => println!("{}", rep(line)),
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break Ok(()),
            Err(err) => {
                println!("Error: {:?}", err);
                break Err(err.into());
            }
        }
    }
}

#[allow(non_snake_case)]
fn READ(input: String) -> String {
    input
}

#[allow(non_snake_case)]
fn EVAL(input: String) -> String {
    input
}

#[allow(non_snake_case)]
fn PRINT(input: String) -> String {
    input
}

fn rep(input: String) -> String {
    PRINT(EVAL(READ(input)))
}
