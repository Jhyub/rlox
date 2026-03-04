
pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: i32,
}

impl <'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, start: 0, current: 0, line: 1 }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();

        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();

        if c.is_alphabetic() {
            return self.identifier();
        }

        if c.is_digit(10) {
            return self.number();
        }

        let token = match c {
            '(' => Some(self.make_token(TokenType::LeftParen)),
            ')' => Some(self.make_token(TokenType::RightParen)),
            '{' => Some(self.make_token(TokenType::LeftBrace)),
            '}' => Some(self.make_token(TokenType::RightBrace)),
            ',' => Some(self.make_token(TokenType::Comma)),
            '.' => Some(self.make_token(TokenType::Dot)),
            '-' => Some(self.make_token(TokenType::Minus)),
            '+' => Some(self.make_token(TokenType::Plus)),
            ';' => Some(self.make_token(TokenType::Semicolon)),
            '/' => Some(self.make_token(TokenType::Slash)),
            '*' => Some(self.make_token(TokenType::Star)),
            '!' => {
                if self.match_next('=') {
                    Some(self.make_token(TokenType::BangEqual))
                } else {
                    Some(self.make_token(TokenType::Bang))
                }
            }
            '=' => {
                if self.match_next('=') {
                    Some(self.make_token(TokenType::EqualEqual))
                } else {
                    Some(self.make_token(TokenType::Equal))
                }
            }
            '>' => {
                if self.match_next('=') {
                    Some(self.make_token(TokenType::GreaterEqual))
                } else {
                    Some(self.make_token(TokenType::Greater))
                }
            }
            '<' => {
                if self.match_next('=') {
                    Some(self.make_token(TokenType::LessEqual))
                } else {
                    Some(self.make_token(TokenType::Less))
                }
            }
            '"' => Some(self.string()),
            _ => None,
        };

        token.unwrap_or_else(|| self.make_error_token("Unexpected character."))
    }

    fn make_token(&self, ttype: TokenType) -> Token {
        Token {
            ttype,
            lexeme: self.source[self.start..self.current].to_string(),
            line: self.line,
        }
    }

    fn make_error_token(&self, message: &str) -> Token {
        Token {
            ttype: TokenType::Error,
            lexeme: message.to_string(),
            line: self.line,
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
                        loop {
                            self.advance();
                            if self.peek() == '\n' || self.is_at_end() {
                                continue;
                            }
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn string(&mut self) -> Token {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return self.make_error_token("Unterminated string.");
        }

        self.advance();
        return self.make_token(TokenType::String);
    }
    
    fn number(&mut self) -> Token {
        while self.peek().is_digit(10) {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_digit(10) {
            self.advance();
            while self.peek().is_digit(10) {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn identifier(&mut self) -> Token {
        while self.peek().is_alphanumeric() {
            self.advance();
        }

        let ttype = self.identifier_type();
        return self.make_token(ttype);
    }

    fn identifier_type(&mut self) -> TokenType {
        match self.source[self.start..self.start+1].chars().next().unwrap() {
            'a' => self.check_keyword(1, 2, "nd", TokenType::And),
            'c' => self.check_keyword(1, 3, "lass", TokenType::Class),
            'e' => self.check_keyword(1, 2, "lse", TokenType::Else),
            'f' => {
                if self.current - self.start > 1 {
                    match self.source[self.start+1..self.start+2].chars().next().unwrap() {
                        'a' => self.check_keyword(2, 3, "lse", TokenType::False),
                        'o' => self.check_keyword(2, 1, "r", TokenType::For),
                        'u' => self.check_keyword(2, 3, "n", TokenType::Fun),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'i' => self.check_keyword(1, 1, "f", TokenType::If),
            'n' => self.check_keyword(1, 2, "il", TokenType::Nil),
            'o' => self.check_keyword(1, 1, "r", TokenType::Or),
            'p' => self.check_keyword(1, 4, "rint", TokenType::Print),
            'r' => self.check_keyword(1, 5, "eturn", TokenType::Return),
            's' => self.check_keyword(1, 4, "uper", TokenType::Super),
            't' => {
                if self.current - self.start > 1 {
                    match self.source[self.start+1..self.start+2].chars().next().unwrap() {
                        'h' => self.check_keyword(2, 1, "is", TokenType::This),
                        'r' => self.check_keyword(2, 1, "ue", TokenType::True),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                    }
            }
            'v' => self.check_keyword(1, 2, "ar", TokenType::Var),
            'w' => self.check_keyword(1, 4, "hile", TokenType::While),
            _ => TokenType::Identifier,
        }
    }

    fn check_keyword(&self, start: usize, length: usize, rest: &str, ttype: TokenType) -> TokenType {
        if (self.current - self.start) == (start + length) && self.source[self.start..self.start + length].to_string() == rest {
            return ttype;
        }

        TokenType::Identifier
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    pub fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current - 1..self.current].chars().next().unwrap()
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current..self.current + 1].chars().next().unwrap()
        }
    }

    fn peek_next(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current + 1..self.current + 2].chars().next().unwrap()
        }
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            false
        } else {
            if self.source[self.current..self.current + 1].chars().next().unwrap() == expected {
                self.current += 1;
                true
            } else {
                false
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    ttype: TokenType,
    lexeme: String,
    line: i32,
}

impl Token {
    pub fn new(ttype: TokenType, lexeme: String, line: i32) -> Self {
        Self { ttype, lexeme, line }
    }

    pub fn ttype(&self) -> TokenType {
        self.ttype
    }

    pub fn lexeme(&self) -> &str {
        &self.lexeme
    }

    pub fn line(&self) -> i32 {
        self.line
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Identifier,
    String,
    Number,

    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Error,
    Eof,
}