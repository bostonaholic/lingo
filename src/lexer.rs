/// S-expression tokenizer for the Lingo programming language (Lisp dialect).

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LParen,
    RParen,
    Atom(String),
    Str(String),
    Quote,
    Eof,
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
            let is_eof = tok == Token::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
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

    fn next_token(&mut self) -> Result<Token, String> {
        // Skip whitespace and commas (commas are optional whitespace)
        while let Some(c) = self.peek() {
            if c.is_whitespace() || c == ',' {
                self.advance();
            } else {
                break;
            }
        }

        let ch = match self.peek() {
            Some(c) => c,
            None => return Ok(Token::Eof),
        };

        // Line comments: ; to end of line
        if ch == ';' {
            while let Some(c) = self.peek() {
                if c == '\n' {
                    break;
                }
                self.advance();
            }
            return self.next_token();
        }

        // Delimiters
        if ch == '(' {
            self.advance();
            return Ok(Token::LParen);
        }
        if ch == ')' {
            self.advance();
            return Ok(Token::RParen);
        }

        // Quote
        if ch == '\'' {
            self.advance();
            return Ok(Token::Quote);
        }

        // String literals
        if ch == '"' {
            return self.lex_string();
        }

        // Atoms: everything else until whitespace, paren, quote, comma, or semicolon
        self.lex_atom()
    }

    fn lex_string(&mut self) -> Result<Token, String> {
        let start_line = self.line;
        let start_col = self.col;
        self.advance(); // consume opening "
        let mut s = String::new();

        loop {
            match self.peek() {
                None => {
                    return Err(format!(
                        "Unterminated string starting at {}:{}",
                        start_line, start_col
                    ))
                }
                Some('"') => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    self.advance();
                    match self.peek() {
                        Some('n') => {
                            self.advance();
                            s.push('\n');
                        }
                        Some('t') => {
                            self.advance();
                            s.push('\t');
                        }
                        Some('\\') => {
                            self.advance();
                            s.push('\\');
                        }
                        Some('"') => {
                            self.advance();
                            s.push('"');
                        }
                        Some(c) => {
                            self.advance();
                            s.push('\\');
                            s.push(c);
                        }
                        None => {
                            return Err(format!(
                                "Unterminated escape in string at {}:{}",
                                self.line, self.col
                            ))
                        }
                    }
                }
                Some(c) => {
                    s.push(c);
                    self.advance();
                }
            }
        }

        Ok(Token::Str(s))
    }

    fn lex_atom(&mut self) -> Result<Token, String> {
        let mut atom = String::new();
        while let Some(c) = self.peek() {
            if c.is_whitespace() || c == '(' || c == ')' || c == '\'' || c == ',' || c == ';' || c == '"' {
                break;
            }
            atom.push(c);
            self.advance();
        }
        if atom.is_empty() {
            Err(format!(
                "Unexpected character at {}:{}",
                self.line, self.col
            ))
        } else {
            Ok(Token::Atom(atom))
        }
    }
}
