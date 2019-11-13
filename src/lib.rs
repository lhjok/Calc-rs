pub mod bignum {
    use rug::ops::Pow;
    use rug::{float::Constant, Float};
    use std::{char::from_digit, cell::RefCell};
    use std::process::exit;

    #[derive(Clone)]
    enum Sign {
        Init,
        Data,
        Char
    }

    pub struct Calculator {
        sign: RefCell<Sign>,
        numbers: RefCell<Vec<Float>>,
        operator: RefCell<Vec<u8>>,
        expression: String,
    }

    impl Calculator {
        pub fn new(expr: String) -> Self {
            Calculator {
                sign: RefCell::new(Sign::Init),
                numbers: RefCell::new(Vec::new()),
                operator: RefCell::new(Vec::new()),
                expression: expr + "=",
            }
        }

        fn to_fixed(src: Float) -> String {

            let mut exp: i32 = 0;
            let (mut zero, mut x) = (0, 0);
            let (mut temp, mut res) = (String::new(), String::new());
            let fix: String = src.to_string_radix(10, None);

            for (i, v) in fix.as_bytes().iter().enumerate() {
                if v == &b'e' {
                    temp = fix[..i].to_string();
                    exp = fix[i+1..].parse::<i32>().unwrap();
                    break;
                }
            }

            if exp == 0 { temp = fix; }
            for (i, v) in temp.as_bytes().iter().enumerate() {
                if v == &b'.'{
                    let a = temp[..i].to_string();
                    let b = temp[i+1..].to_string();
                    res = a + &b;
                }
                zero += 1;
                if v != &b'0' { zero = 0; }
                if exp < 0 && v == &b'-' { x = 1; }
                if v == &b'-' && exp >= 0 { exp += 1; }
            }

            if exp < 0 {
                exp = exp.abs();
                for _ in 0..exp {
                    res.insert(x,'0');
                    exp -= 1;
                }
                if x != 0 { exp += 1; }
            }

            if exp == 0 && res.len() - zero <= 1 {
                return res[..res.len() - zero].to_string();
            } else if exp == 0 && res.len() - zero > 1 {
                res = res[..res.len() - zero].to_string();
                res.insert(1,'.');
                return res;
            }

            let uexp = exp as usize + 1;
            res.insert(uexp,'.');
            if uexp >= res.len() - 1 - zero {
                return res[..uexp].to_string();
            }
            res = res[..res.len() - zero].to_string();
            res
        }

        pub fn to_fixed_round(src: Float, digits: Option<usize>) -> String {
            let fix = Calculator::to_fixed(src);
            match digits {
                None => return fix,
                Some(x) => {

                    if x < 3 {
                        return "Set Accuracy Greater Than 2".to_string();
                    } else if let None = fix.find('.') {
                        return fix;
                    }

                    let mut dig: usize = 0;
                    let mut is: bool = false;
                    let mut res = String::new();

                    for (i, v) in fix.as_bytes().iter().enumerate() {
                        if v == &b'.'{
                            dig = 0;
                            is = true;
                        }
                        dig += 1;
                        if dig - 1 <= x && i == fix.len() {
                            return fix;
                        } else if is == true && dig - 1 == x && i < fix.len() {
                            let a = fix[i..i+1].parse::<u32>().unwrap();
                            let b = fix[i-1..i].parse::<u32>().unwrap();
                            let c = fix[i-2..i-1].parse::<u32>().unwrap();
                            res = fix[..i].to_string();
                            if a < 5 {
                                return res;
                            } else if a > 4 && b < 9 {
                                res.pop();
                                res.push(from_digit(b+1, 10).unwrap());
                                return res;
                            } else if a > 4 && b == 9 && c < 9 {
                                res.pop();
                                res.push(from_digit(0, 10).unwrap());
                                res.remove(res.len()-2);
                                res.insert(res.len()-1, from_digit(c+1, 10).unwrap());
                                return res;
                            }
                            break;
                        }
                    }

                    let mut n = 0;
                    let rev = res.chars().rev().collect::<String>();
                    if let Some(_) = res.find('-') {
                        n = 1;
                    }

                    for (i, v) in rev.as_bytes().iter().enumerate() {
                        if v == &b'.' || v == &b'-' {
                            continue;
                        }

                        if rev.len()-1-n == i || rev.len()-1-n == i && v == &b'9' {
                            let a = res.remove(0+n).to_digit(10).unwrap();
                            res.insert_str(0+n, &(a+1).to_string());
                            return res;
                        }

                        let mut b: u32 = 0;
                        let a = rev[i..i+1].parse::<u32>().unwrap();
                        let nonum = rev[i+1..i+2].as_bytes();

                        if nonum != &[b'.'] && nonum != &[b'-'] {
                            b = rev[i+1..i+2].parse::<u32>().unwrap();
                        }

                        if a == 9 {
                            res.remove(res.len()-1-i);
                            res.insert(res.len()-i, from_digit(0, 10).unwrap());
                            if b + 1 <= 9 && nonum != &[b'.'] && nonum != &[b'-'] {
                                res.remove(res.len()-2-i);
                                res.insert(res.len()-1-i, from_digit(b+1, 10).unwrap());
                                return res;
                            }
                        }
                    }
                    return fix;
                }
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
            let mut vernier: u8 = b'0'; //移动游标
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

                        if let Sign::Char | Sign::Init = self.sign.clone().into_inner() {
                            match intercept(locat, index) {
                                Ok(value) => num.borrow_mut().push(value),
                                Err(err) => return Err(err),
                            }
                            *self.sign.borrow_mut() = Sign::Data;
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
                        *self.sign.borrow_mut() = Sign::Char;
                        locat = index + 1;
                        vernier = b'B';
                        continue;
                    }

                    ch @ b'(' => {
                        if let Sign::Char | Sign::Init = self.sign.clone().into_inner() {
                            if vernier != b'A' && vernier != b'-' {
                                ope.borrow_mut().push(ch);
                                locat = index + 1;
                                bracket = bracket + 1;
                                vernier = b'(';
                                continue;
                            }
                        }
                        return Err("Expression error".to_string());
                    }

                    b')' => {
                        if let Sign::Char | Sign::Init = self.sign.clone().into_inner() {
                            if vernier == b'A' {
                                match intercept(locat, index) {
                                    Ok(value) => num.borrow_mut().push(value),
                                    Err(err) => return Err(err),
                                }
                                *self.sign.borrow_mut() = Sign::Data;
                            }
                        }

                        if let Sign::Data = self.sign.clone().into_inner() {
                            if bracket > 0 {
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
                        }
                        return Err("Expression error".to_string());
                    }

                    b'=' | b'\n' | b'\r' => {
                        if vernier == b'0' {
                            return Err("Empty expression".to_string());
                        } else if bracket > 0 || vernier == b'-' || vernier == b'B' {
                            return Err("Expression error".to_string());
                        }

                        if let Sign::Char | Sign::Init = self.sign.clone().into_inner() {
                            match intercept(locat, index) {
                                Ok(value) => num.borrow_mut().push(value),
                                Err(err) => return Err(err),
                            }
                            *self.sign.borrow_mut() = Sign::Data;
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
                        if let Sign::Char | Sign::Init = self.sign.clone().into_inner()
                        {
                            if vernier != b'B' && vernier != 0 || vernier == b'F' || vernier == b')'
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

                                *self.sign.borrow_mut() = Sign::Data;
                                locat = index + 1;
                                vernier = b'F';
                                continue;
                            }
                        }
                        return Err("Expression error".to_string());
                    }

                    b'P' => {
                        if let Sign::Char | Sign::Init = self.sign.clone().into_inner() {
                            if vernier != b'A' {
                                let pi2 = if vernier == b'-' {
                                    Float::with_val(128, 0.0 - &pi)
                                } else {
                                    Float::with_val(128, &pi)
                                };
                                num.borrow_mut().push(pi2);
                                *self.sign.borrow_mut() = Sign::Data;
                                locat = index + 1;
                                vernier = b'F';
                                continue;
                            }
                            return Err("Expression error".to_string());
                        }
                    }

                    _ => return Err("Operator error".to_string()),
                }
            }
            Err("Error".to_string())
        }
    }
}
