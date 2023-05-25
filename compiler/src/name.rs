pub use ustr::{existing_ustr as existing_name, ustr as name, Ustr as Name};

pub mod nm {
    use super::*;

    lazy_static! {
        pub static ref UINT8: Name = name("uint8");
        pub static ref UINT16: Name = name("uint16");
        pub static ref UINT32: Name = name("uint32");
        pub static ref UINT64: Name = name("uint64");
        pub static ref INT8: Name = name("int8");
        pub static ref INT16: Name = name("int16");
        pub static ref INT32: Name = name("int32");
        pub static ref INT64: Name = name("int64");
        pub static ref BOOL: Name = name("bool");
    }
}
