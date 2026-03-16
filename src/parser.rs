//! S-expression reader for the Lingo programming language (Lisp dialect).
//!
//! Parses a flat token stream into a `Vec<Expr>` of top-level S-expressions.

use crate::ast::Expr;
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Expr>, String> {
        let mut exprs = Vec::new();
        while !self.is_at_end() {
            exprs.push(self.read_expr()?);
        }
        Ok(exprs)
    }

    fn read_expr(&mut self) -> Result<Expr, String> {
        let token = self.peek().cloned();
        match token {
            Some(Token::LParen) => {
                self.advance();
                let elems = self.read_list()?;
                Ok(Expr::List(elems))
            }
            Some(Token::Quote) => {
                self.advance();
                let expr = self.read_expr()?;
                Ok(Expr::List(vec![Expr::Symbol("quote".to_string()), expr]))
            }
            Some(Token::Atom(ref s)) => {
                let s = s.clone();
                self.advance();
                Ok(self.parse_atom(&s))
            }
            Some(Token::Str(ref s)) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Str(s))
            }
            Some(Token::RParen) => Err("Unexpected ')'".to_string()),
            Some(Token::Eof) | None => Err("Unexpected end of input".to_string()),
        }
    }

    fn read_list(&mut self) -> Result<Vec<Expr>, String> {
        let mut elems = Vec::new();
        loop {
            match self.peek() {
                Some(Token::RParen) => {
                    self.advance();
                    return Ok(elems);
                }
                Some(Token::Eof) | None => {
                    return Err("Unexpected end of input, expected ')'".to_string());
                }
                _ => {
                    elems.push(self.read_expr()?);
                }
            }
        }
    }

    fn parse_atom(&self, s: &str) -> Expr {
        // Try integer
        if let Ok(n) = s.parse::<i64>() {
            return Expr::Int(n);
        }
        // Try float
        if let Ok(f) = s.parse::<f64>() {
            return Expr::Float(f);
        }
        // Booleans
        if s == "true" {
            return Expr::Bool(true);
        }
        if s == "false" {
            return Expr::Bool(false);
        }
        // Nil
        if s == "nil" {
            return Expr::Nil;
        }
        // Everything else is a symbol
        Expr::Symbol(s.to_string())
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), None | Some(Token::Eof))
    }
}
