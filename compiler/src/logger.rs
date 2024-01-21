use crate::ast::region::{LineColumn, Region};
use crate::source::Source;
use crate::token::Token;

/// Build an error message
/// ```ignore
/// let source = Source::new("<foo>", "bar");
/// ErrorReporter::new()
///     .source(&source)
///     .message("error message goes here")
///     .report();
/// ```
#[derive(Default)]
pub struct ErrorReporter<'a> {
    source: Option<&'a Source>,
    region: Option<Region>,
    token: Option<Token>,
    message: &'static str,
}

impl<'a> ErrorReporter<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn source(mut self, source: &'a Source) -> Self {
        self.source = Some(source);
        self
    }

    pub fn region(mut self, region: Region) -> Self {
        self.region = Some(region);
        self
    }

    pub fn token(mut self, token: Token) -> Self {
        self.token = Some(token);
        self
    }

    pub fn message(mut self, message: &'static str) -> Self {
        self.message = message;
        self
    }

    pub fn report(self) {
        eprintln!("Error: {}", self.message);
        let Some(source) = self.source else {
            return;
        };

        if let Some(token) = self.token {
            let LineColumn { line, column } = token.region.line_column(source);
            eprintln!(
                "{} Line {} | {}",
                source.name,
                line,
                &source.contents[token.region.start() - column..token.region.end()]
            );
        } else if let Some(region) = self.region {
            let LineColumn { line, .. } = region.line_column(source);
            eprintln!(
                "{} Line {} | {}",
                source.name,
                line,
                &source.contents[region.start()..region.end()]
            );
        }
    }
}
