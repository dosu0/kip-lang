///! A lexer; the important code is contained in the `TokenStream` struct
use std::fmt;
use std::str::Chars;
pub use Token::*;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Token {
    // keywords
    Func,
    Extern,
    Var,

    // primary
    Ident(String),
    Number(f64),

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
    /// `;`
    Semicolon,

    // misc
    Op(char),
    Unknown(char),
    Eof,
}

impl fmt::Display for Token {
    /// TODO: refactor?
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Func => "func".fmt(f),
            Extern => "extern".fmt(f),
            Var => "var".fmt(f),
            Ident(s) => s.fmt(f),
            Number(v) => v.fmt(f),
            OpenParen => ')'.fmt(f),
            CloseParen => '('.fmt(f),
            OpenBrace => '{'.fmt(f),
            CloseBrace => '}'.fmt(f),
            Comma => ','.fmt(f),
            Semicolon => ';'.fmt(f),
            Op(ch) => ch.fmt(f),
            Unknown(ch) => ch.fmt(f),
            Eof => "end of file".fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenStream<'a> {
    chars: Chars<'a>,
    line: usize,
    col: usize,
}

fn is_id_start(ch: char) -> bool {
    ('a'..='z').contains(&ch) || ('A'..='Z').contains(&ch) || ch == '_'
}

fn is_id_cont(ch: char) -> bool {
    ('a'..='z').contains(&ch)
        || ('A'..='Z').contains(&ch)
        || ('0'..='9').contains(&ch)
        || ch == '_'
}

impl<'a> TokenStream<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars(),
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
            // Line Comments ('//')
            '/' => {
                if self.peek_ch() == '/' {
                    self.bump();

                    let mut ch = self.peek_ch();

                    while ch != '\n' && ch != '\0' {
                        self.bump();
                        ch = self.peek_ch();
                    }

                    self.eat()
                } else {
                    Token::Op('/')
                }
            }

            // Skip whitespace
            ch if ch.is_ascii_whitespace() => {
                while self.peek_ch().is_ascii_whitespace() {
                    self.bump();
                }

                self.eat()
            }

            // Identifiers and reserved words
            ch if is_id_start(ch) => {
                let mut s = String::from(ch);
                while is_id_cont(self.peek_ch()) {
                    s.push(self.bump().unwrap());
                }

                match &*s {
                    "func" => Func,
                    "extern" => Extern,
                    "var" => Var,
                    s => Ident(s.to_owned()),
                }
            }

            // Number Literals
            ch @ '0'..='9' => {
                let mut s = String::from(ch);

                while self.peek_ch().is_ascii_digit() {
                    s.push(self.bump().unwrap());
                }

                // Handle decimal points
                if self.peek_ch() == '.' {
                    s.push(self.bump().unwrap());
                    while self.peek_ch().is_ascii_digit() {
                        s.push(self.bump().unwrap());
                    }
                }

                // TODO: Add error handling
                Number(s.parse().unwrap())
            }

            '(' => OpenParen,
            ')' => CloseParen,
            '{' => OpenBrace,
            '}' => CloseBrace,
            ',' => Comma,
            ';' => Semicolon,
            ch @ ('>' | '<' | '+' | '-' | '*' | '=') => Op(ch),
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

    /// Move to the next character
    fn bump(&mut self) -> Option<char> {
        let ch = self.chars.next();

        match ch {
            Some('\n') => {
                self.line += 1;
                self.col = 0;
            }
            Some(_) => self.col += 1,
            _ => {}
        }

        ch
    }
}

impl<'a> Iterator for TokenStream<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        match self.eat() {
            Token::Eof => None,
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
        let mut tokens = TokenStream::new(input);
        assert_eq!(tokens.next(), Some(Token::Func));
        // Should skip whitespace...
        assert_eq!(tokens.next(), Some(Token::Ident(String::from("foo"))));
        assert_eq!(tokens.next(), Some(Token::OpenParen));
        assert_eq!(tokens.next(), Some(Token::CloseParen));
        // Should return None when done lexing...
        assert_eq!(tokens.next(), None);
        assert_eq!(tokens.next(), None);
    }

    #[test]
    fn numbers_ops_and_commas() {
        let input = "42 + 3.14, 69 * 100, 1000";
        let mut tokens = TokenStream::new(input);
        assert_eq!(tokens.next(), Some(Token::Number(42.0)));
        assert_eq!(tokens.next(), Some(Token::Op('+')));
        assert_eq!(tokens.next(), Some(Token::Number(3.14)));
        assert_eq!(tokens.next(), Some(Token::Comma));
        assert_eq!(tokens.next(), Some(Token::Number(69.0)));
        assert_eq!(tokens.next(), Some(Token::Op('*')));
        assert_eq!(tokens.next(), Some(Token::Number(100.0)));
        assert_eq!(tokens.next(), Some(Token::Comma));
        assert_eq!(tokens.next(), Some(Token::Number(1000.0)));
        assert_eq!(tokens.next(), None);
    }

    #[test]
    fn comments() {
        let input = "// this is a comment\n/// this is another\n42 3.14";
        let mut tokens = TokenStream::new(input);
        assert_eq!(tokens.next(), Some(Token::Number(42.0)));
        assert_eq!(tokens.next(), Some(Token::Number(3.14)));
        assert_eq!(tokens.next(), None);
    }
}
