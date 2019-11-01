use rug::ops::Pow;
use rug::{float::Constant, Float};
use std::cell::RefCell;
use std::process::exit;

pub struct Calculator {
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
        if x < &0.0 {
            res = m.ceil();
        } else {
            res = m.floor();
        }
        x - res * n
    }

    pub fn run(&self) -> Result<Float, String> {

        let mut sign: u8 = 0; //入栈签名
        let mut vernier: u8 = 0; //移动游标
        let mut locat: usize = 0; //切片定位
        let mut bracket: u32 = 0; //括号标记
        let expr = &self.expression;
        let bytes = self.expression.as_bytes();
        let pi = Float::with_val(128, Constant::Pi);
        let max = Float::with_val(2560, Float::parse("1e+768").unwrap());
        let min = Float::with_val(2560, Float::parse("-1e+768").unwrap());

        let computing = |x: &u8| -> Result<Float, String> {
            let c1 = self.numbers.borrow_mut().pop().unwrap();
            let c2 = self.numbers.borrow_mut().pop().unwrap();
            match x {
                b'+' => {
                    let res = c2 + c1;
                    if max >= res && min <= res {
                        return Ok(res);
                    }
                    return Err("Beyond the precision range".to_string());
                }
                b'-' => {
                    let res = c2 - c1;
                    if max >= res && min <= res {
                        return Ok(res);
                    }
                    return Err("Beyond the precision range".to_string());
                }
                b'*' => {
                    let res = c2 * c1;
                    if max >= res && min <= res {
                        return Ok(res);
                    }
                    return Err("Beyond the precision range".to_string());
                }
                b'/' => {
                    if c1 != 0.0 {
                        let res = c2 / c1;
                        if max >= res && min <= res {
                            return Ok(res);
                        }
                        return Err("Beyond the precision range".to_string());
                    }
                    return Err("Divide by zero".to_string());
                }
                b'%' => {
                    if c1 != 0.0 {
                        let res = Calculator::fmod(&c2, &c1);
                        if max >= res && min <= res {
                            return Ok(res);
                        }
                        return Err("Beyond the precision range".to_string());
                    }
                    return Err("Divide by zero".to_string());
                }
                b'^' => {
                    let res = c2.pow(c1);
                    if max >= res && min <= res {
                        return Ok(res);
                    }
                    return Err("Beyond the precision range".to_string());
                }
                _ => exit(0),
            }
        };

        for (index, &value) in bytes.iter().enumerate() {
            match value {
                b'0'..=b'9' | b'.' => {
                    if vernier != b')' && vernier != b'F' {
                        vernier = b'A';
                        continue;
                    }
                    return Err("Expression error".to_string());
                }

                ch @ b'+' | ch @ b'-' | ch @ b'*' | ch @ b'/' | ch @ b'%' | ch @ b'^' => {

                    let negative1 = ch == b'-' && vernier == b'(' && vernier != b'A';
                    let negative2 = ch == b'-' && vernier != b')' && vernier != b'A' && vernier != b'F' && vernier != b'-';

                    if negative1 == true || negative2 == true {
                        vernier = b'-';
                        continue;
                    } else if vernier != b'A' && vernier != b')' && vernier != b'F' {
                        return Err("Expression error".to_string());
                    }

                    if sign != b'A' {
                        match Float::parse(&expr[locat..index]) {
                            Ok(valid) => {
                                let value = Float::with_val(2560, valid);
                                if max < value || min > value {
                                    return Err("Beyond the precision range".to_string());
                                }
                                self.numbers.borrow_mut().push(value);
                                sign = b'A';
                            }
                            Err(_) => return Err("Invalid number".to_string()),
                        }
                    }

                    while self.operator.borrow().len() != 0 && self.operator.borrow().last().unwrap() != &b'(' {
                        let p1 = Calculator::priority(self.operator.borrow().last().unwrap());
                        let p2 = Calculator::priority(&ch);
                        if p1 >= p2 {
                            let res = computing(self.operator.borrow().last().unwrap());
                            match res {
                                Ok(_) => {
                                    self.numbers.borrow_mut().push(res.unwrap());
                                    self.operator.borrow_mut().pop();
                                }
                                Err(_) => return res,
                            }
                        } else {
                            break;
                        }
                    }

                    self.operator.borrow_mut().push(ch);
                    locat = index + 1;
                    vernier = b'B';
                    sign = b'B';
                    continue;
                }

                ch @ b'(' => {
                    if sign != b'A' && vernier != b'A' && vernier != b'-' {
                        self.operator.borrow_mut().push(ch);
                        locat = index + 1;
                        bracket = bracket + 1;
                        vernier = b'(';
                        continue;
                    }
                    return Err("Expression error".to_string());
                }

                b')' => {
                    if sign != b'A' && vernier == b'A' {
                        match Float::parse(&expr[locat..index]) {
                            Ok(valid) => {
                                let value = Float::with_val(2560, valid);
                                if max < value || min > value {
                                    return Err("Beyond the precision range".to_string());
                                }
                                self.numbers.borrow_mut().push(value);
                                sign = b'A';
                            }
                            Err(_) => return Err("Invalid number".to_string()),
                        }
                    }

                    if bracket > 0 && sign == b'A' {
                        while self.operator.borrow().last().unwrap() != &b'(' {
                            let res = computing(&self.operator.borrow_mut().pop().unwrap());
                            match res {
                                Ok(_) => self.numbers.borrow_mut().push(res.unwrap()),
                                Err(_) => return res,
                            }
                        }

                        self.operator.borrow_mut().pop();
                        locat = index + 1;
                        bracket = bracket - 1;
                        vernier = b')';
                        continue;
                    }
                    return Err("Expression error".to_string());
                }

                b'=' | b'\n' => {
                    if vernier == 0 {
                        return Err("Empty expression".to_string());
                    } else if bracket > 0 || vernier == b'-' || vernier == b'B' {
                        return Err("Expression error".to_string());
                    }

                    if sign != b'A' {
                        match Float::parse(&expr[locat..index]) {
                            Ok(valid) => {
                                let value = Float::with_val(2560, valid);
                                if max < value || min > value {
                                    return Err("Beyond the precision range".to_string());
                                }
                                self.numbers.borrow_mut().push(value);
                                sign = b'A';
                            }
                            Err(_) => return Err("Invalid number".to_string()),
                        }
                    }

                    while self.operator.borrow().len() != 0 {
                        let res = computing(&self.operator.borrow_mut().pop().unwrap());
                        match res {
                            Ok(_) => self.numbers.borrow_mut().push(res.unwrap()),
                            Err(_) => return res,
                        }
                    }

                    let res = self.numbers.borrow_mut().pop().unwrap();
                    return Ok(res);
                }

                ch @ b'A' | ch @ b'S' | ch @ b'c' | ch @ b's' | ch @ b't'
                | ch @ b'C' | ch @ b'I' | ch @ b'T' | ch @ b'l' | ch @ b'L' | ch @ b'E' => {

                    if sign != b'A' && vernier != b'B' && vernier != 0 || vernier == b'F' || vernier == b')' {
                        let mut res = Float::new(2560);
                        if vernier != b'F' && vernier != b')' {
                            match Float::parse(&expr[locat..index]) {
                                Ok(valid) => {
                                    let value = Float::with_val(2560, valid);
                                    if max < value || min > value {
                                        return Err("Beyond the precision range".to_string());
                                    }
                                    if ch == b'l' || ch == b'L' || ch == b'S' && value < 0.0 {
                                        return Err("Expression error".to_string());
                                    }
                                    match ch {
                                        b'A' => res = value.abs(),
                                        b'c' => res = value.cos(),
                                        b's' => res = value.sin(),
                                        b't' => res = value.tan(),
                                        b'C' => res = value.cosh(),
                                        b'I' => res = value.sinh(),
                                        b'T' => res = value.tanh(),
                                        b'E' => res = value.exp(),
                                        b'l' => res = value.ln(),
                                        b'L' => res = value.log2(),
                                        b'S' => res = value.sqrt(),
                                        _ => return Err("Operator error".to_string()),
                                    }
                                }
                                Err(_) => return Err("Invalid number".to_string()),
                            }
                        } else {
                            let value = self.numbers.borrow_mut().pop().unwrap();
                            if ch == b'l' || ch == b'L' || ch == b'S' && value < 0.0 {
                                return Err("Expression error".to_string());
                            }
                            match ch {
                                b'A' => res = value.abs(),
                                b'c' => res = value.cos(),
                                b's' => res = value.sin(),
                                b't' => res = value.tan(),
                                b'C' => res = value.cosh(),
                                b'I' => res = value.sinh(),
                                b'T' => res = value.tanh(),
                                b'E' => res = value.exp(),
                                b'l' => res = value.ln(),
                                b'L' => res = value.log2(),
                                b'S' => res = value.sqrt(),
                                _ => return Err("Operator error".to_string()),
                            }
                        }

                        self.numbers.borrow_mut().push(res);
                        locat = index + 1;
                        vernier = b'F';
                        sign = b'A';
                        continue;
                    }
                    return Err("Expression error".to_string());
                }

                b'P' => {
                    if sign != b'A' && vernier != b'A' {
                        let pi2 = if vernier == b'-' {
                            Float::with_val(128, 0.0 - &pi)
                        } else {
                            Float::with_val(128, &pi)
                        };
                        self.numbers.borrow_mut().push(pi2);
                        locat = index + 1;
                        vernier = b'F';
                        sign = b'A';
                        continue;
                    }
                    return Err("Expression error".to_string());
                }

                _ => return Err("Operator error".to_string()),
            }
        }
        Err("Possible error".to_string())
    }
}
