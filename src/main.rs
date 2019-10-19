use rug::ops::Pow;
use rug::{float::Constant, Float};
use std::process::exit;
use std::cell::RefCell;
use ansi_term::Colour::RGB;

struct Calculator {
    numbers: RefCell<Vec<Float>>,
    operator: RefCell<Vec<u8>>,
    expression: String,
}

impl Calculator {
    pub fn new(expr: String) -> Self {
        Calculator {
            numbers: RefCell::new(Vec::new()),
            operator: RefCell::new(Vec::new()),
            expression: expr,
        }
    }

    fn priority(x: &u8) -> u8 {
        match x {
            b'+' | b'-' => 1,
            b'*' | b'/' | b'%' => 2,
            b'^' => 3,
            _ => exit(0),
        }
    }

    fn fmod(x: &Float, n: &Float) -> Float {
        let m = Float::with_val(2560, x / n);
        let mut res = Float::new(2560);
        if x < 0.0 {
            res = m.ceil();
        } else {
            res = m.floor();
        }
        x - res * n
    }

    pub fn run(&self) -> Result<Float, String> {

        let mut sign: u8 = 0;   //入栈签名
        let mut vernier: u8 = 0;   //移动游标
        let mut locat: usize = 0;    //切片定位
        let mut bracket: u32 = 0;    //括号标记
        let expr = &self.expression;
        let bytes = self.expression.as_bytes();
        let pi = Float::with_val(128, Constant::Pi);
        let max = Float::with_val(2560, Float::parse("1e+768").unwrap());
        let min = Float::with_val(2560, Float::parse("-1e+768").unwrap());

        let computing = |x: &u8| -> Result<Float, String> {
            match x {
                b'+' => {
                    let c1 = self.numbers.borrow_mut().pop().unwrap();
                    let c2 = self.numbers.borrow_mut().pop().unwrap();
                    let res = c2 + c1;
                    if max >= res && min <= res {    //控制在绝对精度范围。
                        return Ok(res);
                    }
                    return Err("Beyond the precision range".to_string());
                }

                b'-' => {
                    let c1 = self.numbers.borrow_mut().pop().unwrap();
                    let c2 = self.numbers.borrow_mut().pop().unwrap();
                    let res = c2 - c1;
                    if max >= res && min <= res {    //控制在绝对精度范围。
                        return Ok(res);
                    }
                    return Err("Beyond the precision range".to_string());
                }

                b'*' => {
                    let c1 = self.numbers.borrow_mut().pop().unwrap();
                    let c2 = self.numbers.borrow_mut().pop().unwrap();
                    let res = c2 * c1;
                    if max >= res && min <= res {    //控制在绝对精度范围。
                        return Ok(res);
                    }
                    return Err("Beyond the precision range".to_string());
                }

                b'/' => {
                    let c1 = self.numbers.borrow_mut().pop().unwrap();
                    let c2 = self.numbers.borrow_mut().pop().unwrap();
                    if c1 == 0.0 {
                        return Err("Divide by zero".to_string());
                    }
                    let res = c2 / c1;
                    if max >= res && min <= res {    //控制在绝对精度范围。
                        return Ok(res);
                    }
                    return Err("Beyond the precision range".to_string());
                }

                b'%' => {
                    let c1 = self.numbers.borrow_mut().pop().unwrap();
                    let c2 = self.numbers.borrow_mut().pop().unwrap();
                    if c1 == 0.0 {
                        return Err("Divide by zero".to_string());
                    }
                    let res = Calculator::fmod(&c2, &c1);
                    if max >= res && min <= res {    //控制在绝对精度范围。
                        return Ok(res);
                    }
                    return Err("Beyond the precision range".to_string());
                }

                b'^' => {
                    let c1 = self.numbers.borrow_mut().pop().unwrap();
                    let c2 = self.numbers.borrow_mut().pop().unwrap();
                    let res = c2.pow(c1);
                    if max >= res && min <= res {    //控制在绝对精度范围。
                        return Ok(res);
                    }
                    return Err("Beyond the precision range".to_string());
                }

                _ => exit(0),
            }
        };

        for (index, &value) in bytes.iter().enumerate() {
            match value {
                b'0' ..= b'9' | b'.' => {
                    if vernier != b')' && vernier != b'F' {    //数字前是右括号、函数，则表达式错误。
                        vernier = b'A';
                        continue
                    }
                    return Err("Expression error".to_string());
                }

                ch @ b'+' | ch @ b'-' | ch @ b'*' | ch @ b'/' | ch @ b'%' | ch @ b'^' => {

                    let negative1 = ch == b'-' && vernier == b'(' && vernier != b'A';
                    let negative2 = ch == b'-' && vernier != b')' && vernier != b'A' && vernier != b'F' && vernier != b'-';

                    if negative1 == true || negative2 == true {    //判断是减号还是负号
                        vernier = b'-';
                        continue
                    }else if vernier != b'A' && vernier != b')' && vernier != b'F' {    //运算符前非数字、非右括号、非函数，则表达式错误。
                        return Err("Expression error".to_string());
                    }

                    if sign != b'A' {
                        match Float::parse(&expr[locat..index]) {    //将运算符前的数字取出来
                            Ok(valid) => {
                                let value = Float::with_val(2560, valid);
                                if max < value || min > value {    //控制在绝对精度范围。
                                    return Err("Beyond the precision range".to_string());
                                }
                                self.numbers.borrow_mut().push(value);    //读取的数字进栈
                                sign = b'A';
                            }
                            Err(_) => return Err("Invalid number".to_string()),
                        }
                    }

                    while self.operator.borrow().len() != 0 && self.operator.borrow().last().unwrap() != &b'(' {
                        let p1 = Calculator::priority(self.operator.borrow().last().unwrap());
                        let p2 = Calculator::priority(&ch);
                        if p1 >= p2 {    //优先级比较
                            let res = computing(self.operator.borrow().last().unwrap());    //调用二元运算函数
                            match res {
                                Ok(_) => {
                                    self.numbers.borrow_mut().push(res.unwrap());    //运算结果进栈
                                    self.operator.borrow_mut().pop();    //运算符出栈
                                }
                                Err(_) => return res,
                            }
                        } else {
                            break
                        }
                    }

                    self.operator.borrow_mut().push(ch);     //运算符进栈
                    locat = index + 1;    //移动切片定位
                    vernier = b'B';
                    sign = b'B';
                    continue
                }

                ch @ b'(' => {
                    if sign != b'A' && vernier != b'A' && vernier != b'-' {    //左括号前如果是数字或负号，则表达式错误。
                        self.operator.borrow_mut().push(ch);     //左括号直接进栈
                        locat = index + 1;   //移动切片定位
                        bracket = bracket + 1;
                        vernier = b'(';
                        continue
                    }
                    return Err("Expression error".to_string());
                }

                b')' => {
                    if sign != b'A' && vernier == b'A' {    //上一次进栈是运算符，同时括号前必须是数字。
                        match Float::parse(&expr[locat..index]) {    //将运算符前的数字取出来
                            Ok(valid) => {
                                let value = Float::with_val(2560, valid);
                                if max < value || min > value {    //控制在绝对精度范围。
                                    return Err("Beyond the precision range".to_string());
                                }
                                self.numbers.borrow_mut().push(value);   //读取的数字进栈
                                sign = b'A';
                            }
                            Err(_) => return Err("Invalid number".to_string()),
                        }
                    }

                    if bracket > 0 && sign == b'A' {    //运算符栈中必须有左括号，右括号前必须是数字。
                        while self.operator.borrow().last().unwrap() != &b'(' {    //遇到左括号时停止循环
                            let res = computing(&self.operator.borrow_mut().pop().unwrap());    //调用二元运算函数
                            match res {
                                Ok(_) => self.numbers.borrow_mut().push(res.unwrap()),    //运算结果进栈
                                Err(_) => return res,
                            }
                        }

                        self.operator.borrow_mut().pop();     //左括号出栈
                        locat = index + 1;     //移动切片定位
                        bracket = bracket - 1;
                        vernier = b')';
                        continue
                    }
                    return Err("Expression error".to_string());
                }

                b'=' | b'\n' => {
                    if vernier == 0 {   //未输入表达式
                        return Err("Empty expression".to_string());
                    } else if bracket > 0 || vernier == b'-' || vernier == b'B' {   //等号前还有左括号、负号、运算符，则表达式错误。
                        return Err("Expression error".to_string());
                    }

                    if sign != b'A' {
                        match Float::parse(&expr[locat..index]) {    //将运算符前的数字取出来
                            Ok(valid) => {
                                let value = Float::with_val(2560, valid);
                                if max < value || min > value {    //控制在绝对精度范围。
                                    return Err("Beyond the precision range".to_string());
                                }
                                self.numbers.borrow_mut().push(value);   //读取的数字进栈
                                sign = b'A';
                            }
                            Err(_) => return Err("Invalid number".to_string()),
                        }
                    }

                    while self.operator.borrow().len() != 0 {     //直到运算符栈为空停止循环
                        let res = computing(&self.operator.borrow_mut().pop().unwrap());     //调用二元运算函数
                        match res {
                            Ok(_) => self.numbers.borrow_mut().push(res.unwrap()),    //运算结果进栈
                            Err(_) => return res,
                        }
                    }

                    let res = self.numbers.borrow_mut().pop().unwrap();     //清空最后一个数据栈
                    return Ok(res);
                }

                b'A' => {
                    if sign != b'A' && vernier != b'B' && vernier != 0 || vernier == b'F' || vernier == b')' {
                        let mut res = Float::new(2560);
                        if vernier != b'F' && vernier != b')' {
                            match Float::parse(&expr[locat..index]) {    //将运算符前的数字取出来
                                Ok(valid) => {
                                    let value = Float::with_val(2560, valid);
                                    if max < value || min > value {    //控制在绝对精度范围。
                                        return Err("Beyond the precision range".to_string());
                                    }
                                    res = value.abs();
                                }   //读取的数字进栈
                                Err(_) => return Err("Invalid number".to_string()),
                            }
                        } else {
                            res = self.numbers.borrow_mut().pop().unwrap().abs();
                        }

                        self.numbers.borrow_mut().push(res);
                        locat = index + 1;     //移动切片定位
                        vernier = b'F';
                        sign = b'A';
                        continue
                    }
                    return Err("Expression error".to_string());
                }

                b'S' => {
                    if sign != b'A' && vernier != b'B' && vernier != 0 || vernier == b'F' || vernier == b')' {
                        let mut res = Float::new(2560);
                        if vernier != b'F' && vernier != b')' {
                            match Float::parse(&expr[locat..index]) {    //将运算符前的数字取出来
                                Ok(valid) => {
                                    let value = Float::with_val(2560, valid);
                                    if max < value || min > value {    //控制在绝对精度范围。
                                        return Err("Beyond the precision range".to_string());
                                    }
                                    res = value.sqrt();
                                }   //读取的数字进栈
                                Err(_) => return Err("Invalid number".to_string()),
                            }
                        } else {
                            res = self.numbers.borrow_mut().pop().unwrap().sqrt();
                        }

                        if res >= 0.0 {
                            self.numbers.borrow_mut().push(res);
                            locat = index + 1;     //移动切片定位
                            vernier = b'F';
                            sign = b'A';
                            continue
                        }
                        return Err("Expression error".to_string());
                    }
                    return Err("Expression error".to_string());
                }

                b'P' => {
                    if sign != b'A' && vernier != b'A' {     //标记符前面必须是字符或者为空
                        let pi2 = if vernier == b'-' {
                            Float::with_val(2560, 0.0 - &pi)
                        }else {
                            Float::with_val(2560, &pi)
                        };
                        self.numbers.borrow_mut().push(pi2);    //pi2进入数字栈
                        locat = index + 1;    //移动切片定位
                        vernier = b'F';
                        sign = b'A';
                        continue
                    }
                    return Err("Expression error".to_string());
                }

                _ => return Err("Operator error".to_string()),
            }
        }
        Err("Possible error".to_string())
    }
}

fn main() {
    println!("{}", RGB(255, 0, 0).bold().paint("Calculator 1.0.0"));
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
