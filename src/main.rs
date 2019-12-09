use calc::bignum::Calc;
use ansi_term::Colour::RGB;

fn main() {
    println!("{}", RGB(255, 0, 0).bold().paint("Calculator 1.3.0"));
    loop {
        let mut expr = String::new();
        std::io::stdin().read_line(&mut expr).expect("Failed to read line");
        let test = Calc::new(expr);
        match test.run_round(Some(5)) {
            Ok(value) => {
                println!("{}", RGB(30, 144, 255)
                    .bold().paint("=".to_owned() + &value));
            }
            Err(msg) => println!("{}", RGB(255, 0, 0).paint(msg)),
        }
    }
}
