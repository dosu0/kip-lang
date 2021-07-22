///! The kip lexer
use std::fmt;
use std::str::Chars;
pub use Token::*;

use crate::ast::BinOp;
use crate::source::Source;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Token {
    // key / reserved words
    Func,
    Extern,
    Var,
    If,
    Else,
    Ret,
    Impt,
    Expt,

    // primary
    Ident(String),
    /// TODO: add back float support, I removed it for easier code generation
    Number(i64),
    Str(String),

    // symbols
    /// `(`
    OpenParen,
    /// `)`
    CloseParen,
    /// `{`
    OpenBrace,
    /// `}`
    CloseBrace,
    /// `,`
    Comma,
    /// `:`
    Colon,
    /// `;`
    Semicolon,

    // operators
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `/`
    Slash,
    /// `*`
    Star,
    /// `=`
    Equal,
    /// `:=` maybe not?
    /// Assign,
    /// `%`
    Percent,
    /// `>`
    Gt,
    /// `>=`
    Ge,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// '.'
    Dot,
    Unknown(char),
    Eof,
}

impl Token {
    // convert lexical token to its corresponding binary operator
    pub fn to_bin_op(&self) -> Option<BinOp> {
        match self {
            Plus => Some(BinOp::Add),
            Minus => Some(BinOp::Sub),
            Slash => Some(BinOp::Div),
            Star => Some(BinOp::Mul),
            Equal => Some(BinOp::Eq),
            Percent => Some(BinOp::Mod),
            Gt => Some(BinOp::Gt),
            Ge => Some(BinOp::Ge),
            Lt => Some(BinOp::Lt),
            Le => Some(BinOp::Le),
            _ => None,
        }
    }
}
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Func => "func".fmt(f),
            Extern => "extern".fmt(f),
            Var => "var".fmt(f),
            Ret => "return".fmt(f),
            If => "if".fmt(f),
            Else => "else".fmt(f),
            Impt => "@impt".fmt(f),
            Expt => "@expt".fmt(f),
            Ident(s) => s.fmt(f),
            Number(v) => v.fmt(f),
            Str(s) => write!(f, "\"{}\"", s.escape_debug()),
            OpenParen => ')'.fmt(f),
            CloseParen => '('.fmt(f),
            OpenBrace => '{'.fmt(f),
            CloseBrace => '}'.fmt(f),
            Colon => ':'.fmt(f),
            Comma => ','.fmt(f),
            Semicolon => ';'.fmt(f),
            Plus => '+'.fmt(f),
            Minus => '-'.fmt(f),
            Star => '*'.fmt(f),
            Slash => '/'.fmt(f),
            Dot => '.'.fmt(f),
            // Assign => ":=".fmt(f),
            Percent => "%".fmt(f),
            Gt => '>'.fmt(f),
            Ge => ">=".fmt(f),
            Lt => "<".fmt(f),
            Le => "<=".fmt(f),
            Equal => '='.fmt(f),
            Unknown(ch) => ch.fmt(f),
            Eof => "end of file".fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenStream<'a> {
    input: &'a Source,
    chars: Chars<'a>,
    line: usize,
    col: usize,
    /// offset
    offset: usize,
}

fn is_id_start(ch: char) -> bool {
    ('a'..='z').contains(&ch) || ('A'..='Z').contains(&ch) || ch == '_'
}

fn is_id_cont(ch: char) -> bool {
    ('a'..='z').contains(&ch) || ('A'..='Z').contains(&ch) || ('0'..='9').contains(&ch) || ch == '_'
}

impl<'a> TokenStream<'a> {
    pub fn new(input: &'a Source) -> Self {
        Self {
            input,
            chars: input.contents.chars(),
            offset: 0,
            line: 1,
            col: 0,
        }
    }

    pub fn eat(&mut self) -> Token {
        let fst_ch = match self.bump() {
            Some(c) => c,
            None => return Token::Eof,
        };

        match fst_ch {
            '/' => match self.peek_ch() {
                // Line Comments ('//')
                '/' => {
                    self.bump().unwrap();

                    let mut ch = self.peek_ch();

                    while ch != '\n' && ch != '\0' {
                        self.bump();
                        ch = self.peek_ch();
                    }

                    self.eat()
                }

                // block comment
                '*' => {
                    self.bump().unwrap();

                    loop {
                        if let Some(ch) = self.bump() {
                            if ch == '*' && self.peek_ch() == '/' {
                                self.bump().unwrap();
                                break;
                            }
                        } else {
                            // TODO: better error handling :|
                            panic!("unterminated block comment");
                        }
                    }
                    self.eat()
                }
                _ => Token::Slash,
            },
            // Skip whitespace
            ch if ch.is_ascii_whitespace() => {
                while self.peek_ch().is_ascii_whitespace() {
                    self.bump();
                }

                self.eat()
            }

            // Identifiers and reserved words
            ch if is_id_start(ch) => {
                let mut s = ch.to_string();
                while is_id_cont(self.peek_ch()) {
                    s.push(self.bump().unwrap());
                }

                match &*s {
                    "func" => Func,
                    "extern" => Extern,
                    "var" => Var,
                    "if" => If,
                    "else" => Else,
                    "ret" => Ret,
                    _ => Ident(s),
                }
            }

            // Number Literals
            ch @ '0'..='9' => {
                let mut s = ch.to_string();

                while self.peek_ch().is_ascii_digit() {
                    s.push(self.bump().unwrap());
                }

                // Handle decimal points
                if self.peek_ch() == '.' {
                    // TODO: add back float support, I removed it for easier code generation
                    todo!("floating-point numbers");

                    /* s.push(self.bump().unwrap());
                    while self.peek_ch().is_ascii_digit() {
                        s.push(self.bump().unwrap());
                    } */
                }

                // TODO: Add error handling
                Number(s.parse().unwrap())
            }

            // string literal e.g. (`"foo"`)
            '"' => {
                let mut s = String::new();

                while self.peek_ch() != '"' {
                    s.push(self.bump().unwrap_or_else(|| {
                        // TODO: improved error handling
                        panic!("unterminated string literal");
                    }));
                }

                self.bump();

                Str(s)
            }

            '@' => {
                let ch = self.bump().expect("expected identifier after `@`");
                if is_id_start(ch) {
                    let mut s = ch.to_string();
                    while is_id_cont(self.peek_ch()) {
                        s.push(self.bump().unwrap());
                    }

                    match &*s {
                        "impt" => Impt,
                        "expt" => Expt,
                        s => panic!("unknown preproc directive `@{}`", s),
                    }
                } else {
                    panic!("expected identifier after `@`");
                }
            }
            '(' => OpenParen,
            ')' => CloseParen,
            '{' => OpenBrace,
            '}' => CloseBrace,
            ',' => Comma,
            ';' => Semicolon,
            '>' => Gt,
            '<' => Lt,
            '+' => Plus,
            '-' => Minus,
            '*' => Star,
            ':' => Colon,
            '%' => Percent,
            '=' => Equal,
            '.' => Dot,
            ch => Unknown(ch),
        }
    }

    fn peek_ch(&self) -> char {
        self.chars.clone().next().unwrap_or('\0')
    }

    /// peek at the next token without '`eat`'ing it
    pub fn peek(&self) -> Token {
        self.clone().eat()
    }

    /// retreive line number
    pub fn line(&self) -> usize {
        self.line
    }

    /// retreive column number
    pub fn col(&self) -> usize {
        self.col
    }

    /// retreive character offset
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Move to the next character
    fn bump(&mut self) -> Option<char> {
        let ch = self.chars.next();

        self.offset += 1;
        match ch {
            Some('\n') => {
                self.line += 1;
                self.col = 0;
            }
            Some(_) => {
                self.col += 1;
            }
            _ => {}
        }

        ch
    }

    /// Get a reference to the token stream's input.
    pub fn input(&self) -> &Source {
        self.input
    }
}

impl<'a> Iterator for TokenStream<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        match self.eat() {
            Eof => None,
            t => Some(t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_and_parens() {
        let input = "func foo()";
        let source = Source::new(input, "<string literal>");
        let mut tokens = TokenStream::new(&source);
        assert_eq!(tokens.eat(), Func);
        // Should skip whitespace...
        assert_eq!(tokens.eat(), Ident("foo".into()));
        assert_eq!(tokens.eat(), OpenParen);
        assert_eq!(tokens.eat(), CloseParen);
        // Should return None when done lexing...
        assert_eq!(tokens.eat(), Eof);
        assert_eq!(tokens.eat(), Eof);
    }

    #[test]
    fn numbers_ops_and_commas() {
        let input = "42 + 3, 69 * 100, 1000";
        let source = Source::new(input, "<string literal>");
        let mut tokens = TokenStream::new(&source);
        assert_eq!(tokens.eat(), Number(42));
        assert_eq!(tokens.eat(), Plus);
        assert_eq!(tokens.eat(), Number(3));
        assert_eq!(tokens.eat(), Comma);
        assert_eq!(tokens.eat(), Number(69));
        assert_eq!(tokens.eat(), Star);
        assert_eq!(tokens.eat(), Number(100));
        assert_eq!(tokens.eat(), Comma);
        assert_eq!(tokens.eat(), Number(1000));
        assert_eq!(tokens.eat(), Eof);
    }

    #[test]
    fn comments() {
        let input = concat!(
            "// this is a line comment\n foo\n",
            "/**\n * this is a block comment\n */\n bar\n",
            "// another line comment\n baz",
        );

        let source = Source::new(input, "<string literal>");
        let mut tokens = TokenStream::new(&source);
        assert_eq!(tokens.eat(), Ident("foo".into()));
        assert_eq!(tokens.eat(), Ident("bar".into()));
        assert_eq!(tokens.eat(), Ident("baz".into()));
        assert_eq!(tokens.eat(), Eof);
    }
}
