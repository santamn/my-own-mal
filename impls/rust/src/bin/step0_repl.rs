fn main() {
    loop {
        print!("user> ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        println!("{}", rep(input));
    }
}

fn read(input: String) -> String {
    input
}

fn eval(input: String) -> String {
    input
}

fn print(input: String) -> String {
    input
}

fn rep(input: String) -> String {
    print(eval(read(input)))
}
