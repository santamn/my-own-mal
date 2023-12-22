macro_rules! int_op {
    ($name:expr, $fn:ident) => {
        int_op!($name, |a, b| Ok($fn(a, b)))
    };
    ($name:expr, $fn:expr) => {
        crate::types::MalVal::BuiltinFn(|args| {
            args.into_iter()
                .try_reduce(|acc, x| match (acc, x) {
                    (crate::types::MalVal::Number(acc), crate::types::MalVal::Number(x)) => {
                        $f(acc, x)
                    }
                    (z, crate::types::MalVal::Number(_)) | (_, z) => {
                        Err(crate::types::MalError::InvalidType(
                            crate::printer::pr_str(&z),
                            "number".to_string(),
                            z.type_str(),
                        ))
                    }
                })?
                .ok_or(crate::types::MalError::WrongArity(
                    $name.to_string(),
                    crate::types::Arity::Variadic(1),
                    0,
                ))
        })
    };
}
