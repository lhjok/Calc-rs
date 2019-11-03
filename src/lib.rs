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
            expression: expr + "=",
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
        let res = if x < &0.0 { m.ceil() }
        else { m.floor() };
        x - res * n
    }

    pub fn run(&self) -> Result<Float, String> {

        let num = &self.numbers;
        let ope = &self.operator;
        let expr = &self.expression;
        let mut locat: usize = 0; //切片定位
        let mut bracket: u32 = 0; //括号标记
        let (mut sign, mut vernier) = (b'0', b'0'); //入栈签名，移动游标。
        let pi = Float::with_val(128, Constant::Pi);
        let max = Float::with_val(2560, Float::parse("1e+768").unwrap());
        let min = Float::with_val(2560, Float::parse("-1e+768").unwrap());

        let computing = |x: &u8| -> Result<Float, String> {
            let c1 = num.borrow_mut().pop().unwrap();
            let c2 = num.borrow_mut().pop().unwrap();
            if x == &b'/' && c1 == 0.0 || x == &b'%' && c1 == 0.0 {
                return Err("Divide by zero".to_string());
            }
            let accurate = |value: Float| -> Result<Float, String> {
                if max < value || min > value {
                    return Err("Beyond the precision range".to_string());
                }
                return Ok(value);
            };
            match x {
                b'+' => return accurate(c2 + c1),
                b'-' => return accurate(c2 - c1),
                b'*' => return accurate(c2 * c1),
                b'/' => return accurate(c2 / c1),
                b'%' => return accurate(Calculator::fmod(&c2, &c1)),
                b'^' => return accurate(c2.pow(c1)),
                _ => return Err("Error".to_string()),
            }
        };

        let intercept = |n: usize, i: usize| -> Result<Float, String> {
            match Float::parse(&expr[n..i]) {
                Ok(valid) => {
                    let value = Float::with_val(2560, valid);
                    if max < value || min > value {
                        return Err("Beyond the precision range".to_string());
                    }
                    return Ok(value);
                }
                Err(_) => return Err("Invalid number".to_string()),
            }
        };

        let maths = |ch: u8, value: Float| -> Result<Float, String> {
            if ch == b'l' && value < 0.0 || ch == b'L'
               && value < 0.0 || ch == b'S' && value < 0.0 {
                return Err("Expression error".to_string());
            }
            match ch {
                b'A' => return Ok(value.abs()),
                b'c' => return Ok(value.cos()),
                b's' => return Ok(value.sin()),
                b't' => return Ok(value.tan()),
                b'C' => return Ok(value.cosh()),
                b'I' => return Ok(value.sinh()),
                b'T' => return Ok(value.tanh()),
                b'E' => return Ok(value.exp()),
                b'l' => return Ok(value.ln()),
                b'L' => return Ok(value.log2()),
                b'S' => return Ok(value.sqrt()),
                _ => return Err("Error".to_string()),
            }
        };

        for (index, &value) in expr.as_bytes().iter().enumerate() {
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
                    let negative2 = ch == b'-' && vernier != b')' && vernier != b'A'
                        && vernier != b'F' && vernier != b'-';

                    if negative1 == true || negative2 == true {
                        vernier = b'-';
                        continue;
                    } else if vernier != b'A' && vernier != b')' && vernier != b'F' {
                        return Err("Expression error".to_string());
                    }

                    if sign != b'A' {
                        match intercept(locat, index) {
                            Ok(value) => num.borrow_mut().push(value),
                            Err(err) => return Err(err),
                        }
                        sign = b'A';
                    }

                    while ope.borrow().len() != 0 && ope.borrow().last().unwrap() != &b'(' {
                        let p1 = Calculator::priority(ope.borrow().last().unwrap());
                        let p2 = Calculator::priority(&ch);
                        if p1 >= p2 {
                            let res = computing(ope.borrow().last().unwrap());
                            match res {
                                Ok(_) => {
                                    num.borrow_mut().push(res.unwrap());
                                    ope.borrow_mut().pop();
                                }
                                Err(_) => return res,
                            }
                        } else {
                            break;
                        }
                    }

                    ope.borrow_mut().push(ch);
                    locat = index + 1;
                    vernier = b'B';
                    sign = b'B';
                    continue;
                }

                ch @ b'(' => {
                    if sign != b'A' && vernier != b'A' && vernier != b'-' {
                        ope.borrow_mut().push(ch);
                        locat = index + 1;
                        bracket = bracket + 1;
                        vernier = b'(';
                        continue;
                    }
                    return Err("Expression error".to_string());
                }

                b')' => {
                    if sign != b'A' && vernier == b'A' {
                        match intercept(locat, index) {
                            Ok(value) => num.borrow_mut().push(value),
                            Err(err) => return Err(err),
                        }
                        sign = b'A';
                    }

                    if bracket > 0 && sign == b'A' {
                        while ope.borrow().last().unwrap() != &b'(' {
                            let res = computing(&ope.borrow_mut().pop().unwrap());
                            match res {
                                Ok(_) => num.borrow_mut().push(res.unwrap()),
                                Err(_) => return res,
                            }
                        }

                        ope.borrow_mut().pop();
                        locat = index + 1;
                        bracket = bracket - 1;
                        vernier = b')';
                        continue;
                    }
                    return Err("Expression error".to_string());
                }

                b'=' | b'\n' | b'\r' => {
                    if vernier == 0 {
                        return Err("Empty expression".to_string());
                    } else if bracket > 0 || vernier == b'-' || vernier == b'B' {
                        return Err("Expression error".to_string());
                    }

                    if sign != b'A' {
                        match intercept(locat, index) {
                            Ok(value) => num.borrow_mut().push(value),
                            Err(err) => return Err(err),
                        }
                        sign = b'A';
                    }

                    while ope.borrow().len() != 0 {
                        let res = computing(&ope.borrow_mut().pop().unwrap());
                        match res {
                            Ok(_) => num.borrow_mut().push(res.unwrap()),
                            Err(_) => return res,
                        }
                    }

                    let res = num.borrow_mut().pop().unwrap();
                    return Ok(res);
                }

                ch @ b'A' | ch @ b'S' | ch @ b'c' | ch @ b's' | ch @ b't' | ch @ b'C'
                | ch @ b'I' | ch @ b'T' | ch @ b'l' | ch @ b'L' | ch @ b'E' =>
                {
                    if sign != b'A' && vernier != b'B' && vernier != 0
                       || vernier == b'F' || vernier == b')' 
                    {
                        if vernier != b'F' && vernier != b')' {
                            match intercept(locat, index) {
                                Ok(valid) => {
                                    match maths(ch, valid) {
                                        Ok(value) => num.borrow_mut().push(value),
                                        Err(err) => return Err(err),
                                    }
                                }
                                Err(err) => return Err(err),
                            }
                        } else {
                            match maths(ch, num.borrow_mut().pop().unwrap()) {
                                Ok(value) => num.borrow_mut().push(value),
                                Err(err) => return Err(err),
                            }
                        }

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
                        num.borrow_mut().push(pi2);
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
        Err("Error".to_string())
    }
}
