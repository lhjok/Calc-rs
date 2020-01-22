use rug::ops::Pow;
use rug::{float::Constant, Float};
use std::{char::from_digit, cell::RefCell};
use std::collections::HashMap;
use std::process::exit;
use lazy_static::lazy_static;

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

#[macro_use]
lazy_static! {
    static ref MAX: Float = {
        let max = Float::parse("1e+768").unwrap();
        Float::with_val(2560, max)
    }; 
    static ref MIN: Float = {
        let min = Float::parse("-1e+768").unwrap();
        Float::with_val(2560, min)
    };
}

trait Symbol {
    fn priority(&self) -> u8;
    fn computing(&self, n: &Calc) -> Result<Float, String>;
}

trait Bignum {
    fn fmod(&self, n: &Float) -> Float;
    fn accuracy(&self) -> Result<Float, String>;
    fn to_fixed(&self) -> String;
    fn to_fixed_round(&self, n: Option<usize>) -> String;
}

trait Other {
    fn clean_zero(&self) -> String;
    fn math(&self, v: &Float) -> Result<Float, String>;
}

impl Symbol for u8 {
    fn priority(&self) -> u8 {
        match self {
            b'+' | b'-' => 1,
            b'*' | b'/' | b'%' => 2,
            b'^' => 3,
            _ => exit(0)
        }
    }

    fn computing(&self, num: &Calc) -> Result<Float, String> {
        let c1 = num.numbers.borrow_mut().pop().unwrap();
        let c2 = num.numbers.borrow_mut().pop().unwrap();
        match self {
            b'+' => Float::with_val(2560, &c2 + &c1).accuracy(),
            b'-' => Float::with_val(2560, &c2 - &c1).accuracy(),
            b'*' => Float::with_val(2560, &c2 * &c1).accuracy(),
            b'/' if &c1 != &0.0 => Float::with_val(2560, &c2 / &c1).accuracy(),
            b'%' if &c1 != &0.0 => c2.fmod(&c1).accuracy(),
            b'^' => Float::with_val(2560, &c2.pow(&c1)).accuracy(),
            _ => Err("Divide By Zero".to_string())
        }
    }
}

impl Bignum for Float {
    fn fmod(&self, n: &Float) -> Float {
        let mut m = Float::with_val(2560, self / n);
        if self < &0.0 { m.ceil_mut() }
        else { m.floor_mut() };
        Float::with_val(2560, self - &m * n)
    }

    fn accuracy(&self) -> Result<Float, String> {
        if *MAX > *self && *MIN < *self {
            return Ok(self.clone());
        }
        Err("Beyond Accuracy".to_string())
    }

    fn to_fixed(&self) -> String {
        let mut exp: i32 = 0;
        let (mut zero, mut i_or_u) = (0, 0);
        let (mut temp, mut res) = (String::new(), String::new());
        let fix: String = self.to_string_radix(10, None);

        for (i, v) in fix.as_bytes().iter().enumerate() {
            if v == &b'e' {
                temp = fix[..i].to_string();
                exp = fix[i+1..].parse::<i32>().unwrap();
                break;
            }
        }

        if exp == 0 { temp = fix };
        for (i, v) in temp.as_bytes().iter().enumerate() {
            match v {
                b'.' => {
                    res = temp[..i].to_string();
                    res += &temp[i+1..]; zero = 0;
                },
                b'-' => {
                    if exp < 0 { i_or_u = 1 }
                    else { exp += 1 }; zero = 0;
                },
                b'0' => zero += 1,
                _ => zero = 0,
            }
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

    fn to_fixed_round(&self, digits: Option<usize>) -> String {
        let fix = self.to_fixed();
        match digits {
            None => fix,
            Some(x) => {
                if let None = fix.find('.') {
                    return fix;
                } else if x < 3 {
                    return "Set Accuracy Greater Than 2".to_string();
                }

                let mut n: usize = 0;
                let mut dig: usize = 0;
                let mut point: bool = false;
                let mut res = String::new();

                for (i, v) in fix.as_bytes().iter().enumerate() {
                    match v {
                        b'-' => n = 1,
                        b'.' => { dig = 0; point = true; },
                        _ => dig += 1,
                    }
                    if dig < x && i == fix.len()-1 {
                        return fix;
                    } else if point == true && dig == x && i <= fix.len()-1 {
                        let a = fix[i..i+1].parse::<u32>().unwrap();
                        let b = fix[i-1..i].parse::<u32>().unwrap();
                        let c = fix[i-2..i-1].parse::<u32>().unwrap();
                        res = fix[..i].to_string();
                        if a < 5 {
                            return res.clean_zero();
                        } else if a > 4 && b < 9 {
                            res.pop();
                            res.push(from_digit(b+1, 10).unwrap());
                            return res;
                        } else if a > 4 && b == 9 && c < 9 {
                            res.pop(); res.pop();
                            res.push(from_digit(c+1, 10).unwrap());
                            return res;
                        }
                        break;
                    }
                }

                let rev = res.chars().rev().collect::<String>();
                for (i, v) in rev.as_bytes().iter().enumerate() {
                    if v == &b'.' {
                        continue;
                    }
                    let a = rev[i..i+1].parse::<u32>().unwrap();
                    if a == 9 {
                        res.remove(res.len()-1-i);
                        if i == rev.len()-1-n {
                            res.insert_str(0+n, &(a+1).to_string());
                            return res.clean_zero();
                        }
                        res.insert(res.len()-i, from_digit(0, 10).unwrap());
                    } else if a < 9 {
                        res.remove(res.len()-1-i);
                        res.insert(res.len()-i, from_digit(a+1, 10).unwrap());
                        return res.clean_zero();
                    }
                    let point = rev[i+1..i+2].as_bytes();
                    if point == &[b'.'] {
                        continue;
                    }
                    let b = rev[i+1..i+2].parse::<u32>().unwrap();
                    if b < 9 {
                        res.remove(res.len()-2-i);
                        res.insert(res.len()-1-i, from_digit(b+1, 10).unwrap());
                        return res.clean_zero();
                    }
                }
                exit(0)
            }
        }
    }
}

impl Other for String {
    fn clean_zero(&self) -> String {
        let mut find: bool = false;
        let (mut zero, mut dig) = (0, 0);
        for valid in self.as_bytes().iter() {
            match valid {
                b'0' => { dig += 1; zero += 1; },
                b'.' => { dig = 0; find = true; zero = 0; },
                _ => { dig += 1; zero = 0; },
            }
        }
        if find == true {
            if zero == dig {
                return self[..self.len()-dig-1].to_string();
            }
            return self[..self.len()-zero].to_string();
        }
        self.clone()
    }

    fn math(&self, v: &Float) -> Result<Float, String> {
        match self {
            _ if self == "abs" => Float::with_val(2560, v.abs_ref()).accuracy(),
            _ if self == "ln" && v > &0.0 => Float::with_val(2560, v.ln_ref()).accuracy(),
            _ if self == "exp" => Float::with_val(2560, v.exp_ref()).accuracy(),
            _ if self == "log" && v > &0.0 => Float::with_val(2560, v.log2_ref()).accuracy(),
            _ if self == "logx" && v > &0.0 => Float::with_val(2560, v.log10_ref()).accuracy(),
            _ if self == "cos" => Float::with_val(2560, v.cos_ref()).accuracy(),
            _ if self == "sin" => Float::with_val(2560, v.sin_ref()).accuracy(),
            _ if self == "tan" => Float::with_val(2560, v.tan_ref()).accuracy(),
            _ if self == "csc" && v != &0.0 => Float::with_val(2560, v.csc_ref()).accuracy(),
            _ if self == "sec" => Float::with_val(2560, v.sec_ref()).accuracy(),
            _ if self == "cot" && v != &0.0 => Float::with_val(2560, v.cot_ref()).accuracy(),
            _ if self == "cosh" => Float::with_val(2560, v.cosh_ref()).accuracy(),
            _ if self == "sinh" => Float::with_val(2560, v.sinh_ref()).accuracy(),
            _ if self == "tanh" => Float::with_val(2560, v.tanh_ref()).accuracy(),
            _ if self == "csch" && v != &0.0 => Float::with_val(2560, v.csch_ref()).accuracy(),
            _ if self == "sech" => Float::with_val(2560, v.sech_ref()).accuracy(),
            _ if self == "coth" && v != &0.0 => Float::with_val(2560, v.coth_ref()).accuracy(),
            _ if self == "acos" && v >= &-1.0 && v <= &1.0 => Float::with_val(2560, v.acos_ref()).accuracy(),
            _ if self == "asin" && v >= &-1.0 && v <= &1.0 => Float::with_val(2560, v.asin_ref()).accuracy(),
            _ if self == "atan" => Float::with_val(2560, v.atan_ref()).accuracy(),
            _ if self == "acosh" && v >= &1.0 => Float::with_val(2560, v.acosh_ref()).accuracy(),
            _ if self == "asinh" => Float::with_val(2560, v.asinh_ref()).accuracy(),
            _ if self == "atanh" && v > &-1.0 && v < &1.0 => Float::with_val(2560, v.atanh_ref()).accuracy(),
            _ if self == "cbrt" => Float::with_val(2560, v.cbrt_ref()).accuracy(),
            _ if self == "sqrt" && v >= &0.0 => Float::with_val(2560, v.sqrt_ref()).accuracy(),
            _ if self == "fac" => {
                let fac = Float::factorial(v.to_u32_saturating().unwrap());
                Float::with_val(2560, fac).accuracy()
            },
            _ => Err("Parameter Error".to_string())
        }
    }
}

impl Calc {
    pub fn new(expr: String) -> Self {
        Self {
            sign: RefCell::new(Sign::Init),
            numbers: RefCell::new(Vec::new()),
            operator: RefCell::new(Vec::new()),
            func: RefCell::new(HashMap::new()),
            expression: expr+"=",
        }
    }

    fn extract(&self, n: usize, i: usize) -> Result<Float, String> {
        match Float::parse(&self.expression[n..i]) {
            Ok(valid) => Float::with_val(2560, valid).accuracy(),
            Err(_) => Err("Invalid Number".to_string())
        }
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
        let math = ["abs","cos","sin","tan","csc","sec","cot","coth",
        "cosh","sinh","tanh","sech","ln","csch","acos","asin","atan",
        "acosh","asinh","atanh","exp","log","logx","sqrt","cbrt","fac"];
        let pi = Float::with_val(128, Constant::Pi);

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
                    if nega == true && ch == b'-' {
                        mark = b'-';
                        continue;
                    } else if mark != b'N' && mark != b')' && mark != b'P' {
                        return Err("Expression Error".to_string());
                    }

                    if let Sign::Char | Sign::Init = sign.clone().into_inner() {
                        num.borrow_mut().push(self.extract(locat, index)?);
                        *sign.borrow_mut() = Sign::Data;
                    }

                    while ope.borrow().len() != 0 && ope.borrow().last().unwrap() != &b'(' {
                        if ope.borrow().last().unwrap().priority() >= ch.priority() {
                            let bo = ope.borrow_mut().pop().unwrap();
                            let value = bo.computing(self)?;
                            num.borrow_mut().push(value);
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
                        for value in math.iter() {
                            if value == &valid {
                                func.borrow_mut().insert(bracket+1, valid.clone());
                                find = true;
                                break;
                            }
                        }
                        if find == false {
                            return Err("Function Undefined".to_string());
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
                            num.borrow_mut().push(self.extract(locat, index)?);
                            *sign.borrow_mut() = Sign::Data;
                        }
                    }

                    if let Sign::Data = sign.clone().into_inner() {
                        if bracket > 0 {
                            while ope.borrow().last().unwrap() != &b'(' {
                                let bo = ope.borrow_mut().pop().unwrap();
                                let value = bo.computing(self)?;
                                num.borrow_mut().push(value);
                            }

                            if let Some(fun) = func.borrow_mut().remove(&bracket) {
                                let value = fun.math(&num.borrow_mut().pop().unwrap())?;
                                num.borrow_mut().push(value);
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
                        num.borrow_mut().push(self.extract(locat, index)?);
                        *sign.borrow_mut() = Sign::Data;
                    }

                    while ope.borrow().len() != 0 {
                        let bo = ope.borrow_mut().pop().unwrap();
                        let value = bo.computing(self)?;
                        num.borrow_mut().push(value);
                    }
                    let res = num.borrow_mut().pop().unwrap();
                    return Ok(res);
                }

                b'P' => {
                    if let Sign::Char | Sign::Init = sign.clone().into_inner() {
                        if mark != b'N' && mark != b'F' {
                            let value = if mark == b'-' {
                                Float::with_val(128, 0.0 - &pi)
                            } else { pi.clone() };
                            num.borrow_mut().push(value);
                            *sign.borrow_mut() = Sign::Data;
                            locat = index + 1;
                            mark = b'P';
                            continue;
                        }
                    }
                    return Err("Expression Error".to_string());
                }

                _ => return Err("Operator Undefined".to_string())
            }
        }
        Err("No Terminator".to_string())
    }

    pub fn run_round(&self, digits: Option<usize>) -> Result<String, String> {
        match self.run() {
            Ok(value) => Ok(value.to_fixed_round(digits)),
            Err(err) => Err(err)
        }
    }
}
