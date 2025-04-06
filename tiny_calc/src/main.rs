mod error;
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::io::Write;
#[derive(Debug, Clone, Copy, PartialEq)]
enum TokenType {
    Num,
    Name,
    Plus,
    Minus,
    Times,
    Lparen,
    Rparen,
    Assign,
}

#[derive(Debug, Clone)]
struct Token {
    token_type: TokenType,
    val: String,
}
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
fn tokenize(source: &str) -> Result<Vec<Token>> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut n: usize = 0;
    let source_chars: Vec<char> = source.chars().collect();
    while n < source_chars.len() {
        if source_chars[n].is_whitespace() {
            n += 1;
            continue;
        } else if source_chars[n].is_ascii_digit() {
            let start = n;
            while n < source_chars.len() && source_chars[n].is_ascii_digit() {
                n += 1;
            }
            tokens.push(Token {
                token_type: TokenType::Num,
                val: source_chars[start..n].iter().collect(),
            });
        } else if source_chars[n].is_ascii_alphabetic() {
            let start = n;
            while n < source_chars.len() && source_chars[n].is_ascii_alphabetic() {
                n += 1;
            }
            tokens.push(Token {
                token_type: TokenType::Name,
                val: source_chars[start..n].iter().collect(),
            });
        } else {
            let token = match source_chars[n] {
                '+' => Ok(Token {
                    token_type: TokenType::Plus,
                    val: String::from('+'),
                }),
                '*' => Ok(Token {
                    token_type: TokenType::Times,
                    val: String::from('*'),
                }),

                '-' => Ok(Token {
                    token_type: TokenType::Minus,
                    val: String::from('-'),
                }),
                '(' => Ok(Token {
                    token_type: TokenType::Lparen,
                    val: String::from('('),
                }),
                ')' => Ok(Token {
                    token_type: TokenType::Rparen,
                    val: String::from(')'),
                }),
                '=' => Ok(Token {
                    token_type: TokenType::Assign,
                    val: String::from('='),
                }),
                _ => Err(Error::SyntaxError(
                    format!("Couldn't parse {} to a token", source_chars[n]).to_string(),
                )),
            };
            tokens.push(token?);
            n += 1;
        }
    }

    Ok(tokens)
}

#[derive(Debug)]
enum Expr {
    Number {
        n: i32,
    },
    Variable {
        name: String,
    },
    Assign {
        location: Box<Expr>,
        value: Box<Expr>,
    },
    Add {
        left: Box<Expr>,
        right: Box<Expr>,
    },

    Minus {
        left: Box<Expr>,
        right: Box<Expr>,
    },

    Mul {
        left: Box<Expr>,
        right: Box<Expr>,
    },
}
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
struct Parser {
    tokens: Vec<Token>,
    n: usize,
}

impl Parser {
    fn accept(&mut self, token_type: TokenType) -> bool {
        if self.n < self.tokens.len() && self.tokens[self.n].token_type == token_type {
            self.n += 1;
            return true;
        }
        false
    }
    fn last(&self) -> Result<Token> {
        if self.n >= self.tokens.len() {
            return Err(Error::SyntaxError("Syntax error somewhere.".to_string()));
        }
        Ok(self.tokens[self.n - 1].clone())
    }
    fn at_end(&self) -> bool {
        self.n <= self.tokens.len()
    }
}

fn parse_term(p: &mut Parser) -> Result<Expr> {
    if p.accept(TokenType::Num) {
        Ok(Expr::Number {
            n: p.last()?.val.parse().expect("couldn't parse digit"),
        })
    } else if p.accept(TokenType::Name) {
        Ok(Expr::Variable {
            name: p.last()?.val,
        })
    } else if p.accept(TokenType::Lparen) {
        let e = parse_expression(p)?;
        if !p.accept(TokenType::Rparen) {
            Err(Error::SyntaxError(format!(
                "( not closed by a ). Found ( {e} "
            )))
        } else {
            Ok(e)
        }
    } else {
        Err(Error::SyntaxError("Cannot process token".to_string()))
    }
}

fn parse_expression(p: &mut Parser) -> Result<Expr> {
    let left = Box::new(parse_term(p)?);
    if p.accept(TokenType::Plus) {
        Ok(Expr::Add {
            left,
            right: Box::new(parse_term(p)?),
        })
    } else if p.accept(TokenType::Minus) {
        Ok(Expr::Minus {
            left,
            right: Box::new(parse_term(p)?),
        })
    } else if p.accept(TokenType::Times) {
        Ok(Expr::Mul {
            left,
            right: Box::new(parse_term(p)?),
        })
    } else if p.accept(TokenType::Assign) {
        Ok(Expr::Assign {
            location: left,
            value: Box::new(parse_expression(p)?),
        })
    } else {
        Ok(*left)
    }
}

fn parse(p: &mut Parser) -> Result<Expr> {
    let e = parse_expression(p)?;
    if !p.at_end() {
        return Err(Error::SyntaxError(
            format!(
                "Unprocessed characters remain. Last unprocessed: {}",
                p.last()?
            )
            .to_string(),
        ));
    }
    Ok(e)
}

struct Environment {
    vars: HashMap<String, i32>,
}

impl Environment {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }
    fn assign(&mut self, name: &str, val: i32) {
        self.vars.insert(name.to_string(), val);
    }
    fn lookup(&self, name: &str) -> i32 {
        *self.vars.get(name).unwrap()
    }
}

fn evaluate(expr: &Expr, env: &mut Environment) -> Result<i32> {
    let out = match expr {
        Expr::Number { n } => *n,
        Expr::Variable { name } => env.lookup(name),
        Expr::Assign { location, value } => match **location {
            Expr::Variable { ref name } => {
                let eval = evaluate(value, env)?;
                env.assign(name, eval);
                Ok(env.lookup(name))
            }
            _ => Err(Error::SyntaxError(format!("{}{}", location, value))),
        }?,
        Expr::Add { left, right } => evaluate(left, env)? + evaluate(right, env)?,
        Expr::Minus { left, right } => evaluate(left, env)? - evaluate(right, env)?,
        Expr::Mul { left, right } => evaluate(left, env)? * evaluate(right, env)?,
    };
    Ok(out)
}

fn main() {
    let mut env = Environment::new();
    loop {
        print!("calc > ");
        io::stdout().flush().unwrap();

        let mut raw_calc = String::new();

        io::stdin()
            .read_line(&mut raw_calc)
            .expect("Failed to read line");

        if raw_calc.trim().is_empty() {
            break;
        };

        let tokens = match tokenize(&raw_calc) {
            Ok(tokens) => tokens,
            Err(e) => {
                println!("{:?}", e);
                continue;
            }
        };
        println!("tokens: {:?}", tokens);

        let mut p = Parser {
            tokens: tokens.clone(),
            n: 0,
        };

        let parsed = match parse(&mut p) {
            Ok(parsed) => parsed,
            Err(e) => {
                println!("{:?}", e);
                continue;
            }
        };
        println!("parsed: {:?}", parsed);
        let out = match evaluate(&parsed, &mut env) {
            Ok(out) => out,
            Err(e) => {
                println!("{:?}", e);
                continue;
            }
        };
        println!("{out}");
    }
}
