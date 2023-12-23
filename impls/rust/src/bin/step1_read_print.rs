use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use rustymal::printer;
use rustymal::reader;
use rustymal::types::MalError;
use rustymal::types::{MalResult, MalVal};

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
    printer::pr_str(&input, false)
}

fn rep(input: String) -> Result<String, MalError> {
    Ok(PRINT(EVAL(READ(input)?)))
}

// 追加課題

// 1. 以下の型をサポートする: 済
//  - MalString: '\'で'\'と'"'をエスケープする
//  - MalBool
//  - MalNil
// 2. 括弧の対応が取れていない場合はエラーを返す: 済
// 3. リーダーマクロのサポート: tests/step1_read_print.malを参照
// 4. キーワード、ベクター、ハッシュマップのサポート: 済
// 5. コメントのサポート: ';'から行末までをコメントとして扱う: 済
