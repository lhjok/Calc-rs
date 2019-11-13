use calc::bignum::Calculator;
use ansi_term::Colour::RGB;

fn main() {
    println!("{}", RGB(255, 0, 0).bold().paint("Calculator 1.2.0"));
    loop {
        let mut expr = String::new();
        std::io::stdin().read_line(&mut expr).expect("Failed to read line");
        let test = Calculator::new(expr);
        match test.run() {
            Ok(value) => {
                println!("{}", RGB(30, 144, 255).bold()
                    .paint("=".to_owned() + &Calculator::to_fixed_round(value, Some(5))));
                }
            Err(msg) => println!("{}", RGB(255, 0, 0).paint(msg)),
        }
    }
}
