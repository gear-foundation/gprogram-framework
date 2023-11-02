use gprogram_framework_macros::{gprogram, query_handlers};

#[gprogram]
mod gprogram {}

query_handlers!(
    fn value(value: u32) -> Result<u32, ()> {
        Ok(value)
    }
);

fn main() {}
