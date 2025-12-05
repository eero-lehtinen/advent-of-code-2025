use std::{cell::RefCell, rc::Rc};

use log::trace;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Assign,
    Operator(Operator),
    LParen,
    RParen,
    LSquareParen,
    RSquareParen,
    LBrace,
    RBrace,
    Comma,
    Literal(Value),
    Ident(String),
    KeywordFor,
    KeywordIn,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
}

impl Operator {
    pub fn precedence(&self) -> u8 {
        match self {
            Operator::Plus | Operator::Minus => 0,
            Operator::Multiply | Operator::Divide => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(Rc<str>),
    List(Rc<RefCell<Vec<Value>>>),
}

pub fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut iter = source.chars().peekable();

    let mut tbuf = String::new();
    while let Some(ch) = iter.next() {
        match ch {
            '=' => tokens.push(Token::Assign),
            '+' => tokens.push(Token::Operator(Operator::Plus)),
            '-' => tokens.push(Token::Operator(Operator::Minus)),
            '*' => tokens.push(Token::Operator(Operator::Multiply)),
            '/' => tokens.push(Token::Operator(Operator::Divide)),
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            '[' => tokens.push(Token::LSquareParen),
            ']' => tokens.push(Token::RSquareParen),
            '{' => tokens.push(Token::LBrace),
            '}' => tokens.push(Token::RBrace),
            ',' => tokens.push(Token::Comma),
            '#' => {
                while let Some(&next_ch) = iter.peek() {
                    if next_ch == '\n' {
                        break;
                    }
                    iter.next();
                }
            }
            '"' => {
                tbuf.clear();
                let mut escape = false;
                for next_ch in iter.by_ref() {
                    if next_ch == '\\' && !escape {
                        escape = true;
                        continue;
                    }
                    if next_ch == '"' && !escape {
                        break;
                    }
                    if escape {
                        match next_ch {
                            'n' => tbuf.push('\n'),
                            't' => tbuf.push('\t'),
                            'r' => tbuf.push('\r'),
                            '\\' => tbuf.push('\\'),
                            '"' => tbuf.push('"'),
                            other => tbuf.push(other),
                        }
                    } else {
                        tbuf.push(next_ch);
                    }
                    escape = false;
                }
                tokens.push(Token::Literal(Value::String(tbuf.clone().into())));
                tbuf.clear();
            }
            ch if ch.is_alphabetic() => {
                tbuf.push(ch);
                while let Some(&next_ch) = iter.peek() {
                    if next_ch.is_alphanumeric() {
                        tbuf.push(next_ch);
                        iter.next();
                    } else {
                        break;
                    }
                }
                match tbuf.as_str() {
                    "for" => tokens.push(Token::KeywordFor),
                    "in" => tokens.push(Token::KeywordIn),
                    _ => tokens.push(Token::Ident(tbuf.clone())),
                }
                tbuf.clear();
            }
            ch if ch.is_ascii_digit() => {
                tbuf.push(ch);
                let mut is_float = false;
                while let Some(&next_ch) = iter.peek() {
                    if next_ch.is_ascii_digit() {
                        tbuf.push(next_ch);
                        iter.next();
                    } else if next_ch == '.' && !is_float {
                        is_float = true;
                        tbuf.push(next_ch);
                        iter.next();
                    } else {
                        break;
                    }
                }
                if is_float {
                    if let Ok(float_val) = tbuf.parse::<f64>() {
                        tokens.push(Token::Literal(Value::Float(float_val)));
                    } else {
                        panic!("Invalid float literal: {}", tbuf);
                    }
                } else if let Ok(int_val) = tbuf.parse::<i64>() {
                    tokens.push(Token::Literal(Value::Integer(int_val)));
                } else {
                    panic!("Invalid integer literal: {}", tbuf);
                }
                tbuf.clear();
            }
            ch if ch.is_whitespace() => {}
            _ => {
                panic!("Unexpected character: {}", ch);
            }
        }
    }

    trace!("Tokenized source:\n{:#?}", tokens);

    tokens
}
