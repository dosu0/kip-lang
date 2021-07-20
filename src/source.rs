#[derive(Debug, Clone)]
pub struct Source {
    pub contents: String,
    /// can either be the file name (`src/foo.kip`) or something like (`<stdin>`)
    pub name: String,
}

impl Source {
    pub fn new<T: Into<String>>(contents: T, name: T) -> Self {
        Self {
            contents: contents.into(),
            name: name.into(),
        }
    }
}
