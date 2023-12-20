use crate::types::MalVal;
use itertools::Itertools;

pub fn pr_str(form: &MalVal) -> String {
    match form {
        MalVal::Nil => String::from("nil"),
        MalVal::Bool(b) => b.to_string(),
        MalVal::Number(n) => n.to_string(),
        MalVal::String(s) => format!("\"{}\"", escape(s)),
        MalVal::Keyword(k) => format!(":{}", k),
        MalVal::Symbol(s) => s.to_string(),
        MalVal::List(l, _) => format!("({})", l.iter().map(pr_str).join(" ")),
        MalVal::Vector(v, _) => format!("[{}]", v.iter().map(pr_str).join(" ")),
        MalVal::HashMap(m, _) => format!(
            "{{{}}}",
            m.iter()
                .map(|(k, v)| format!("{} {}", pr_str(k), pr_str(v)))
                .join(" ")
        ),
        MalVal::HashSet(s, _) => format!("#{{{}}}", s.iter().map(pr_str).join(" ")),
        MalVal::BuiltinFn(_) => String::from("#<function>"),
        MalVal::Func(_, _) => String::from("#<function>"),
    }
}

fn escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => [Some('\\'), Some('\"')],
            '\\' => [Some('\\'), Some('\\')],
            '\n' => [Some('\\'), Some('n')],
            '\r' => [Some('\\'), Some('r')],
            '\t' => [Some('\\'), Some('t')],
            _ => [Some(c), None],
        })
        .flat_map(|c| c.into_iter().flatten())
        .collect()
}
