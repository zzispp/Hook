use std::collections::{BTreeMap, BTreeSet};

use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;
use rust_decimal::prelude::ToPrimitive;
use serde_json::Value;

use super::value::value_decimal;

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(Decimal),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Pow,
    LParen,
    RParen,
    Comma,
}

#[derive(Clone, Debug, PartialEq)]
enum ExprNode {
    Number(Decimal),
    Variable(String),
    Unary { op: Token, inner: Box<ExprNode> },
    Binary { op: Token, left: Box<ExprNode>, right: Box<ExprNode> },
    Call { name: String, args: Vec<ExprNode> },
}

pub struct SafeExpressionEvaluator;

impl SafeExpressionEvaluator {
    pub fn eval_decimal(expression: &str, variables: &BTreeMap<String, Value>) -> Result<Decimal, String> {
        let tokens = tokenize(expression)?;
        let mut parser = Parser { tokens, index: 0 };
        let node = parser.parse_expression()?;
        parser.ensure_done()?;
        eval_node(&node, variables)
    }

    pub fn variable_names(expression: &str) -> Result<BTreeSet<String>, String> {
        let tokens = tokenize(expression)?;
        Ok(tokens
            .into_iter()
            .filter_map(|token| match token {
                Token::Ident(name) if !is_allowed_function(&name) => Some(name),
                _ => None,
            })
            .collect())
    }
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    fn parse_expression(&mut self) -> Result<ExprNode, String> {
        self.parse_add_sub()
    }

    fn parse_add_sub(&mut self) -> Result<ExprNode, String> {
        let mut node = self.parse_mul_div()?;
        while self.matches(|token| matches!(token, Token::Plus | Token::Minus)) {
            let op = self.previous().clone();
            let right = self.parse_mul_div()?;
            node = ExprNode::Binary {
                op,
                left: Box::new(node),
                right: Box::new(right),
            };
        }
        Ok(node)
    }

    fn parse_mul_div(&mut self) -> Result<ExprNode, String> {
        let mut node = self.parse_power()?;
        while self.matches(|token| matches!(token, Token::Star | Token::Slash | Token::Percent)) {
            let op = self.previous().clone();
            let right = self.parse_power()?;
            node = ExprNode::Binary {
                op,
                left: Box::new(node),
                right: Box::new(right),
            };
        }
        Ok(node)
    }

    fn parse_power(&mut self) -> Result<ExprNode, String> {
        let mut node = self.parse_unary()?;
        if self.matches(|token| matches!(token, Token::Pow)) {
            let op = self.previous().clone();
            let right = self.parse_power()?;
            node = ExprNode::Binary {
                op,
                left: Box::new(node),
                right: Box::new(right),
            };
        }
        Ok(node)
    }

    fn parse_unary(&mut self) -> Result<ExprNode, String> {
        if self.matches(|token| matches!(token, Token::Plus | Token::Minus)) {
            let op = self.previous().clone();
            let inner = self.parse_unary()?;
            return Ok(ExprNode::Unary { op, inner: Box::new(inner) });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<ExprNode, String> {
        let token = self.advance().ok_or_else(|| "unexpected end of expression".to_owned())?.clone();
        match token {
            Token::Number(value) => Ok(ExprNode::Number(value)),
            Token::Ident(name) => self.parse_identifier(name),
            Token::LParen => {
                let node = self.parse_expression()?;
                self.expect_rparen()?;
                Ok(node)
            }
            _ => Err("unexpected token in expression".into()),
        }
    }

    fn parse_identifier(&mut self, name: String) -> Result<ExprNode, String> {
        if !self.matches(|token| matches!(token, Token::LParen)) {
            return Ok(ExprNode::Variable(name));
        }
        if !is_allowed_function(&name) {
            return Err(format!("function not allowed: {name}"));
        }
        let mut args = Vec::new();
        if self.matches(|token| matches!(token, Token::RParen)) {
            return Ok(ExprNode::Call { name, args });
        }
        loop {
            args.push(self.parse_expression()?);
            if self.matches(|token| matches!(token, Token::RParen)) {
                break;
            }
            self.expect_comma()?;
        }
        Ok(ExprNode::Call { name, args })
    }

    fn ensure_done(&self) -> Result<(), String> {
        (self.index == self.tokens.len())
            .then_some(())
            .ok_or_else(|| "unexpected trailing tokens".into())
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.index)?;
        self.index += 1;
        Some(token)
    }

    fn matches(&mut self, predicate: impl FnOnce(&Token) -> bool) -> bool {
        if self.tokens.get(self.index).is_some_and(predicate) {
            self.index += 1;
            return true;
        }
        false
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.index - 1]
    }

    fn expect_rparen(&mut self) -> Result<(), String> {
        self.matches(|token| matches!(token, Token::RParen))
            .then_some(())
            .ok_or_else(|| "expected ')'".into())
    }

    fn expect_comma(&mut self) -> Result<(), String> {
        self.matches(|token| matches!(token, Token::Comma))
            .then_some(())
            .ok_or_else(|| "expected ','".into())
    }
}

fn tokenize(expression: &str) -> Result<Vec<Token>, String> {
    let chars = expression.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if ch.is_whitespace() {
            index += 1;
            continue;
        }
        match ch {
            '+' => tokens.push(Token::Plus),
            '-' => tokens.push(Token::Minus),
            '*' if chars.get(index + 1) == Some(&'*') => {
                tokens.push(Token::Pow);
                index += 1;
            }
            '*' => tokens.push(Token::Star),
            '/' => tokens.push(Token::Slash),
            '%' => tokens.push(Token::Percent),
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            ',' => tokens.push(Token::Comma),
            '0'..='9' | '.' => index = push_number(&chars, index, &mut tokens)?,
            '_' | 'a'..='z' | 'A'..='Z' => index = push_ident(&chars, index, &mut tokens)?,
            _ => return Err(format!("unsupported character in expression: {ch}")),
        }
        index += 1;
    }
    Ok(tokens)
}

fn push_number(chars: &[char], start: usize, tokens: &mut Vec<Token>) -> Result<usize, String> {
    let mut end = start;
    while matches!(chars.get(end), Some('0'..='9' | '.')) {
        end += 1;
    }
    let text = chars[start..end].iter().collect::<String>();
    let value = text.parse::<Decimal>().map_err(|error| format!("invalid number {text}: {error}"))?;
    tokens.push(Token::Number(value));
    Ok(end - 1)
}

fn push_ident(chars: &[char], start: usize, tokens: &mut Vec<Token>) -> Result<usize, String> {
    let mut end = start;
    while matches!(chars.get(end), Some('_' | 'a'..='z' | 'A'..='Z' | '0'..='9')) {
        end += 1;
    }
    let text = chars[start..end].iter().collect::<String>();
    if text.starts_with("__") {
        return Err("dunder names are not allowed".into());
    }
    tokens.push(Token::Ident(text));
    Ok(end - 1)
}

fn eval_node(node: &ExprNode, variables: &BTreeMap<String, Value>) -> Result<Decimal, String> {
    match node {
        ExprNode::Number(value) => Ok(*value),
        ExprNode::Variable(name) => value_decimal(variables.get(name).ok_or_else(|| format!("missing_variable:{name}"))?),
        ExprNode::Unary { op, inner } => eval_unary(op, eval_node(inner, variables)?),
        ExprNode::Binary { op, left, right } => eval_binary(op, eval_node(left, variables)?, eval_node(right, variables)?),
        ExprNode::Call { name, args } => eval_call(name, args, variables),
    }
}

fn eval_unary(op: &Token, value: Decimal) -> Result<Decimal, String> {
    match op {
        Token::Plus => Ok(value),
        Token::Minus => Ok(-value),
        _ => Err("unsupported unary operator".into()),
    }
}

fn eval_binary(op: &Token, left: Decimal, right: Decimal) -> Result<Decimal, String> {
    match op {
        Token::Plus => Ok(left + right),
        Token::Minus => Ok(left - right),
        Token::Star => Ok(left * right),
        Token::Slash => Ok(left / right),
        Token::Percent => Ok(left % right),
        Token::Pow => decimal_pow(left, right),
        _ => Err("unsupported binary operator".into()),
    }
}

fn eval_call(name: &str, args: &[ExprNode], variables: &BTreeMap<String, Value>) -> Result<Decimal, String> {
    let values = args.iter().map(|arg| eval_node(arg, variables)).collect::<Result<Vec<_>, _>>()?;
    match name {
        "min" => values.into_iter().min().ok_or_else(|| "min requires arguments".into()),
        "max" => values.into_iter().max().ok_or_else(|| "max requires arguments".into()),
        "abs" if values.len() == 1 => Ok(values[0].abs()),
        "round" if values.len() == 1 => Ok(values[0].round()),
        "round" if values.len() == 2 => Ok(values[0].round_dp(values[1].to_u32().unwrap_or(0))),
        "int" if values.len() == 1 => Ok(Decimal::from(values[0].trunc().to_i128().unwrap_or(0))),
        "float" if values.len() == 1 => Ok(values[0]),
        _ => Err(format!("invalid function call: {name}")),
    }
}

fn decimal_pow(left: Decimal, right: Decimal) -> Result<Decimal, String> {
    let exponent = right.to_i64().ok_or_else(|| "pow exponent must be integer".to_owned())?;
    if Decimal::from(exponent) != right {
        return Err("pow exponent must be integer".into());
    }
    Ok(left.powi(exponent))
}

fn is_allowed_function(name: &str) -> bool {
    matches!(name, "min" | "max" | "abs" | "round" | "int" | "float")
}
