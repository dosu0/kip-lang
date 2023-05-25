use ast::Region;
use source::Source;

#[macro_use]
extern crate lazy_static;

mod ast;
pub mod cli;
pub mod driver;
mod lexer;
pub mod logger;
mod name;
mod parser;
mod scopechk;
mod source;
mod token;

pub fn generate_error_message(base_message: &'static str, source: &Source, region: Region) -> String {
    let error_source_code = source.context_of(region);
    let line = error_source_code.line;
    let width = region.start() - error_source_code.region.start() + 1;
    let error_context = source.slice(error_source_code.region);
    return format!(
        "{base_message}\nin {}, line {line}\n{error_context}\n{:>width$}",
        source.name, '^'
    );
}
