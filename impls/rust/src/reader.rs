use crate::types::{MalError, MalResult, MalVal, Paren};
use fnv::{FnvHashMap, FnvHashSet};
use std::collections::LinkedList;
use std::iter::Peekable;
use std::rc::Rc;

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

// tokenize関数を呼び出しReaderオブジェクトを作成する
// その後、Readerオブジェクトを引数にしてread_str関数を呼び出す
pub fn read_str(input: String) -> MalResult {
    read_form(&mut tokenize(input).iter().peekable())
}

// 正規表現についてのメモ
//  - []: 括弧内の文字のいずれかにマッチ. []内では特殊文字をエスケープする必要がない
//  - (): グループ化. グループ化した文字列にマッチする
//  - (?:): non-capturing group. グループ化した文字列にマッチするが、グループ化した文字列を取得しない

// 文字列を受け取り、トークンのベクタを返す: 正規表現を使う
// malのトークンすべてにマッチする正規表現: [\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)
// - [\s,]*: 任意個の空白とカンマ
// - ~@: '~@'自体
// - #{: '#{'自体
// - [\[\]{}()'`~^@]: []{}()'`~^@のいずれか
// - "(?:\\.|[^\\"])*"?: "で囲まれた文字列(閉じていない場合を含む)
//  - \\.: \と任意の文字(エスケープされた文字)
//  - [^\\"]: \と"以外の任意の文字
// - ;.*: コメント行
// - [^\s\[\]{}('"`,;)]*: 空白と[]{}('"`,;)以外の任意の文字
fn tokenize(input: String) -> Vec<String> {
    regex!(r###"[\s,]*(~@|#{|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"###)
        .captures_iter(&input)
        .map(|cap| cap[1].to_string()) // cap[0]はマッチした文字列全体, cap[1]はグループ化した文字列=空白以外の部分
        .filter(|token| !token.starts_with(";")) // コメント行を除外
        .collect()
}

// ここまでLexer
// 以下がParser

fn read_form<I, S>(reader: &mut Peekable<I>) -> MalResult
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    match reader.peek().ok_or(MalError::NoInput)?.as_ref() {
        "(" => read_list(reader),
        "[" => read_vec(reader),
        "{" => read_hashmap(reader),
        "#{" => read_hashset(reader),
        _ => read_atom(reader),
    }
}

fn read_list<I, S>(reader: &mut Peekable<I>) -> MalResult
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let mut l = LinkedList::new();
    reader.next(); // '('を読み飛ばす
    while let Some(token) = reader.peek() {
        if token.as_ref() == ")" {
            reader.next(); // ')'を読み飛ばす
            return Ok(MalVal::List(Rc::new(l), Rc::new(MalVal::Nil)));
        }
        l.push_back(read_form(reader)?);
    }

    Err(MalError::Unbalanced(Paren::Round))
}

fn read_vec<I, S>(reader: &mut Peekable<I>) -> MalResult
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let mut v = Vec::new();
    reader.next(); // '['を読み飛ばす
    while let Some(token) = reader.peek() {
        if token.as_ref() == "]" {
            reader.next(); // ']'を読み飛ばす
            return Ok(MalVal::Vector(Rc::new(v), Rc::new(MalVal::Nil)));
        }
        v.push(read_form(reader)?);
    }

    Err(MalError::Unbalanced(Paren::Square))
}

fn read_hashmap<I, S>(reader: &mut Peekable<I>) -> MalResult
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let mut m = FnvHashMap::default();
    reader.next(); // "{"を読み飛ばす
    while let Some(token) = reader.peek() {
        if token.as_ref() == "}" {
            reader.next(); // "}"を読み飛ばす
            return Ok(MalVal::HashMap(Rc::new(m), Rc::new(MalVal::Nil)));
        }

        let key = read_form(reader)?;
        if let Ok(value) = read_form(reader) {
            m.insert(key, value);
        } else {
            return Err(MalError::OddMap(m.len() * 2 + 1));
        }
    }

    Err(MalError::Unbalanced(Paren::Curly))
}

fn read_hashset<I, S>(reader: &mut Peekable<I>) -> MalResult
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let mut s = FnvHashSet::default();
    reader.next(); // "#{"を読み飛ばす
    while let Some(token) = reader.peek() {
        if token.as_ref() == "}" {
            reader.next(); // "}"を読み飛ばす
            return Ok(MalVal::HashSet(Rc::new(s), Rc::new(MalVal::Nil)));
        }
        s.insert(read_form(reader)?);
    }

    Err(MalError::Unbalanced(Paren::Curly))
}

fn read_atom<I, S>(reader: &mut I) -> MalResult
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    match reader.next().unwrap().as_ref() {
        "nil" => Ok(MalVal::Nil),
        token => {
            if let Ok(b) = token.parse::<bool>() {
                return Ok(MalVal::Bool(b));
            } else if let Ok(n) = token.parse::<i64>() {
                return Ok(MalVal::Number(n));
            } else if token.starts_with("\"") {
                if token.ends_with("\"") {
                    return Ok(MalVal::String(token[1..token.len() - 1].to_string()));
                } else {
                    Err(MalError::UncloedQuote)
                }
            } else if token.starts_with(":") {
                Ok(MalVal::Keyword(token[1..].to_string()))
            } else {
                Ok(MalVal::Symbol(token.to_string()))
            }
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        assert_eq!(
            tokenize("(+ 134 234)".to_string()),
            vec!["(", "+", "134", "234", ")"]
        );
    }
}
