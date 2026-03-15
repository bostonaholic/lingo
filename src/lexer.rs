/// Tokenizer for the Lingo programming language.

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Fn,
    Let,
    Mut,
    If,
    Else,
    Match,
    For,
    In,
    While,
    Loop,
    Break,
    Return,
    True,
    False,
    Struct,
    Enum,
    Type,
    Pub,
    Use,
    Mod,
    Trait,
    Impl,
    Async,
    Await,

    // Literals
    IntLit(i64),
    FloatLit(f64),
    StrLit(String),
    // String interpolation yields multiple tokens:
    StringInterpStart(String), // text before first {
    StringInterpMid(String),   // text between } and next {
    StringInterpEnd(String),   // text after last }

    // Identifiers
    Ident(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqEq,
    BangEq,
    Lt,
    Gt,
    Le,
    Ge,
    AndAnd,
    PipePipe,
    Bang,
    Eq,
    PlusEq,
    MinusEq,
    PipeGt,    // |>
    FatArrow,  // =>
    ThinArrow, // ->
    Dot,
    ColonColon,
    DotDot,    // ..
    DotDotEq,  // ..=
    Question,
    Colon,
    Pipe,
    PlusPlus,  // ++

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Underscore,

    // Special
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        // Insert newlines as statement terminators (Go-style semicolon insertion)
        tokens = self.insert_newlines(tokens);
        Ok(tokens)
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek_ahead(&self, offset: usize) -> Option<char> {
        self.source.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.source.get(self.pos).copied();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        ch
    }

    fn skip_whitespace_not_newline(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace_not_newline();

        let line = self.line;
        let col = self.col;

        let ch = match self.peek() {
            Some(c) => c,
            None => {
                return Ok(Token {
                    kind: TokenKind::Eof,
                    line,
                    col,
                });
            }
        };

        // Comments
        if ch == '#' {
            while let Some(c) = self.peek() {
                if c == '\n' {
                    break;
                }
                self.advance();
            }
            return self.next_token();
        }

        // Newlines
        if ch == '\n' {
            self.advance();
            return Ok(Token {
                kind: TokenKind::Newline,
                line,
                col,
            });
        }

        // Semicolons act as newlines
        if ch == ';' {
            self.advance();
            return Ok(Token {
                kind: TokenKind::Newline,
                line,
                col,
            });
        }

        // Numbers
        if ch.is_ascii_digit() {
            return self.lex_number(line, col);
        }

        // Strings
        if ch == '"' {
            return self.lex_string(line, col);
        }

        // Identifiers and keywords
        if ch.is_alphabetic() || ch == '_' {
            return self.lex_ident(line, col);
        }

        // Multi-char operators first
        if ch == '=' && self.peek_ahead(1) == Some('=') {
            self.advance();
            self.advance();
            return Ok(Token { kind: TokenKind::EqEq, line, col });
        }
        if ch == '!' && self.peek_ahead(1) == Some('=') {
            self.advance();
            self.advance();
            return Ok(Token { kind: TokenKind::BangEq, line, col });
        }
        if ch == '<' && self.peek_ahead(1) == Some('=') {
            self.advance();
            self.advance();
            return Ok(Token { kind: TokenKind::Le, line, col });
        }
        if ch == '>' && self.peek_ahead(1) == Some('=') {
            self.advance();
            self.advance();
            return Ok(Token { kind: TokenKind::Ge, line, col });
        }
        if ch == '&' && self.peek_ahead(1) == Some('&') {
            self.advance();
            self.advance();
            return Ok(Token { kind: TokenKind::AndAnd, line, col });
        }
        if ch == '|' && self.peek_ahead(1) == Some('|') {
            self.advance();
            self.advance();
            return Ok(Token { kind: TokenKind::PipePipe, line, col });
        }
        if ch == '|' && self.peek_ahead(1) == Some('>') {
            self.advance();
            self.advance();
            return Ok(Token { kind: TokenKind::PipeGt, line, col });
        }
        if ch == '=' && self.peek_ahead(1) == Some('>') {
            self.advance();
            self.advance();
            return Ok(Token { kind: TokenKind::FatArrow, line, col });
        }
        if ch == '-' && self.peek_ahead(1) == Some('>') {
            self.advance();
            self.advance();
            return Ok(Token { kind: TokenKind::ThinArrow, line, col });
        }
        if ch == '+' && self.peek_ahead(1) == Some('+') {
            self.advance();
            self.advance();
            return Ok(Token { kind: TokenKind::PlusPlus, line, col });
        }
        if ch == '+' && self.peek_ahead(1) == Some('=') {
            self.advance();
            self.advance();
            return Ok(Token { kind: TokenKind::PlusEq, line, col });
        }
        if ch == '-' && self.peek_ahead(1) == Some('=') {
            self.advance();
            self.advance();
            return Ok(Token { kind: TokenKind::MinusEq, line, col });
        }
        if ch == ':' && self.peek_ahead(1) == Some(':') {
            self.advance();
            self.advance();
            return Ok(Token { kind: TokenKind::ColonColon, line, col });
        }
        if ch == '.' && self.peek_ahead(1) == Some('.') && self.peek_ahead(2) == Some('=') {
            self.advance();
            self.advance();
            self.advance();
            return Ok(Token { kind: TokenKind::DotDotEq, line, col });
        }
        if ch == '.' && self.peek_ahead(1) == Some('.') {
            self.advance();
            self.advance();
            return Ok(Token { kind: TokenKind::DotDot, line, col });
        }

        // Single-char tokens
        self.advance();
        let kind = match ch {
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '<' => TokenKind::Lt,
            '>' => TokenKind::Gt,
            '!' => TokenKind::Bang,
            '=' => TokenKind::Eq,
            '.' => TokenKind::Dot,
            '?' => TokenKind::Question,
            ':' => TokenKind::Colon,
            '|' => TokenKind::Pipe,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            ',' => TokenKind::Comma,
            _ => return Err(format!("Unexpected character '{}' at {}:{}", ch, line, col)),
        };
        Ok(Token { kind, line, col })
    }

    fn lex_number(&mut self, line: usize, col: usize) -> Result<Token, String> {
        let mut num_str = String::new();
        let mut is_float = false;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.advance();
            } else if c == '.' && self.peek_ahead(1) != Some('.') {
                // Check it's not the start of `..` or `..=`
                if let Some(next) = self.peek_ahead(1) {
                    if next.is_ascii_digit() {
                        is_float = true;
                        num_str.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else if c == '_' {
                // Allow _ as separator in numbers
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            let val: f64 = num_str
                .parse()
                .map_err(|e| format!("Invalid float '{}' at {}:{}: {}", num_str, line, col, e))?;
            Ok(Token {
                kind: TokenKind::FloatLit(val),
                line,
                col,
            })
        } else {
            let val: i64 = num_str
                .parse()
                .map_err(|e| format!("Invalid integer '{}' at {}:{}: {}", num_str, line, col, e))?;
            Ok(Token {
                kind: TokenKind::IntLit(val),
                line,
                col,
            })
        }
    }

    fn lex_string(&mut self, line: usize, col: usize) -> Result<Token, String> {
        self.advance(); // consume opening "
        let mut parts: Vec<(bool, String)> = Vec::new(); // (is_expr, content)
        let mut current = String::new();
        let mut has_interp = false;

        loop {
            match self.peek() {
                None => return Err(format!("Unterminated string starting at {}:{}", line, col)),
                Some('"') => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    self.advance();
                    match self.peek() {
                        Some('n') => {
                            self.advance();
                            current.push('\n');
                        }
                        Some('t') => {
                            self.advance();
                            current.push('\t');
                        }
                        Some('\\') => {
                            self.advance();
                            current.push('\\');
                        }
                        Some('"') => {
                            self.advance();
                            current.push('"');
                        }
                        Some('{') => {
                            self.advance();
                            current.push('{');
                        }
                        Some(c) => {
                            self.advance();
                            current.push('\\');
                            current.push(c);
                        }
                        None => {
                            return Err(format!(
                                "Unterminated escape in string at {}:{}",
                                self.line, self.col
                            ))
                        }
                    }
                }
                Some('{') => {
                    has_interp = true;
                    // Save current literal part
                    parts.push((false, std::mem::take(&mut current)));
                    self.advance(); // consume {

                    // Read expression until matching }
                    let mut depth = 1;
                    let mut expr_str = String::new();
                    loop {
                        match self.peek() {
                            None => {
                                return Err(format!(
                                    "Unterminated interpolation in string at {}:{}",
                                    self.line, self.col
                                ))
                            }
                            Some('{') => {
                                depth += 1;
                                expr_str.push('{');
                                self.advance();
                            }
                            Some('}') => {
                                depth -= 1;
                                self.advance();
                                if depth == 0 {
                                    break;
                                }
                                expr_str.push('}');
                            }
                            Some(c) => {
                                expr_str.push(c);
                                self.advance();
                            }
                        }
                    }
                    parts.push((true, expr_str));
                }
                Some(c) => {
                    current.push(c);
                    self.advance();
                }
            }
        }

        if !has_interp {
            // Simple string
            Ok(Token {
                kind: TokenKind::StrLit(current),
                line,
                col,
            })
        } else {
            // Push final literal part
            parts.push((false, current));

            // Convert to interp token sequence
            // We'll emit: StringInterpStart, then for each expr part we need to
            // re-lex the expression. But that's complex. Instead, let's use a
            // single token that carries all parts and parse it later.
            // Actually, let's just produce the start/mid/end tokens with embedded
            // expressions stored as strings that will be parsed by the parser.

            // Simplification: store all parts in StringInterpStart.
            // We'll encode as a special token that the parser understands.

            // Better approach: produce the tokens inline. The parser will handle
            // StringInterpStart (text) [expr tokens] StringInterpMid (text) [expr tokens] ... StringInterpEnd (text)

            // Actually, let me just store the parts directly and have a special handling.
            // For simplicity, we'll encode the full interpolation as a single token
            // with the parts pre-split.
            let mut result_parts = Vec::new();
            for (is_expr, content) in parts {
                if is_expr {
                    result_parts.push(format!("\x01{}", content)); // \x01 marks expr
                } else {
                    result_parts.push(format!("\x02{}", content)); // \x02 marks literal
                }
            }
            let encoded = result_parts.join("");
            Ok(Token {
                kind: TokenKind::StringInterpStart(encoded),
                line,
                col,
            })
        }
    }

    fn lex_ident(&mut self, line: usize, col: usize) -> Result<Token, String> {
        let mut ident = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let kind = match ident.as_str() {
            "fn" => TokenKind::Fn,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "match" => TokenKind::Match,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "while" => TokenKind::While,
            "loop" => TokenKind::Loop,
            "break" => TokenKind::Break,
            "return" => TokenKind::Return,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "struct" => TokenKind::Struct,
            "enum" => TokenKind::Enum,
            "type" => TokenKind::Type,
            "pub" => TokenKind::Pub,
            "use" => TokenKind::Use,
            "mod" => TokenKind::Mod,
            "trait" => TokenKind::Trait,
            "impl" => TokenKind::Impl,
            "async" => TokenKind::Async,
            "await" => TokenKind::Await,
            "_" => TokenKind::Underscore,
            _ => TokenKind::Ident(ident),
        };

        Ok(Token { kind, line, col })
    }

    /// Go-style newline insertion: suppress newline after continuation tokens.
    fn insert_newlines(&self, tokens: Vec<Token>) -> Vec<Token> {
        let mut result = Vec::new();

        for tok in tokens {
            if tok.kind == TokenKind::Newline {
                // Check if we should suppress this newline
                if let Some(prev) = result.last() {
                    let prev: &Token = prev;
                    if self.is_continuation_token(&prev.kind) {
                        // Skip this newline
                        continue;
                    }
                }
                // Also suppress consecutive newlines
                if let Some(prev) = result.last() {
                    let prev: &Token = prev;
                    if prev.kind == TokenKind::Newline {
                        continue;
                    }
                }
                // Don't emit newline at the very start
                if result.is_empty() {
                    continue;
                }
            }
            result.push(tok);
        }

        result
    }

    fn is_continuation_token(&self, kind: &TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::PipeGt
                | TokenKind::FatArrow
                | TokenKind::ThinArrow
                | TokenKind::Eq
                | TokenKind::PlusEq
                | TokenKind::MinusEq
                | TokenKind::LParen
                | TokenKind::LBracket
                | TokenKind::LBrace
                | TokenKind::Comma
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Percent
                | TokenKind::EqEq
                | TokenKind::BangEq
                | TokenKind::Lt
                | TokenKind::Gt
                | TokenKind::Le
                | TokenKind::Ge
                | TokenKind::AndAnd
                | TokenKind::PipePipe
                | TokenKind::PlusPlus
                | TokenKind::Pipe
                | TokenKind::Newline
        )
    }
}
