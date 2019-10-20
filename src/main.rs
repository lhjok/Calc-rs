extern crate Calc;
use Calc::Calculator;
use ansi_term::Colour::RGB;

fn main() {
    println!("{}", RGB(255, 0, 0).bold().paint("Calculator 1.2.0"));
    loop {    //让程序一直运行，除非终止进程，或者恐慌。
        let mut expr = String::new();
        std::io::stdin().read_line(&mut expr).expect("Failed to read line");
        let test = Calculator::new(expr);
        match test.run() {    //在终端打印单次计算结果，或者错误信息。
            Ok(value) => println!("{}", RGB(30, 144, 255).bold().paint("=".to_owned() + &value.to_string_radix(10, Some(5)))),
            Err(msg) => println!("{}", RGB(255, 0, 0).paint(msg)),
        }
    }
}
