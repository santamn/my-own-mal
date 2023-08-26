use crate::types::MalVal;
use itertools::Itertools;

// MalTypeを受け取り文字列に変換する
// - MalSymbol: 文字列を返す
// - MalNumber: 文字列を返す
// - MalList: リストの要素を文字列に変換し、それらをスペース区切りで連結した文字列を()で囲んで返す
pub fn pr_str(form: &MalVal) -> String {
    match form {
        MalVal::Nil => String::from("nil"),
        MalVal::Bool(b) => b.to_string(),
        MalVal::Number(n) => n.to_string(),
        MalVal::String(s) => format!("\"{}\"", s),
        MalVal::Keyword(k) => format!(":{}", k),
        MalVal::Symbol(s) => s.to_string(),
        MalVal::List(l, _) => format!("({})", l.iter().map(|e| pr_str(e)).join(" ")),
        MalVal::Vector(v, _) => format!("[{}]", v.iter().map(|e| pr_str(e)).join(" ")),
        MalVal::HashMap(m, _) => format!(
            "{{{}}}",
            m.iter()
                .map(|(k, v)| format!("{} => {}", pr_str(k), pr_str(v)))
                .join(", ")
        ),
        MalVal::HashSet(s, _) => format!("#{{{}}}", s.iter().map(|e| pr_str(e)).join(" ")),
    }
}
