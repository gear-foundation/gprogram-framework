use gprogram_framework_macros::{command_handlers, gprogram, query_handlers};
use parity_scale_codec::Encode;

#[gprogram]
mod gprogram {
    use super::*;

    query_handlers!(
        fn get_value() -> Result<u32, ()> {
            Ok(42)
        }
    );

    command_handlers!(
        fn set_value(_value: u32) -> Result<(), ()> {
            Ok(())
        }
    );
}

fn main() {}
