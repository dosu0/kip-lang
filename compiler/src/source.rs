///! A simple struct that encapsulates the source code
///
use crate::ast::region::Region;

#[derive(Debug, Clone)]
pub struct Source {
    pub contents: String,
    /// can either be the file name (`src/foo.kip`) or something like (`<stdin>`)
    pub name: String,
}

pub struct Context {
    pub region: Region,
    pub line: usize,
}

impl Source {
    pub fn new<T: Into<String>, U: Into<String>>(contents: T, name: U) -> Self {
        Self {
            contents: contents.into(),
            name: name.into(),
        }
    }

    /// Retrieves the character at the provided index
    pub fn char_at(&self, idx: usize) -> char {
        self.contents.chars().nth(idx).unwrap_or('\0')
    }

    pub fn lines(&self) -> Vec<&str> {
        self.contents.lines().collect()
    }

    pub fn slice(&self, region: Region) -> &str {
        &self.contents[region.start()..region.end()]
    }

    /// Retreives the length in `char`s of the source's contents.
    pub fn len(&self) -> usize {
        self.contents.chars().count()
    }

    // this is a crappy way of trying to get the surrounding source code of a "Region" of code
    pub fn context_of(&self, region: Region) -> Context {
        let mut context_start = region.start();
        let mut context_end = region.end();

        if self.char_at(context_start - 1) == '\n' {
            context_start -= 2;

            for i in (0..context_start).rev() {
                if self.char_at(i).is_ascii() && (i == 0 || self.char_at(i - 1) == '\n') {
                    context_start = i;
                    break;
                }
            }
        } else {
            // find the first newline before the region
            for i in (0..region.start()).rev() {
                if self.char_at(i) == '\n' {
                    context_start = i + 1;
                    break;
                }
            }
        }

        // find the first newline after the region
        for idx in region.end()..self.len() {
            if self.char_at(idx) == '\n' {
                context_end = idx;
                break;
            }
        }

        let line_number = self.contents[0..context_end].lines().count();

        let region = Region::new(context_start, context_end);
        Context {
            region,
            line: line_number,
        }
    }
}
