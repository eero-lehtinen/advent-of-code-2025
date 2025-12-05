use std::iter::Peekable;

use log::trace;

use crate::tokenizer::{Operator, Token, Value};

#[derive(Debug)]
pub enum AstNode {
    Assign(String, Box<AstNode>),
    FunctionCall(String, Vec<AstNode>),
    BinaryOp(Box<AstNode>, Operator, Box<AstNode>),
    List(Vec<AstNode>),
    Literal(Value),
    Variable(String),
}

fn parse_comma_separated_list<'a, I: Iterator<Item = &'a Token> + Clone>(
    iter: &mut Peekable<I>,
    end_token: Token,
) -> Vec<AstNode> {
    let mut elements = Vec::new();
    loop {
        let next = iter.peek();
        if next == Some(&&end_token) {
            iter.next();
            break;
        }
        let element = parse_expression(iter).unwrap();
        elements.push(element);
        let next = iter.peek();
        if let Some(Token::Comma) = next {
            iter.next();
        } else if next == Some(&&end_token) {
            iter.next();
            break;
        } else {
            panic!("Expected ',' or ']/)' in list");
        }
    }
    elements
}

fn parse_function_call<'a, I: Iterator<Item = &'a Token> + Clone>(
    ident: &str,
    iter: &mut Peekable<I>,
) -> Option<AstNode> {
    let next = iter.peek();
    let Some(Token::LParen) = next else {
        return None;
    };
    iter.next();
    let args = parse_comma_separated_list(iter, Token::RParen);
    Some(AstNode::FunctionCall(ident.to_owned(), args))
}

fn parse_primary_expression<'a, I: Iterator<Item = &'a Token> + Clone>(
    iter: &mut Peekable<I>,
) -> Option<AstNode> {
    let token = iter.peek()?;

    match token {
        Token::Literal(lit) => {
            iter.next();
            Some(AstNode::Literal(lit.clone()))
        }
        Token::Ident(ident) => {
            iter.next();
            trace!("Parsing identifier: {}", ident);
            if let Some(fcall) = parse_function_call(ident, iter) {
                Some(fcall)
            } else {
                Some(AstNode::Variable(ident.clone()))
            }
        }
        Token::LParen => {
            iter.next();
            let expr = parse_expression(iter).unwrap();
            let next = iter.peek().unwrap();
            if next != &&Token::RParen {
                panic!("Expected closing parenthesis, found: {:?}", next);
            }
            iter.next();
            Some(expr)
        }
        Token::LSquareParen => {
            iter.next();
            let elements = parse_comma_separated_list(iter, Token::RSquareParen);
            Some(AstNode::List(elements))
        }
        _ => None,
    }
}
//

fn parse_expression<'a, I: Iterator<Item = &'a Token> + Clone>(
    iter: &mut Peekable<I>,
) -> Option<AstNode> {
    let left = parse_primary_expression(iter)?;
    trace!("Parsed primary expression: {:?}", left);
    parse_expression_impl(iter, left, 0)
}

fn parse_expression_impl<'a, I: Iterator<Item = &'a Token> + Clone>(
    iter: &mut Peekable<I>,
    mut left: AstNode,
    min_precedence: u8,
) -> Option<AstNode> {
    while let Some(Token::Operator(op)) = iter.peek() {
        if op.precedence() < min_precedence {
            break;
        }
        let op = *op;
        iter.next();
        let mut right =
            parse_primary_expression(iter).expect("Expected an expression after operator");

        trace!(
            "Parsed right-hand side expression: {:?} after op, {:?}",
            right, op
        );

        while let Some(Token::Operator(next_op)) = iter.peek() {
            if next_op.precedence() > op.precedence() {
                right = parse_expression_impl(iter, right, next_op.precedence())
                    .expect("Expected expression after operator");
                trace!(
                    "Updated right-hand side expression to: {:?} after parsing higher precedence op {:?}",
                    right, next_op
                );
            } else {
                break;
            }
        }

        left = AstNode::BinaryOp(Box::new(left), op, Box::new(right));
    }

    Some(left)
}

pub fn parse(tokens: &[Token]) -> Vec<AstNode> {
    let mut iter = tokens.iter().peekable();

    let mut block = Vec::new();

    while let Some(token) = iter.next() {
        match token {
            Token::Ident(ident) => match iter.peek().copied().unwrap() {
                Token::Assign => {
                    trace!("Parsing assignment to {}", ident);
                    iter.next();
                    let expr = parse_expression(&mut iter).unwrap();
                    block.push(AstNode::Assign(ident.clone(), Box::new(expr)));
                }
                t => {
                    trace!("Parsing function call starting with identifier {}", ident);
                    if let Some(f) = parse_function_call(ident, &mut iter) {
                        block.push(f);
                    } else {
                        panic!("Token not allowed after identifier: {:?}", t);
                    }
                }
            },
            Token::Eol => {}
            t => {
                println!("{:#?}", block);
                panic!("Token not allowed: {:?}", t);
            }
        }
    }

    block
}
