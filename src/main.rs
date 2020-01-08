use calc::Calc;
use ansi_term::Colour::RGB;

fn main() {
    println!("{}", RGB(255, 0, 0).bold().paint("Calculator 1.5.0"));
    println!("{}", RGB(98, 98, 98).bold().paint("============================================================"));
    println!("{}", RGB(96, 96, 96).paint("Pi= P, Func= abs cos sin tan csc sec cot coth cosh sinh sqrt"));
    println!("{}", RGB(96, 96, 96).paint("tanh sech csch acos asin atan acosh asinh atanh exp fac cbrt"));
    println!("{}", RGB(96, 96, 96).paint("ln log logx (Default: Radians), Ex= 18-(sin(30*P/180)+2^3)%5"));
    println!("{}", RGB(98, 98, 98).bold().paint("============================================================"));
    loop {
        let mut expr = String::new();
        std::io::stdin().read_line(&mut expr).expect("Failed to read line");
        let test = Calc::new(expr);
        match test.run_round(Some(7)) {
            Ok(value) => {
                println!("{}", RGB(30, 144, 255)
                    .bold().paint("=".to_owned() + &value));
            }
            Err(msg) => println!("{}", RGB(255, 0, 0).paint(msg)),
        }
    }
}
