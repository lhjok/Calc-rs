pub mod bignum {
    use rug::ops::Pow;
    use rug::{float::Constant, Float};
    use std::{char::from_digit, cell::RefCell};
    use std::collections::HashMap;
    use std::process::exit;

    #[derive(Clone)]
    enum Sign {
        Init,
        Data,
        Char
    }

    pub struct Calc {
        sign: RefCell<Sign>,
        numbers: RefCell<Vec<Float>>,
        operator: RefCell<Vec<u8>>,
        func: RefCell<HashMap<u32, String>>,
        expression: String,
    }

    impl Calc {
        pub fn new(expr: String) -> Self {
            Calc {
                sign: RefCell::new(Sign::Init),
                numbers: RefCell::new(Vec::new()),
                operator: RefCell::new(Vec::new()),
                func: RefCell::new(HashMap::new()),
                expression: expr + "=",
            }
        }

        fn clean_zero(src: String) -> String {
            let mut find: bool = false;
            let (mut zero, mut dig) = (0, 0);
            let mut res = src;
            for v in res.as_bytes().iter() {
                dig += 1;
                zero += 1;
                if v == &b'.'{
                    dig = 0;
                    find = true;
                }
                if v != &b'0' {
                    zero = 0;
                }
            }
            if find == true {
                if zero == dig {
                    return res[..res.len()-dig-1].to_string();
                }
                res = res[..res.len()-zero].to_string();
            }
            res
        }

        fn to_fixed(src: Float) -> String {
            let mut exp: i32 = 0;
            let (mut zero, mut i_or_u) = (0, 0);
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
                if exp < 0 && v == &b'-' { i_or_u = 1; }
                if exp >= 0 && v == &b'-' { exp += 1; }
            }

            if exp < 0 {
                exp = exp.abs();
                for _ in 0..exp {
                    res.insert(i_or_u, '0');
                    exp -= 1;
                }
                if i_or_u != 0 {
                    exp += 1;
                }
            }

            if exp == 0 && res.len()-zero == 1 {
                return res[..res.len()-zero].to_string();
            } else if exp == 0 && res.len()-zero > 1 {
                res = res[..res.len()-zero].to_string();
                res.insert(1, '.');
                return res;
            }

            let u_exp = exp as usize + 1;
            res.insert(u_exp, '.');
            if u_exp >= res.len()-1-zero {
                return res[..u_exp].to_string();
            }
            res[..res.len()-zero].to_string()
        }

        pub fn to_fixed_round(src: Float, digits: Option<usize>) -> String {
            let fix = Calc::to_fixed(src);
            match digits {
                None => fix,
                Some(x) => {
                    if let None = fix.find('.') {
                        return fix;
                    } else if x < 3 {
                        return "Set Accuracy Greater Than 2".to_string();
                    }
                    let mut dig: usize = 0;
                    let mut point: bool = false;
                    let mut res = String::new();
                    for (i, v) in fix.as_bytes().iter().enumerate() {
                        dig += 1;
                        if v == &b'.'{
                            dig = 0;
                            point = true;
                        }
                        if dig < x && i == fix.len()-1 {
                            return fix;
                        } else if point == true && dig == x && i <= fix.len()-1 {
                            let a = fix[i..i+1].parse::<u32>().unwrap();
                            let b = fix[i-1..i].parse::<u32>().unwrap();
                            let c = fix[i-2..i-1].parse::<u32>().unwrap();
                            res = fix[..i].to_string();
                            if a < 5 {
                                return Calc::clean_zero(res);
                            } else if a > 4 && b < 9 {
                                res.pop();
                                res.push(from_digit(b+1, 10).unwrap());
                                return res;
                            } else if a > 4 && b == 9 && c < 9 {
                                res.pop();
                                res.pop();
                                res.push(from_digit(c+1, 10).unwrap());
                                return res;
                            }
                            break;
                        }
                    }

                    let mut n = 0;
                    if let Some(_) = res.find('-') {
                        n = 1;
                    }

                    let rev = res.chars().rev().collect::<String>();
                    for (i, v) in rev.as_bytes().iter().enumerate() {
                        if v == &b'.' || v == &b'-' {
                            continue;
                        }

                        if i == rev.len()-1-n {
                            let a = res.remove(0+n).to_digit(10).unwrap();
                            res.insert_str(0+n, &(a+1).to_string());
                            return Calc::clean_zero(res);
                        }

                        let a = rev[i..i+1].parse::<u32>().unwrap();
                        let mut b: u32 = 0;
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
                                return Calc::clean_zero(res);
                            }
                        }
                    }
                    exit(0)
                }
            }
        }

        fn priority(x: &u8) -> u8 {
            match x {
                b'+' | b'-' => 1,
                b'*' | b'/' | b'%' => 2,
                b'^' => 3,
                _ => exit(0)
            }
        }

        fn fmod(x: &Float, n: &Float) -> Float {
            let m = Float::with_val(2560, x / n);
            let res = if x < &0.0 {
                m.ceil()
            } else { m.floor() };
            x - res * n
        }

        pub fn run(&self) -> Result<Float, String> {
            let sign = &self.sign;
            let num = &self.numbers;
            let ope = &self.operator;
            let expr = &self.expression;
            let func = &self.func;
            let mut locat: usize = 0;
            let mut bracket: u32 = 0;
            let mut mark: u8 = b'I'; // I = Init, C = Char, N = Number, F = Func, P = Pi
            let pi = Float::with_val(128, Constant::Pi);
            let funcs = ["abs","cos","sin","tan","sec","cosh","sinh","tanh","sech",
                "acos","asin","atan","acosh","asinh","atanh","exp","ln","log","logx","sqrt"];
            let max = Float::with_val(2560, Float::parse("1e+768").unwrap());
            let min = Float::with_val(2560, Float::parse("-1e+768").unwrap());

            let accurate = |value: Float| -> Result<Float, String> {
                if max > value && min < value {
                    return Ok(value);
                }
                Err("Beyond The Precision Range".to_string())
            };

            let computing = |x: u8| -> Result<Float, String> {
                let c1 = num.borrow_mut().pop().unwrap();
                let c2 = num.borrow_mut().pop().unwrap();
                match x {
                    b'+' => accurate(c2 + c1),
                    b'-' => accurate(c2 - c1),
                    b'*' => accurate(c2 * c1),
                    b'/' if c1 != 0.0 => accurate(c2 / c1),
                    b'%' if c1 != 0.0 => accurate(Calc::fmod(&c2, &c1)),
                    b'^' => accurate(c2.pow(c1)),
                    _ => Err("Divide By Zero".to_string())
                }
            };

            let intercept = |n: usize, i: usize| -> Result<Float, String> {
                match Float::parse(&expr[n..i]) {
                    Ok(valid) => accurate(Float::with_val(2560, valid)),
                    Err(_) => Err("Invalid Number".to_string())
                }
            };

            let maths = |n: String, v: Float| -> Result<Float, String> {
                match n {
                    _ if n == "abs" => accurate(v.abs()),
                    _ if n == "cos" => accurate(v.cos()),
                    _ if n == "sin" => accurate(v.sin()),
                    _ if n == "tan" => accurate(v.tan()),
                    _ if n == "sec" => accurate(v.sec()),
                    _ if n == "atan" => accurate(v.atan()),
                    _ if n == "cosh" => accurate(v.cosh()),
                    _ if n == "sinh" => accurate(v.sinh()),
                    _ if n == "tanh" => accurate(v.tanh()),
                    _ if n == "sech" => accurate(v.sech()),
                    _ if n == "asinh" => accurate(v.asinh()),
                    _ if n == "exp" => accurate(v.exp()),
                    _ if n == "ln" && v > 0.0 => accurate(v.ln()),
                    _ if n == "log" && v > 0.0 => accurate(v.log2()),
                    _ if n == "logx" && v > 0.0 => accurate(v.log10()),
                    _ if n == "acosh" && v >= 1.0 => accurate(v.acosh()),
                    _ if n == "sqrt" && v >= 0.0 => accurate(v.sqrt()),
                    _ if n == "acos" && v >= -1.0 && v <= 1.0 => accurate(v.acos()),
                    _ if n == "asin" && v >= -1.0 && v <= 1.0 => accurate(v.asin()),
                    _ if n == "atanh" && v > -1.0 && v < 1.0 => accurate(v.atanh()),
                    _ => Err("Expression Error".to_string())
                }
            };

            for (index, &valid) in expr.as_bytes().iter().enumerate() {
                match valid {
                    b'0'..=b'9' | b'.' => {
                        if mark != b')' && mark != b'P' && mark != b'F' {
                            mark = b'N';
                            continue;
                        }
                        return Err("Expression Error".to_string());
                    }

                    b'a'..=b'z' => {
                        if mark != b')' && mark != b'P' && mark != b'-' && mark != b'N' {
                            mark = b'F';
                            continue;
                        }
                        return Err("Expression Error".to_string());
                    }

                    ch @ b'+' | ch @ b'-' | ch @ b'*' | ch @ b'/' | ch @ b'%' | ch @ b'^' => {
                        let nega: bool = mark == b'I' || mark == b'(' || mark == b'C';
                        if ch == b'-' && nega == true {
                            mark = b'-';
                            continue;
                        } else if mark != b'N' && mark != b')' && mark != b'P' {
                            return Err("Expression Error".to_string());
                        }

                        if let Sign::Char | Sign::Init = sign.clone().into_inner() {
                            match intercept(locat, index) {
                                Ok(value) => num.borrow_mut().push(value),
                                Err(err) => return Err(err)
                            }
                            *sign.borrow_mut() = Sign::Data;
                        }

                        while ope.borrow().len() != 0 && ope.borrow().last().unwrap() != &b'(' {
                            let p1 = Calc::priority(ope.borrow().last().unwrap());
                            let p2 = Calc::priority(&ch);
                            if p1 >= p2 {
                                let valid = computing(ope.borrow_mut().pop().unwrap());
                                match valid {
                                    Ok(value) => num.borrow_mut().push(value),
                                    Err(err) => return Err(err)
                                }
                            } else {
                                break;
                            }
                        }

                        ope.borrow_mut().push(ch);
                        *sign.borrow_mut() = Sign::Char;
                        locat = index + 1;
                        mark = b'C';
                        continue;
                    }

                    ch @ b'(' => {
                        if mark == b'F' {
                            let mut find: bool = false;
                            let valid = expr[locat..index].to_string();
                            for value in funcs.iter() {
                                if value == &valid {
                                    func.borrow_mut().insert(bracket+1, valid.clone());
                                    find = true;
                                    break;
                                }
                            }
                            if find == false {
                                return Err("Expression Error".to_string());
                            }
                        }

                        if let Sign::Char | Sign::Init = sign.clone().into_inner() {
                            if mark != b'N' && mark != b'-' {
                                ope.borrow_mut().push(ch);
                                locat = index + 1;
                                bracket = bracket + 1;
                                mark = b'(';
                                continue;
                            }
                        }
                        return Err("Expression Error".to_string());
                    }

                    b')' => {
                        if let Sign::Char | Sign::Init = sign.clone().into_inner() {
                            if mark == b'N' {
                                match intercept(locat, index) {
                                    Ok(value) => num.borrow_mut().push(value),
                                    Err(err) => return Err(err)
                                }
                                *sign.borrow_mut() = Sign::Data;
                            }
                        }

                        if let Sign::Data = sign.clone().into_inner() {
                            if bracket > 0 {
                                while ope.borrow().last().unwrap() != &b'(' {
                                    let valid = computing(ope.borrow_mut().pop().unwrap());
                                    match valid {
                                        Ok(value) => num.borrow_mut().push(value),
                                        Err(err) => return Err(err)
                                    }
                                }

                                if let Some(fun_name) = func.borrow_mut().remove(&bracket) {
                                    let valid = num.borrow_mut().pop().unwrap();
                                    match maths(fun_name, valid) {
                                        Ok(value) => num.borrow_mut().push(value),
                                        Err(err) => return Err(err)
                                    }
                                }

                                ope.borrow_mut().pop();
                                locat = index + 1;
                                bracket = bracket - 1;
                                mark = b')';
                                continue;
                            }
                        }
                        return Err("Expression Error".to_string());
                    }

                    b'=' | b'\n' | b'\r' => {
                        if mark == b'I' {
                            return Err("Empty Expression".to_string());
                        } else if bracket > 0 || mark == b'-' || mark == b'C' || mark == b'F' {
                            return Err("Expression Error".to_string());
                        }

                        if let Sign::Char | Sign::Init = sign.clone().into_inner() {
                            match intercept(locat, index) {
                                Ok(value) => num.borrow_mut().push(value),
                                Err(err) => return Err(err)
                            }
                            *sign.borrow_mut() = Sign::Data;
                        }

                        while ope.borrow().len() != 0 {
                            let valid = computing(ope.borrow_mut().pop().unwrap());
                            match valid {
                                Ok(value) => num.borrow_mut().push(value),
                                Err(err) => return Err(err)
                            }
                        }
                        let res = num.borrow_mut().pop().unwrap();
                        return Ok(res);
                    }

                    b'P' => {
                        if let Sign::Char | Sign::Init = sign.clone().into_inner() {
                            if mark != b'N' && mark != b'F' {
                                let pi2 = if mark == b'-' {
                                    Float::with_val(128, 0.0 - &pi)
                                } else {
                                    Float::with_val(128, &pi)
                                };
                                num.borrow_mut().push(pi2);
                                *sign.borrow_mut() = Sign::Data;
                                locat = index + 1;
                                mark = b'P';
                                continue;
                            }
                        }
                        return Err("Expression Error".to_string());
                    }

                    _ => return Err("Operator Error".to_string())
                }
            }
            Err("No Terminator".to_string())
        }

        pub fn run_round(&self ,digits: Option<usize>) -> Result<String, String> {
            match self.run() {
                Ok(value) => Ok(Calc::to_fixed_round(value, digits)),
                Err(err) => Err(err)
            }
        }
    }
}
