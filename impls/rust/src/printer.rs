use crate::types::MalVal;
use itertools::Itertools;

pub fn pr_str(form: &MalVal, print_readably: bool) -> String {
    match form {
        MalVal::Nil => String::from("nil"),
        MalVal::Bool(b) => b.to_string(),
        MalVal::Number(n) => n.to_string(),
        MalVal::String(s) => {
            if print_readably {
                format!("\"{}\"", escape(s))
            } else {
                s.to_string()
            }
        }
        MalVal::Keyword(k) => format!(":{}", k),
        MalVal::Symbol(s) => s.to_string(),
        MalVal::List(l, _) => format!(
            "({})",
            l.iter().map(|e| pr_str(e, print_readably)).join(" ")
        ),
        MalVal::Vector(v, _) => format!(
            "[{}]",
            v.iter().map(|v| pr_str(v, print_readably)).join(" ")
        ),
        MalVal::HashMap(m, _) => format!(
            "{{{}}}",
            m.iter()
                .map(|(k, v)| format!(
                    "{} {}",
                    pr_str(k, print_readably),
                    pr_str(v, print_readably)
                ))
                .join(" ")
        ),
        MalVal::HashSet(s, _) => format!(
            "#{{{}}}",
            s.iter().map(|s| pr_str(s, print_readably)).join(" ")
        ),
        MalVal::BuiltinFn(_) | MalVal::Func(_, _) => String::from("#<function>"),
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
