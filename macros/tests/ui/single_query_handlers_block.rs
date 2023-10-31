use gprogram_framework_macros::{gprogram, query_handlers};

#[gprogram]
mod gprogram {
    use super::*;

    query_handlers!(
        fn this(value: u32) -> Result<u32, ()> {
            Ok(value)
        }
    );

    query_handlers!(
        fn that(value: u32) -> Result<u32, ()> {
            Ok(value)
        }
    );
}

fn main() {}
