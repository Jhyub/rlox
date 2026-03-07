use crate::chunk::{Chunk, OpCode};
use crate::object::Object;
use crate::scanner::{Scanner, Token, TokenType};
use crate::value::Value;

use std::rc::Rc;

pub fn compile(source: &str, chunk: &mut Chunk) -> bool {
    let mut scanner = Scanner::new(source);

    let mut parser = Parser::new(&mut scanner, chunk);

    while !parser.match_token(TokenType::Eof) {
        parser.declaration();
    }

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
        self.previous = self.current.clone();

        loop {
            self.current = self.scanner.scan_token();
            if self.current.ttype() != TokenType::Error {
                break;
            }
        }
    }

    pub fn consume(&mut self, ttype: TokenType, message: &str) {
        if self.current.ttype() == ttype {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn check_token(&mut self, ttype: TokenType) -> bool {
        self.current.ttype() == ttype
    }

    pub fn match_token(&mut self, ttype: TokenType) -> bool {
        if self.check_token(ttype) {
            self.advance();
            true
        } else {
            false
        }
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
        let byte = self.make_constant(value);
        self.emit_byte_two(OpCode::Constant as u8, byte as u8);
    }

    fn make_constant(&mut self, value: Value) -> usize {
        let ret = self.compiling_chunk.add_constant(value);
        if ret > u8::MAX as usize {
            self.error_at(None, "Too many constants in one chunk.");
            0
        } else {
            ret
        }
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

    pub fn declaration(&mut self) {
        if self.match_token(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode { self.synchronize(); }
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expected variable name.");

        if self.match_token(TokenType::Equal) {
            self.expression();
        } else {
            self.error_at(None, "Expected '=' after variable name.");
        }

        self.consume(TokenType::Semicolon, "Expected ';' after variable declaration.");

        self.define_variable(global);
    }

    pub fn statement(&mut self) {
        if self.match_token(TokenType::Print) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expected ';' after value.");
        self.emit_byte(OpCode::Print as u8);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expected ';' after expression.");
        self.emit_byte(OpCode::Pop as u8);
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.current.ttype() != TokenType::Eof {
            if self.previous.ttype() == TokenType::Semicolon { return; }
            match self.current.ttype() {
                TokenType::Class | TokenType::Fun | TokenType::Var | TokenType::For | TokenType::If | TokenType::While | TokenType::Print | TokenType::Return => return,
                _ => {}
            }
            self.advance();
        }
    }

    fn binary(&mut self, _: bool) {
        let operator = self.previous.ttype();
        let rule = ParseRule::get(operator);
        self.parse_precedence(Precedence::from(rule.precedence() as u8 + 1));

        match operator {
            TokenType::BangEqual => self.emit_byte_two(OpCode::Equal as u8, OpCode::Not as u8),
            TokenType::EqualEqual => self.emit_byte(OpCode::Equal as u8),
            TokenType::Greater => self.emit_byte(OpCode::Greater as u8),
            TokenType::GreaterEqual => self.emit_byte_two(OpCode::Less as u8, OpCode::Not as u8),
            TokenType::Less => self.emit_byte(OpCode::Less as u8),
            TokenType::LessEqual => self.emit_byte_two(OpCode::Greater as u8, OpCode::Not as u8),
            TokenType::Plus => self.emit_byte(OpCode::Add as u8),
            TokenType::Minus => self.emit_byte(OpCode::Subtract as u8),
            TokenType::Star => self.emit_byte(OpCode::Multiply as u8),
            TokenType::Slash => self.emit_byte(OpCode::Divide as u8),
            _ => {}
        }
    }

    fn literal(&mut self, _: bool) {
        match self.previous.ttype() {
            TokenType::False => self.emit_byte(OpCode::False as u8),
            TokenType::True => self.emit_byte(OpCode::True as u8),
            TokenType::Nil => self.emit_byte(OpCode::Nil as u8),
            _ => {}
        }
    }

    fn grouping(&mut self, _: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expected ')' after expression.");
    }

    fn number(&mut self, _: bool) {
        let value = self.previous.lexeme().parse::<f64>().unwrap();
        self.emit_constant(Value::Number(value));
    }

    fn string(&mut self, _: bool) {
        let value = Object::String(self.previous.lexeme()[1..self.previous.lexeme().len() - 1].to_string());
        let value = Rc::new(value);
        self.emit_constant(Value::Object(value));
    }

    fn variable(&mut self, can_assign: bool) {
        let previous = self.previous.clone();
        self.named_variable(&previous, can_assign);
    }

    fn named_variable(&mut self, name: &Token, can_assign: bool) {
        let arg = self.identifier_constant(name);

        if can_assign && self.match_token(TokenType::Equal) {
            self.expression();
            self.emit_byte_two(OpCode::SetGlobal as u8, arg as u8);
        } else {
            self.emit_byte_two(OpCode::GetGlobal as u8, arg as u8);
        }
    }

    fn unary(&mut self, _: bool) {
        let operator_type = self.previous.ttype();

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Bang => self.emit_byte(OpCode::Not as u8),
            TokenType::Minus => self.emit_byte(OpCode::Negate as u8),
            _ => {}
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        let can_assign = precedence <= Precedence::Assignment;

        if let Some(prefix_rule) = ParseRule::get(self.previous.ttype()).prefix() {
            prefix_rule(self, can_assign);
        } else {
            self.error_at(None, "Expected expression.");
            return;
        }

        while precedence <= ParseRule::get(self.current.ttype()).precedence() {
            self.advance();
            if let Some(infix_rule) = ParseRule::get(self.previous.ttype()).infix() {
                infix_rule(self, can_assign);
            }
        }
    }

    fn parse_variable(&mut self, error_message: &str) -> usize {
        self.consume(TokenType::Identifier, error_message);
        let token = self.previous.clone();
        self.identifier_constant(&token)
    }

    fn define_variable(&mut self, global: usize) {
        self.emit_byte_two(OpCode::DefineGlobal as u8, global as u8);
    }

    fn identifier_constant(&mut self, name: &Token) -> usize {
        self.make_constant(Value::Object(Rc::new(Object::String(name.lexeme().to_string()))))
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

pub type ParseFn = fn(&mut Parser, bool) -> ();

pub struct ParseRule {
    prefix: Option<ParseFn>,
    infix: Option<ParseFn>,
    precedence: Precedence,
}

const RULES: [ParseRule; 40] = [
    ParseRule { prefix: Some(|x: &mut Parser, y: bool| x.grouping(y)), infix: None, precedence: Precedence::None }, // Token::LeftParen
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::RightParen
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::LeftBrace
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::RightBrace
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Comma
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Dot
    ParseRule { prefix: Some(|x: &mut Parser, y: bool| x.unary(y)), infix: Some(|x: &mut Parser, y: bool| x.binary(y)), precedence: Precedence::Term }, // Token::Minus
    ParseRule { prefix: None, infix: Some(|x: &mut Parser, y: bool| x.binary(y)), precedence: Precedence::Term }, // Token::Plus
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Semicolon
    ParseRule { prefix: None, infix: Some(|x: &mut Parser, y: bool| x.binary(y)), precedence: Precedence::Factor }, // Token::Slash
    ParseRule { prefix: None, infix: Some(|x: &mut Parser, y: bool| x.binary(y)), precedence: Precedence::Factor }, // Token::Star
    ParseRule { prefix: Some(|x: &mut Parser, y: bool| x.unary(y)), infix: None, precedence: Precedence::None }, // Token::Bang
    ParseRule { prefix: None, infix: Some(|x: &mut Parser, y: bool| x.binary(y)), precedence: Precedence::Equality }, // Token::BangEqual
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Equal
    ParseRule { prefix: None, infix: Some(|x: &mut Parser, y: bool| x.binary(y)), precedence: Precedence::Equality }, // Token::EqualEqual
    ParseRule { prefix: None, infix: Some(|x: &mut Parser, y: bool| x.binary(y)), precedence: Precedence::Comparison }, // Token::Greater
    ParseRule { prefix: None, infix: Some(|x: &mut Parser, y: bool| x.binary(y)), precedence: Precedence::Comparison }, // Token::GreaterEqual
    ParseRule { prefix: None, infix: Some(|x: &mut Parser, y: bool| x.binary(y)), precedence: Precedence::Comparison }, // Token::Less
    ParseRule { prefix: None, infix: Some(|x: &mut Parser, y: bool| x.binary(y)), precedence: Precedence::Comparison }, // Token::LessEqual
    ParseRule { prefix: Some(|x: &mut Parser, y: bool| x.variable(y)), infix: None, precedence: Precedence::None }, // Token::Identifier
    ParseRule { prefix: Some(|x: &mut Parser, y: bool| x.string(y)), infix: None, precedence: Precedence::None }, // Token::String
    ParseRule { prefix: Some(|x: &mut Parser, y: bool| x.number(y)), infix: None, precedence: Precedence::None }, // Token::Number
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::And
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Class
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Else
    ParseRule { prefix: Some(|x: &mut Parser, y: bool| x.literal(y)), infix: None, precedence: Precedence::None }, // Token::False
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::For
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Fun
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::If
    ParseRule { prefix: Some(|x: &mut Parser, y: bool| x.literal(y)), infix: None, precedence: Precedence::None }, // Token::Nil
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Or
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Print
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Return
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::Super
    ParseRule { prefix: None, infix: None, precedence: Precedence::None }, // Token::This
    ParseRule { prefix: Some(|x: &mut Parser, y: bool| x.literal(y)), infix: None, precedence: Precedence::None }, // Token::True
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