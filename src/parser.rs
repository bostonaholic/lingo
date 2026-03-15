/// Recursive descent parser for the Lingo programming language.

use crate::ast::*;
use crate::lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut items = Vec::new();
        self.skip_newlines();
        while !self.is_at_end() {
            let item = self.parse_item()?;
            items.push(item);
            self.skip_newlines();
        }
        Ok(Program { items })
    }

    fn parse_item(&mut self) -> Result<Item, String> {
        if self.check(&TokenKind::Fn) {
            Ok(Item::FnDecl(self.parse_fn_decl()?))
        } else if self.check(&TokenKind::Let) {
            let stmt = self.parse_let_stmt()?;
            self.expect_terminator()?;
            Ok(Item::Stmt(stmt))
        } else {
            let expr = self.parse_expr()?;
            self.expect_terminator()?;
            Ok(Item::ExprStmt(expr))
        }
    }

    fn parse_fn_decl(&mut self) -> Result<FnDecl, String> {
        let span = self.current_span();
        self.expect(&TokenKind::Fn)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::RParen)?;

        let return_type = if self.check(&TokenKind::ThinArrow) {
            self.advance();
            Some(self.expect_ident()?)
        } else {
            None
        };

        let body = self.parse_block()?;
        Ok(FnDecl {
            name,
            params,
            return_type,
            body,
            span,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, String> {
        let mut params = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.is_at_end() {
            let name = self.expect_ident()?;
            let type_ann = if self.check(&TokenKind::Colon) {
                self.advance();
                Some(self.expect_ident()?)
            } else {
                None
            };
            params.push(Param { name, type_ann });
            if !self.check(&TokenKind::RParen) {
                self.expect(&TokenKind::Comma)?;
                self.skip_newlines();
            }
        }
        Ok(params)
    }

    fn parse_block(&mut self) -> Result<Block, String> {
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();

        let mut stmts = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let stmt = self.parse_stmt()?;
            self.skip_newlines();
            stmts.push(stmt);
        }

        self.expect(&TokenKind::RBrace)?;

        // Check if the last statement is an expression that should be the block's value
        // If the last stmt is an ExprStmt and the block is used in expression position,
        // we want to try to return it as the trailing expression.
        let expr = if let Some(Stmt::Expr(_)) = stmts.last() {
            if let Some(Stmt::Expr(e)) = stmts.pop() {
                Some(Box::new(e))
            } else {
                None
            }
        } else {
            None
        };

        Ok(Block { stmts, expr })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        if self.check(&TokenKind::Let) {
            self.parse_let_stmt()
        } else if self.check(&TokenKind::For) {
            self.parse_for_stmt()
        } else if self.check(&TokenKind::While) {
            self.parse_while_stmt()
        } else if self.check(&TokenKind::Return) {
            self.parse_return_stmt()
        } else if self.check(&TokenKind::Break) {
            self.advance();
            Ok(Stmt::Break)
        } else {
            let expr = self.parse_expr()?;
            // Check for assignment
            if self.check(&TokenKind::Eq) {
                self.advance();
                self.skip_newlines();
                let value = self.parse_expr()?;
                let span = self.current_span();
                return Ok(Stmt::Expr(Expr::Assign(
                    Box::new(expr),
                    Box::new(value),
                    span,
                )));
            }
            if self.check(&TokenKind::PlusEq) || self.check(&TokenKind::MinusEq) {
                let op = if self.check(&TokenKind::PlusEq) {
                    BinOp::Add
                } else {
                    BinOp::Sub
                };
                self.advance();
                self.skip_newlines();
                let value = self.parse_expr()?;
                let span = self.current_span();
                return Ok(Stmt::Expr(Expr::CompoundAssign(
                    Box::new(expr),
                    op,
                    Box::new(value),
                    span,
                )));
            }
            Ok(Stmt::Expr(expr))
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt, String> {
        let span = self.current_span();
        self.expect(&TokenKind::Let)?;

        let mutable = if self.check(&TokenKind::Mut) {
            self.advance();
            true
        } else {
            false
        };

        let pattern = self.parse_pattern()?;

        let type_ann = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(self.expect_ident()?)
        } else {
            None
        };

        self.expect(&TokenKind::Eq)?;
        self.skip_newlines();
        let value = self.parse_expr()?;

        Ok(Stmt::Let(LetStmt {
            pattern,
            mutable,
            type_ann,
            value,
            span,
        }))
    }

    fn parse_for_stmt(&mut self) -> Result<Stmt, String> {
        let span = self.current_span();
        self.expect(&TokenKind::For)?;
        let binding = self.parse_pattern()?;
        self.expect(&TokenKind::In)?;
        let iterable = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::For(ForStmt {
            binding,
            iterable,
            body,
            span,
        }))
    }

    fn parse_while_stmt(&mut self) -> Result<Stmt, String> {
        let span = self.current_span();
        self.expect(&TokenKind::While)?;
        let condition = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::While(WhileStmt {
            condition,
            body,
            span,
        }))
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(&TokenKind::Return)?;
        if self.check(&TokenKind::Newline)
            || self.check(&TokenKind::RBrace)
            || self.is_at_end()
        {
            Ok(Stmt::Return(None))
        } else {
            let expr = self.parse_expr()?;
            Ok(Stmt::Return(Some(expr)))
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_pipeline()
    }

    fn parse_pipeline(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_or()?;
        while self.check(&TokenKind::PipeGt) {
            let span = self.current_span();
            self.advance();
            self.skip_newlines();
            let right = self.parse_or()?;
            left = Expr::Pipeline(Box::new(left), Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while self.check(&TokenKind::PipePipe) {
            let span = self.current_span();
            self.advance();
            self.skip_newlines();
            let right = self.parse_and()?;
            left = Expr::Binary(Box::new(left), BinOp::Or, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_equality()?;
        while self.check(&TokenKind::AndAnd) {
            let span = self.current_span();
            self.advance();
            self.skip_newlines();
            let right = self.parse_equality()?;
            left = Expr::Binary(Box::new(left), BinOp::And, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        while self.check(&TokenKind::EqEq) || self.check(&TokenKind::BangEq) {
            let span = self.current_span();
            let op = if self.check(&TokenKind::EqEq) {
                BinOp::Eq
            } else {
                BinOp::Ne
            };
            self.advance();
            self.skip_newlines();
            let right = self.parse_comparison()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_concat()?;
        while self.check(&TokenKind::Lt)
            || self.check(&TokenKind::Gt)
            || self.check(&TokenKind::Le)
            || self.check(&TokenKind::Ge)
        {
            let span = self.current_span();
            let op = if self.check(&TokenKind::Lt) {
                BinOp::Lt
            } else if self.check(&TokenKind::Gt) {
                BinOp::Gt
            } else if self.check(&TokenKind::Le) {
                BinOp::Le
            } else {
                BinOp::Ge
            };
            self.advance();
            self.skip_newlines();
            let right = self.parse_concat()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_concat(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_range()?;
        while self.check(&TokenKind::PlusPlus) {
            let span = self.current_span();
            self.advance();
            self.skip_newlines();
            let right = self.parse_range()?;
            left = Expr::Binary(Box::new(left), BinOp::Concat, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_range(&mut self) -> Result<Expr, String> {
        let left = self.parse_addition()?;
        if self.check(&TokenKind::DotDotEq) {
            let span = self.current_span();
            self.advance();
            let right = self.parse_addition()?;
            return Ok(Expr::Range(Box::new(left), Box::new(right), true, span));
        }
        if self.check(&TokenKind::DotDot) {
            let span = self.current_span();
            self.advance();
            let right = self.parse_addition()?;
            return Ok(Expr::Range(Box::new(left), Box::new(right), false, span));
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplication()?;
        while self.check(&TokenKind::Plus) || self.check(&TokenKind::Minus) {
            let span = self.current_span();
            let op = if self.check(&TokenKind::Plus) {
                BinOp::Add
            } else {
                BinOp::Sub
            };
            self.advance();
            self.skip_newlines();
            let right = self.parse_multiplication()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        while self.check(&TokenKind::Star)
            || self.check(&TokenKind::Slash)
            || self.check(&TokenKind::Percent)
        {
            let span = self.current_span();
            let op = if self.check(&TokenKind::Star) {
                BinOp::Mul
            } else if self.check(&TokenKind::Slash) {
                BinOp::Div
            } else {
                BinOp::Mod
            };
            self.advance();
            self.skip_newlines();
            let right = self.parse_unary()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.check(&TokenKind::Minus) {
            let span = self.current_span();
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expr::Unary(UnaryOp::Neg, Box::new(operand), span));
        }
        if self.check(&TokenKind::Bang) {
            let span = self.current_span();
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expr::Unary(UnaryOp::Not, Box::new(operand), span));
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.check(&TokenKind::LParen) {
                let span = self.current_span();
                self.advance();
                self.skip_newlines();
                let args = self.parse_call_args()?;
                self.expect(&TokenKind::RParen)?;
                expr = Expr::Call(Box::new(expr), args, span);
            } else if self.check(&TokenKind::LBracket) {
                let span = self.current_span();
                self.advance();
                self.skip_newlines();
                let index = self.parse_expr()?;
                self.expect(&TokenKind::RBracket)?;
                expr = Expr::Index(Box::new(expr), Box::new(index), span);
            } else if self.check(&TokenKind::Dot) {
                let span = self.current_span();
                self.advance();
                let field = self.expect_ident()?;
                expr = Expr::Field(Box::new(expr), field, span);
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_call_args(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.is_at_end() {
            self.skip_newlines();
            // Check for lambda: param => body or (params) => body
            let arg = self.parse_lambda_or_expr()?;
            args.push(arg);
            if self.check(&TokenKind::Comma) {
                self.advance();
                self.skip_newlines();
            } else {
                break;
            }
        }
        self.skip_newlines();
        Ok(args)
    }

    fn parse_lambda_or_expr(&mut self) -> Result<Expr, String> {
        // Try to parse: ident => body (single param lambda)
        // or (params) => body
        let saved_pos = self.pos;

        // Check single ident => expr
        if let Some(TokenKind::Ident(_)) = self.peek_kind() {
            let name = self.expect_ident()?;
            if self.check(&TokenKind::FatArrow) {
                self.advance();
                self.skip_newlines();
                let body = self.parse_expr()?;
                let span = self.current_span();
                return Ok(Expr::Lambda(
                    vec![Param {
                        name,
                        type_ann: None,
                    }],
                    Box::new(body),
                    span,
                ));
            }
            // Not a lambda, backtrack
            self.pos = saved_pos;
        }

        // Check (params) => expr
        if self.check(&TokenKind::LParen) {
            let saved_pos2 = self.pos;
            self.advance();
            // Try to parse as lambda params
            if let Ok(params) = self.try_parse_lambda_params() {
                if self.check(&TokenKind::RParen) {
                    self.advance();
                    if self.check(&TokenKind::FatArrow) {
                        self.advance();
                        self.skip_newlines();
                        let body = self.parse_expr()?;
                        let span = self.current_span();
                        return Ok(Expr::Lambda(params, Box::new(body), span));
                    }
                }
            }
            self.pos = saved_pos2;
        }

        self.parse_expr()
    }

    fn try_parse_lambda_params(&mut self) -> Result<Vec<Param>, String> {
        let mut params = Vec::new();
        self.skip_newlines();
        while !self.check(&TokenKind::RParen) && !self.is_at_end() {
            let name = self.expect_ident()?;
            let type_ann = if self.check(&TokenKind::Colon) {
                self.advance();
                Some(self.expect_ident()?)
            } else {
                None
            };
            params.push(Param { name, type_ann });
            if self.check(&TokenKind::Comma) {
                self.advance();
                self.skip_newlines();
            } else {
                break;
            }
        }
        Ok(params)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let span = self.current_span();
        let kind = self.peek_kind().cloned();

        match kind {
            Some(TokenKind::IntLit(n)) => {
                self.advance();
                Ok(Expr::Literal(Literal::Int(n)))
            }
            Some(TokenKind::FloatLit(f)) => {
                self.advance();
                Ok(Expr::Literal(Literal::Float(f)))
            }
            Some(TokenKind::StrLit(s)) => {
                self.advance();
                Ok(Expr::Literal(Literal::Str(s)))
            }
            Some(TokenKind::StringInterpStart(encoded)) => {
                self.advance();
                self.parse_string_interp(&encoded, span)
            }
            Some(TokenKind::True) => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true)))
            }
            Some(TokenKind::False) => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false)))
            }
            Some(TokenKind::Ident(name)) => {
                self.advance();
                Ok(Expr::Ident(name, span))
            }
            Some(TokenKind::LParen) => {
                self.advance();
                self.skip_newlines();
                if self.check(&TokenKind::RParen) {
                    // Unit tuple ()
                    self.advance();
                    return Ok(Expr::Tuple(Vec::new(), span));
                }
                let first = self.parse_expr()?;
                if self.check(&TokenKind::Comma) {
                    // Tuple
                    self.advance();
                    self.skip_newlines();
                    let mut exprs = vec![first];
                    while !self.check(&TokenKind::RParen) && !self.is_at_end() {
                        let e = self.parse_expr()?;
                        exprs.push(e);
                        if self.check(&TokenKind::Comma) {
                            self.advance();
                            self.skip_newlines();
                        } else {
                            break;
                        }
                    }
                    self.expect(&TokenKind::RParen)?;
                    Ok(Expr::Tuple(exprs, span))
                } else {
                    // Parenthesized expression
                    self.skip_newlines();
                    self.expect(&TokenKind::RParen)?;
                    Ok(first)
                }
            }
            Some(TokenKind::LBracket) => {
                self.advance();
                self.skip_newlines();
                let mut exprs = Vec::new();
                while !self.check(&TokenKind::RBracket) && !self.is_at_end() {
                    let e = self.parse_expr()?;
                    exprs.push(e);
                    if self.check(&TokenKind::Comma) {
                        self.advance();
                        self.skip_newlines();
                    } else {
                        break;
                    }
                }
                self.skip_newlines();
                self.expect(&TokenKind::RBracket)?;
                Ok(Expr::List(exprs, span))
            }
            Some(TokenKind::If) => self.parse_if_expr(),
            Some(TokenKind::Match) => self.parse_match_expr(),
            Some(TokenKind::LBrace) => {
                let block = self.parse_block()?;
                Ok(Expr::Block(block, span))
            }
            Some(TokenKind::Underscore) => {
                self.advance();
                Ok(Expr::Ident("_".to_string(), span))
            }
            _ => {
                let tok = self.peek().cloned();
                Err(format!(
                    "Unexpected token {:?} at {}:{}",
                    tok.as_ref().map(|t| &t.kind),
                    tok.as_ref().map(|t| t.line).unwrap_or(0),
                    tok.as_ref().map(|t| t.col).unwrap_or(0),
                ))
            }
        }
    }

    fn parse_string_interp(&mut self, encoded: &str, span: Span) -> Result<Expr, String> {
        let mut parts = Vec::new();
        let mut i = 0;
        let chars: Vec<char> = encoded.chars().collect();

        while i < chars.len() {
            if chars[i] == '\x02' {
                // Literal part
                i += 1;
                let mut lit = String::new();
                while i < chars.len() && chars[i] != '\x01' && chars[i] != '\x02' {
                    lit.push(chars[i]);
                    i += 1;
                }
                if !lit.is_empty() {
                    parts.push(StringPart::Lit(lit));
                }
            } else if chars[i] == '\x01' {
                // Expression part
                i += 1;
                let mut expr_str = String::new();
                while i < chars.len() && chars[i] != '\x01' && chars[i] != '\x02' {
                    expr_str.push(chars[i]);
                    i += 1;
                }
                // Parse the expression string
                let mut lexer = crate::lexer::Lexer::new(&expr_str);
                let tokens = lexer.tokenize().map_err(|e| {
                    format!("Error lexing interpolated expression: {}", e)
                })?;
                let mut parser = Parser::new(tokens);
                let expr = parser.parse_expr().map_err(|e| {
                    format!("Error parsing interpolated expression: {}", e)
                })?;
                parts.push(StringPart::Expr(expr));
            } else {
                i += 1;
            }
        }

        Ok(Expr::StringInterp(parts, span))
    }

    fn parse_if_expr(&mut self) -> Result<Expr, String> {
        let span = self.current_span();
        self.expect(&TokenKind::If)?;
        let condition = self.parse_expr()?;
        let then_block = self.parse_block()?;
        let else_branch = if self.check(&TokenKind::Else) {
            self.advance();
            if self.check(&TokenKind::If) {
                Some(Box::new(self.parse_if_expr()?))
            } else {
                let block = self.parse_block()?;
                let blk_span = self.current_span();
                Some(Box::new(Expr::Block(block, blk_span)))
            }
        } else {
            None
        };
        Ok(Expr::If(
            Box::new(condition),
            then_block,
            else_branch,
            span,
        ))
    }

    fn parse_match_expr(&mut self) -> Result<Expr, String> {
        let span = self.current_span();
        self.expect(&TokenKind::Match)?;
        let scrutinee = self.parse_expr()?;
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();

        let mut arms = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;
            self.expect(&TokenKind::FatArrow)?;
            self.skip_newlines();
            let body = self.parse_expr()?;
            arms.push(MatchArm {
                pattern,
                guard: None,
                body,
            });
            self.skip_newlines();
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(Expr::Match(Box::new(scrutinee), arms, span))
    }

    fn parse_pattern(&mut self) -> Result<Pattern, String> {
        let pat = self.parse_single_pattern()?;

        // Check for or pattern
        if self.check(&TokenKind::Pipe) {
            let mut patterns = vec![pat];
            while self.check(&TokenKind::Pipe) {
                self.advance();
                patterns.push(self.parse_single_pattern()?);
            }
            return Ok(Pattern::Or(patterns));
        }

        Ok(pat)
    }

    fn parse_single_pattern(&mut self) -> Result<Pattern, String> {
        match self.peek_kind().cloned() {
            Some(TokenKind::Underscore) => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            Some(TokenKind::IntLit(n)) => {
                self.advance();
                Ok(Pattern::Literal(Literal::Int(n)))
            }
            Some(TokenKind::Minus) => {
                // Negative integer literal pattern
                self.advance();
                if let Some(TokenKind::IntLit(n)) = self.peek_kind().cloned() {
                    self.advance();
                    Ok(Pattern::Literal(Literal::Int(-n)))
                } else {
                    Err(format!("Expected integer after - in pattern"))
                }
            }
            Some(TokenKind::FloatLit(f)) => {
                self.advance();
                Ok(Pattern::Literal(Literal::Float(f)))
            }
            Some(TokenKind::StrLit(s)) => {
                self.advance();
                Ok(Pattern::Literal(Literal::Str(s)))
            }
            Some(TokenKind::True) => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(true)))
            }
            Some(TokenKind::False) => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(false)))
            }
            Some(TokenKind::LParen) => {
                self.advance();
                self.skip_newlines();
                let mut pats = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.is_at_end() {
                    pats.push(self.parse_pattern()?);
                    if self.check(&TokenKind::Comma) {
                        self.advance();
                        self.skip_newlines();
                    } else {
                        break;
                    }
                }
                self.expect(&TokenKind::RParen)?;
                Ok(Pattern::Tuple(pats))
            }
            Some(TokenKind::LBracket) => {
                self.advance();
                self.skip_newlines();
                let mut pats = Vec::new();
                while !self.check(&TokenKind::RBracket) && !self.is_at_end() {
                    pats.push(self.parse_pattern()?);
                    if self.check(&TokenKind::Comma) {
                        self.advance();
                        self.skip_newlines();
                    } else {
                        break;
                    }
                }
                self.expect(&TokenKind::RBracket)?;
                Ok(Pattern::List(pats))
            }
            Some(TokenKind::Ident(name)) => {
                self.advance();
                Ok(Pattern::Ident(name))
            }
            other => Err(format!("Unexpected token in pattern: {:?}", other)),
        }
    }

    // -- Utility methods --

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos).map(|t| &t.kind)
    }

    fn check(&self, kind: &TokenKind) -> bool {
        match self.peek_kind() {
            Some(k) => std::mem::discriminant(k) == std::mem::discriminant(kind),
            None => false,
        }
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        self.pos += 1;
        tok
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<Token, String> {
        if self.check(kind) {
            Ok(self.advance().unwrap().clone())
        } else {
            let actual = self.peek().cloned();
            Err(format!(
                "Expected {:?}, got {:?} at {}:{}",
                kind,
                actual.as_ref().map(|t| &t.kind),
                actual.as_ref().map(|t| t.line).unwrap_or(0),
                actual.as_ref().map(|t| t.col).unwrap_or(0),
            ))
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.peek_kind().cloned() {
            Some(TokenKind::Ident(name)) => {
                self.advance();
                Ok(name)
            }
            other => Err(format!(
                "Expected identifier, got {:?} at {}:{}",
                other,
                self.peek().map(|t| t.line).unwrap_or(0),
                self.peek().map(|t| t.col).unwrap_or(0),
            )),
        }
    }

    fn current_span(&self) -> Span {
        if let Some(tok) = self.peek() {
            Span {
                line: tok.line,
                col: tok.col,
            }
        } else {
            Span { line: 0, col: 0 }
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek_kind(), None | Some(TokenKind::Eof))
    }

    fn skip_newlines(&mut self) -> bool {
        let mut found = false;
        while self.check(&TokenKind::Newline) {
            self.advance();
            found = true;
        }
        found
    }

    fn expect_terminator(&mut self) -> Result<(), String> {
        if self.check(&TokenKind::Newline) || self.is_at_end() || self.check(&TokenKind::RBrace) {
            self.skip_newlines();
            Ok(())
        } else {
            Err(format!(
                "Expected newline or end of input, got {:?}",
                self.peek_kind()
            ))
        }
    }
}
