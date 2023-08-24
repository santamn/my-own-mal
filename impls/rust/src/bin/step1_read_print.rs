use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

fn main() -> anyhow::Result<()> {
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

// TODO: read_str関数を呼び出す
#[allow(non_snake_case)]
fn READ(input: String) -> String {
    input
}

// MalTypeを受け取りMalTypeを返す
#[allow(non_snake_case)]
fn EVAL(input: String) -> String {
    input
}

// TODO: pr_str関数を呼び出す
#[allow(non_snake_case)]
fn PRINT(input: String) -> String {
    input
}

fn rep(input: String) -> String {
    PRINT(EVAL(READ(input)))
}

// 追加課題

// 1. 以下の型をサポートする
//  - MalString: '\'で'\'と'"'をエスケープする
//  - MalBool
//  - MalNil
// 2. 括弧の対応が取れていない場合はエラーを返す
// 3. リーダーマクロのサポート: tests/step1_read_print.malを参照
// 4. キーワード、ベクター、ハッシュマップのサポート
// 5. コメントのサポート: ';'から行末までをコメントとして扱う
