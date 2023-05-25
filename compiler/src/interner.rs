use macros::define_symbols;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::mem;

define_symbols! {
    bool,
    u8,
    u16,
    u32,
    u64,
    s8,
    s16,
    s32,
    s64
}

/// The global string interner
static INTERNER: RefCell<Interner> = RefCell::new(Interner::fresh());

pub fn with_interner<F: FnOnce(&mut Interner) -> R, R>(f: F) -> R {
    f(&mut INTERNER.borrow_mut())
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Symbol(u32);

impl Symbol {
    const fn new(idx: u32) -> Self {
        Symbol(idx)
    }

    fn as_usize(self) -> usize {
        self.0 as usize
    }

    /// Retreive the string from the global interner
    pub fn as_str(self) -> &'static str {
        with_interner(|interner| interner.get(self))
    }

    /// Add `string` to the global interner and return it's id
    pub fn intern(string: &str) -> Symbol {
        with_interner(|interner| interner.intern(string))
    }
}

impl fmt::Display for Symbol {
    /// Does the same thing as:
    /// ```
    /// self.as_str().fmt(f)
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}
#[derive(Default)]
/// An interner based on ["Fast and Simple Rust Interner"](https://matklad.github.io/2020/03/22/fast-simple-rust-interner.html) with minor modifications
pub struct Interner {
    names: HashMap<&'static str, Symbol>,
    strings: Vec<&'static str>,
    buf: String,
    full: Vec<String>,
}

impl Interner {
    pub const fn prefill(init: &[&'static str]) -> Self {
        // create symbols for each string in the init list
        let names = init.iter().copied().zip((0..).map(Symbol::new)).collect();
        Self {
            names,
            strings: Vec::from(init),
            ..Default::default()
        }
    }

    pub const fn new() -> Self {
        Default::default()
    }

    pub fn with_capacity(cap: usize) -> Self {
        let cap = cap.next_power_of_two();
        Self {
            buf: String::with_capacity(cap),
            ..Default::default()
        }
    }

    unsafe fn alloc(&mut self, name: &str) -> &'static str {
        let cap = self.buf.capacity();

        // if there isn't enough room for the new string then increase the capacity
        if cap < self.buf.len() + name.len() {
            let new_cap = (cap.max(name.len()) + 1).next_power_of_two();
            let new_buf = String::with_capacity(new_cap);
            let old_buf = mem::replace(&mut self.buf, new_buf);
            self.full.push(old_buf);
        }
        
        // get the new interned string as a slice of the internal buffer
        let interned = {
            let start = self.buf.len();
            self.buf.push_str(name);
            &self.buf[start..]
        };
        
        // SAFETY: gauranteed as long as the interner exists
        &*(interned as *const str)
    }

    pub fn get(&self, symbol: Symbol) -> &'static str {
        self.strings[symbol.as_usize()]
    }

    pub fn intern(&mut self, string: &str) -> Symbol {
        // if the symbol already exists just return it
        if let Some(&name) = self.names.get(string) {
            return name;
        }

        let string = unsafe { self.alloc(string) };
        let name = Symbol(self.names.len() as u32);
        self.names.insert(string, name);
        self.strings.push(string);

        name
    }
}
