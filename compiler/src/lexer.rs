use std::iter::Peekable;
use std::str::Chars;

use crate::ast::lit::Lit;
use crate::ast::region::Region;
use crate::name::name;
use crate::source::Source;
use crate::token::{Token, TokenKind, TokenKind::*};

#[derive(Debug)]
pub struct Lexer<'a> {
    input: &'a Source,
    tokens: Vec<Token>,
    /// the offset to first character in the token
    start: usize,
    /// the current offset from the start of the file, in other words the current character being
    /// looked at
    current: usize,
    chars: Peekable<Chars<'a>>,
}

// identifiers can start with letters from the alphabet or underscores
fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

// identifiers can continue with digits
fn is_ident_cont(ch: char) -> bool {
    is_ident_start(ch) || ch.is_ascii_digit()
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a Source) -> Self {
        Self {
            input,
            tokens: Vec::new(),
            current: 0,
            start: 0,
            chars: input.contents.chars().peekable(),
        }
    }

    pub fn lex(&mut self) -> Vec<Token> {
        while !self.is_eof() {
            self.start = self.current;
            self.lex_token();
        }

        self.add_token(Eof);
        // TODO: get rid of this clone
        return self.tokens.clone();
    }

    pub fn lex_token(&mut self) {
        let c = self.eat().unwrap();

        match c {
            '/' if self.next_is('/') => self.line_comment(),
            '/' if self.next_is('*') => self.block_comment(),

            // Skip whitespace
            ch if ch.is_ascii_whitespace() => {
                while self.peek().is_ascii_whitespace() && !self.is_eof() {
                    self.eat();
                }
            }

            // Identifiers and reserved words
            ch if is_ident_start(ch) => self.identifier(),

            // Number Literals
            // NOTE: Not sure if i should let numbers start with 0
            '0'..='9' => self.number(),
            // string literal e.g. (`"foo"`)
            '"' => self.string(),

            '\'' => {
                let c = if let Some(c) = self.eat() {
                    c
                } else {
                    eprintln!("Error: Expected a character in a character literal");
                    return;
                };

                if let Some('\'') = self.eat() {
                    self.add_token(Literal(Lit::Char(c)));
                } else {
                    eprintln!("Error: Unterminated character literal.");
                }
            }

            '@' => {
                self.eat();
                if is_ident_start(c) {
                    while is_ident_cont(self.peek()) && !self.is_eof() {
                        self.eat();
                    }

                    if self.is_eof() {
                        eprintln!("expected an identifier after a pre processor statement");
                        return;
                    }

                    // exclude the '@'
                    match &self.input.contents[self.start + 1..self.current] {
                        "impt" => self.add_token(Impt),
                        "expt" => self.add_token(Expt),
                        s => eprintln!("unknown preproc directive `@{}`", s),
                    }
                } else {
                    eprintln!("Error: expected identifier after `@`");
                }
            }

            '>' => match self.next_is('=') {
                true => self.add_token(Ge),
                false => self.add_token(Gt),
            },
            '<' => match self.next_is('=') {
                true => self.add_token(Le),
                false => self.add_token(Lt),
            },
            '&' => match self.next_is('&') {
                true => self.add_token(DoubleAmpersand),
                false => self.add_token(Ampersand),
            },
            '|' => match self.next_is('|') {
                true => self.add_token(DoubleBar),
                false => self.add_token(Bar),
            },

            '(' => self.add_token(OpenParen),
            ')' => self.add_token(CloseParen),
            '{' => self.add_token(OpenBrace),
            '}' => self.add_token(CloseBrace),
            ',' => self.add_token(Comma),
            ';' => self.add_token(Semicolon),
            '+' => self.add_token(Plus),
            '-' => self.add_token(Minus),
            '*' => self.add_token(Star),
            ':' => self.add_token(Colon),
            '%' => self.add_token(Percent),
            '=' => match self.next_is('=') {
                true => self.add_token(DoubleEqual),
                false => self.add_token(Equal),
            },
            '.' => self.add_token(Dot),
            '/' => self.add_token(Slash),
            _ => eprintln!("Error: unexpected character"),
        }
    }

    fn line_comment(&mut self) {
        while self.peek() != '\n' && !self.is_eof() {
            self.eat();
        }
    }

    fn block_comment(&mut self) {
        loop {
            let c = if let Some(c) = self.eat() {
                c
            } else {
                return eprintln!("unterminated block comment");
            };

            if c == '*' && self.next_is('/') {
                self.eat();
                break;
            }
        }
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.eat();
        }

        // Handle decimal points
        if self.peek() == '.' {
            // TODO: add back float support, I removed it for easier code generation
            todo!("floating-point numbers");
        }

        let num = self.input.contents[self.start..self.current].parse();
        if let Ok(num) = num {
            self.add_token(Literal(Lit::Int(num)));
        } else if let Err(e) = num {
            eprintln!("Error: invalid integer literal: {}", e);
        }
    }

    fn string(&mut self) {
        while self.peek() != '"' {
            self.eat();
        }

        self.eat();

        // exclude the quotation marks from the string literal
        let s = self.slice(self.start + 1, self.current - 1);
        self.add_token(Literal(Lit::Str(name(s))));
    }

    fn identifier(&mut self) {
        while is_ident_cont(self.peek()) {
            self.eat();
        }

        let s = self.slice(self.start, self.current);

        match s {
            "func" => self.add_token(Func),
            "extern" => self.add_token(Extern),
            "var" => self.add_token(Var),
            "if" => self.add_token(If),
            "while" => self.add_token(While),
            "else" => self.add_token(Else),
            "ret" => self.add_token(Ret),
            _ => self.add_token(Ident(name(s))),
        }
    }

    /// Returns `true` and [`eat`]s next character matches the provided character, otherwise returns `false`
    fn next_is(&mut self, c: char) -> bool {
        if self.is_eof() || self.peek() != c {
            false
        } else {
            self.eat();
            true
        }
    }

    /// Looks at the current un-eaten character
    fn peek(&mut self) -> char {
        *self.chars.peek().unwrap_or(&'\0')
    }

    /// Move to the next character
    fn eat(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.current += c.len_utf8();
        Some(c)
    }

    fn is_eof(&self) -> bool {
        self.current == self.input.contents.len()
    }

    fn add_token(&mut self, kind: TokenKind) {
        self.tokens
            .push(Token::new(kind, Region::new(self.start, self.current)))
    }

    fn slice(&self, start: usize, end: usize) -> &str {
        &self.input.contents[start..end]
    }

    /// Get a reference to the token stream's input.
    pub fn input(&self) -> &Source {
        self.input
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_and_chars() {
        let input = "\"foo\" '7'";
        let source = Source::new(input, "<string literal>");
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.lex();
        assert_eq!(tokens[0], Literal(Lit::Str(name("foo"))));
        assert_eq!(tokens[1], Literal('7'.into()));
    }

    #[test]
    fn identifiers_and_parens() {
        let input = "func foo()";
        let source = Source::new(input, "<string literal>");
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.lex();
        assert_eq!(tokens[0], Func);
        // Should skip whitespace...
        assert_eq!(tokens[1], Ident(name("foo")));
        assert_eq!(tokens[2], OpenParen);
        assert_eq!(tokens[3], CloseParen);
        assert_eq!(tokens[4], Eof);
    }

    #[test]
    fn comparison() {
        let input = "9000 == 1";
        let source = Source::new(input, "<string literal>");
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.lex();
        assert_eq!(tokens[0], Literal(9000.into()));
        assert_eq!(tokens[1], DoubleEqual);
        assert_eq!(tokens[2], Literal(1.into()));
    }

    #[test]
    fn numbers_ops_and_commas() {
        let input = "42 + 3, 69 * 100, 1000";
        let source = Source::new(input, "<string literal>");
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.lex();
        assert_eq!(tokens[0], Literal(42.into()));
        assert_eq!(tokens[1], Plus);
        assert_eq!(tokens[2], Literal(3.into()));
        assert_eq!(tokens[3], Comma);
        assert_eq!(tokens[4], Literal(69.into()));
        assert_eq!(tokens[5], Star);
        assert_eq!(tokens[6], Literal(100.into()));
        assert_eq!(tokens[7], Comma);
        assert_eq!(tokens[8], Literal(1000.into()));
        assert_eq!(tokens[9], Eof);
    }

    #[test]
    fn comments() {
        let input = "\
// this is a line comment\n
foo\n
/**\n
 * this is a block comment\n
 */\n
bar
// another line comment\n 
baz";

        let source = Source::new(input, "<string literal>");
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.lex();
        assert_eq!(tokens[0].kind, Ident(name("foo")));
        assert_eq!(tokens[1].kind, Ident(name("bar")));
        assert_eq!(tokens[2].kind, Ident(name("baz")));
        assert_eq!(tokens[3].kind, Eof);
    }
}
