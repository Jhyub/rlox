use crate::chunk::{Chunk, OpCode};
use crate::scanner::{Scanner, Token, TokenType};
use crate::value::Value;

pub fn compile(source: &str, chunk: &mut Chunk) -> bool {
    let mut scanner = Scanner::new(source);

    let mut parser = Parser::new(&mut scanner, chunk);

    parser.expression();
    parser.consume(TokenType::Eof, "Expected end of expression.");
    parser.end_compiler();

    !parser.had_error()
}

pub struct Parser<'a> {
    scanner: &'a mut Scanner<'a>,
    compiling_chunk: &'a mut Chunk,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
}

impl<'a> Parser<'a> {
    pub fn new(scanner: &'a mut Scanner<'a>, chunk: &'a mut Chunk) -> Self {
        let current = scanner.scan_token();

        Self {
            scanner,
            compiling_chunk: chunk,
            current,
            previous: Token::new(TokenType::Eof, "".to_string(), 0),
            had_error: false,
            panic_mode: false,
        }
    }
    
    pub fn advance(&mut self) {
        println!("[advance] Current current token: {:?}, previous token: {:?}", self.current, self.previous);
        self.previous = self.current.clone();

        loop {
            self.current = self.scanner.scan_token();
            println!("[advance] Scanned token: {:?}", self.current);
            if self.current.ttype() != TokenType::Error {
                break;
            }
        }
    }

    pub fn consume(&mut self, ttype: TokenType, message: &str) {
        if self.current.ttype() == ttype {
            println!("[consume] advancing from consume");
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn emit_byte(&mut self, byte: u8) {
        let line = self.previous.line();
        self.current_chunk().write(byte, line);
    }

    fn emit_byte_two(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_constant(&mut self, value: Value) {
        let byte = {
            let constant = self.compiling_chunk.add_constant(value);
            if constant > u8::MAX as usize {
                self.error_at(None, "Too many constants in one chunk.", );
                0
            } else {
                constant as u8
            }
        };
        self.emit_byte_two(OpCode::Constant as u8, byte);
    }


    pub fn current_chunk(&mut self) -> &mut Chunk {
        self.compiling_chunk
    }

    pub fn had_error(&self) -> bool {
        self.had_error
    }

    fn end_compiler(&mut self) {
        self.emit_byte(OpCode::Return as u8);
        #[cfg(feature = "debug_print_code")]
        {
            if !self.had_error {
                self.current_chunk().disassemble("code");
            }
        }
    }

    pub fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn binary(&mut self) {
        let operator = self.previous.ttype();
        let rule = ParseRule::get(operator);
        self.parse_precedence(Precedence::from(rule.precedence() as u8 + 1));

        match operator {
            TokenType::Plus => self.emit_byte(OpCode::Add as u8),
            TokenType::Minus => self.emit_byte(OpCode::Subtract as u8),
            TokenType::Star => self.emit_byte(OpCode::Multiply as u8),
            TokenType::Slash => self.emit_byte(OpCode::Divide as u8),
            _ => {}
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expected ')' after expression.");
    }

    fn number(&mut self) {
        let value = self.previous.lexeme().parse::<f64>().unwrap();
        self.emit_constant(value);
    }

    fn unary(&mut self) {
        let operator_type = self.previous.ttype();

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate as u8),
            _ => {}
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        println!("[parse_precedence] Parsing precedence: {:?}", precedence);
        println!("[parse_precedence] Previous token: {:?}", self.previous);

        if let Some(prefix_rule) = ParseRule::get(self.previous.ttype()).prefix() {
            println!("[parse_precedence] Calling prefix rule");
            prefix_rule(self);
        } else {
            self.error_at(None, "Expected expression.");
            return;
        }

        while precedence <= ParseRule::get(self.current.ttype()).precedence() {
            println!("[parse_precedence] advancing inside while loop");
            self.advance();
            if let Some(infix_rule) = ParseRule::get(self.previous.ttype()).infix() {
                println!("[parse_precedence] Calling infix rule");
                infix_rule(self);
            }
        }
    }

    fn error_at_current(&mut self, message: &str) {
        let token = self.current.clone();
        self.error_at(Some(&token), message);
    }

    fn error_at(&mut self, token: Option<&Token>, message: &str) {
        if self.panic_mode { return; }
        self.panic_mode = true;
        let token = token.unwrap_or(&self.previous);
        eprint!("[line {}] Error: {}", token.line(), message);

        if token.ttype() == TokenType::Eof {
            eprint!(" at end");
        } else if token.ttype() == TokenType::Error {
            // Nothing
        } else {
            eprint!(" at '{}'", token.lexeme());
        }

        eprintln!(": {}", message);

        self.had_error = true;
    }
}

pub type ParseFn = fn(&mut Parser) -> ();

pub struct ParseRule {
    prefix: Option<ParseFn>,
    infix: Option<ParseFn>,
    precedence: Precedence,
}

const RULES: [ParseRule; 40] = [
    ParseRule { prefix: Some(|x: &mut Parser| x.grouping()), infix: None, precedence: Precedence::None }, // Token::LeftParen
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::RightParen
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::LeftBrace
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::RightBrace
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Comma
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Dot
    ParseRule { prefix: Some(|x: &mut Parser| x.unary()), infix: Some(|x: &mut Parser| x.binary()), precedence: Precedence::Term }, // Token::Minus
    ParseRule { prefix: None, infix: Some(|x: &mut Parser| x.binary()), precedence: Precedence::Term }, // Token::Plus
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Semicolon
    ParseRule { prefix: None, infix: Some(|x: &mut Parser| x.binary()), precedence: Precedence::Factor }, // Token::Slash
    ParseRule { prefix: None, infix: Some(|x: &mut Parser| x.binary()), precedence: Precedence::Factor }, // Token::Star
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Bang
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::BangEqual
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Equal
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::EqualEqual
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Greater
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::GreaterEqual
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Less
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::LessEqual
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Identifier
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::String
    ParseRule { prefix: Some(|x: &mut Parser| x.number()), infix: None, precedence: Precedence::None }, // Token::Number
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::And
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Class
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Else
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::False
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::For
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Fun
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::If
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Nil
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Or
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Print
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Return
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Super
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::This
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::True
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Var
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::While
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Error
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Eof
];

impl ParseRule {
    pub fn new(prefix: Option<ParseFn>, infix: Option<ParseFn>, precedence: Precedence) -> Self {
        Self { prefix, infix, precedence }
    }

    pub fn prefix(&self) -> Option<ParseFn> {
        self.prefix
    }

    pub fn infix(&self) -> Option<ParseFn> {
        self.infix
    }

    pub fn precedence(&self) -> Precedence {
        self.precedence
    }

    pub fn get(ttype: TokenType) -> &'static ParseRule {
        println!("[ParseRule::get] Getting rule for token type: {:?} with index {}", ttype, ttype as usize);
        return &RULES[ttype as usize];
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Precedence {
    None,
    Assignment, // =
    Or, // or
    And, // and
    Equality, // == !=
    Comparison, // < > <= >=
    Term, // + -
    Factor, // * /
    Unary, // ! -
    Call, // . ()
    Primary, // literals, this, super,
}

impl From<u8> for Precedence {
    fn from(value: u8) -> Self {
        match value {
            0 => Precedence::None,
            1 => Precedence::Assignment,
            2 => Precedence::Or,
            3 => Precedence::And,
            4 => Precedence::Equality,
            5 => Precedence::Comparison,
            6 => Precedence::Term,
            7 => Precedence::Factor,
            8 => Precedence::Unary,
            9 => Precedence::Call,
            10 => Precedence::Primary,
            _ => panic!("Invalid precedence: {}", value),
        }
    }
}