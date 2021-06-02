use std::str::Chars;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Token {
    Def,
    Extern,
    Identifier(String),
    Number(f64),
    Unknown(char),
}

struct Tokenizer<'a> {
    chars: Chars<'a>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars(),
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        let fst_char = self.bump()?;

        match fst_char {
            // Line Comments ('//')
            '/' => {
                if self.first() == '/' {
                    self.bump();
                    while self.first() != '\n' {
                        self.bump();
                    }
                    self.next_token()
                } else {
                    Some(Token::Unknown('/'))
                }
            }

            // Skip whitespace
            c if c.is_ascii_whitespace() => {
                while self.first().is_ascii_whitespace() {
                    self.bump();
                }

                self.next_token()
            }

            // Identifiers
            c if c.is_ascii_alphabetic() => {
                let mut s = String::from(c);
                while self.first().is_ascii_alphanumeric() {
                    s.push(self.bump().unwrap());
                }

                Some(if s == "def" {
                    Token::Def
                } else if s == "extern" {
                    Token::Extern
                } else {
                    Token::Identifier(s)
                })
            }

            // Number Literals
            c if c.is_ascii_digit() => {
                let mut s = String::from(c);

                while self.first().is_ascii_digit() {
                    s.push(self.bump().unwrap());
                }

                // Handle decimal points
                if self.first() == '.' {
                    s.push(self.bump().unwrap());
                    while self.first().is_ascii_digit() {
                        s.push(self.bump().unwrap());
                    }
                }

                dbg!(&s);

                // TODO: Add error handling
                match s.parse() {
                    Ok(n) => Some(Token::Number(n)),
                    Err(_) => None,
                }
            }

            c @ _ => Some(Token::Unknown(c)),
        }
    }

    // peek
    fn first(&self) -> char {
        self.chars.clone().nth(0).unwrap_or('\0')
    }

    // peek again
    fn second(&self) -> char {
        self.chars.clone().nth(1).unwrap_or('\0')
    }

    /// Move to the next character
    fn bump(&mut self) -> Option<char> {
        self.chars.next()
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_and_unknown_tokens() {
        let input = "def func()";
        let mut tokenizer = Tokenizer::new(input);
        assert_eq!(tokenizer.next(), Some(Token::Def));
        // Should skip whitespace...
        assert_eq!(
            tokenizer.next(),
            Some(Token::Identifier(String::from("func")))
        );
        assert_eq!(tokenizer.next(), Some(Token::Unknown('(')));
        assert_eq!(tokenizer.next(), Some(Token::Unknown(')')));
        // Should return None when done lexing...
        assert_eq!(tokenizer.next(), None);
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn numbers() {
        let input = "42 3.14";
        let mut tokenizer = Tokenizer::new(input);
        assert_eq!(tokenizer.next(), Some(Token::Number(42.0)));
        assert_eq!(tokenizer.next(), Some(Token::Number(3.14)));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn comments() {
        let input = "// this is a comment\n/// this is another\n42 3.14";
        let mut tokenizer = Tokenizer::new(input);
        assert_eq!(tokenizer.next(), Some(Token::Number(42.0)));
        assert_eq!(tokenizer.next(), Some(Token::Number(3.14)));
        assert_eq!(tokenizer.next(), None);
    }
}
