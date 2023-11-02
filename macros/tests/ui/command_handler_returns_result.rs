use gprogram_framework_macros::{command_handlers, gprogram};

#[gprogram]
mod gprogram {
    use super::*;

    command_handlers!(
        fn do_this(value: u32) -> u32 {
            value
        }
    );
}

fn main() {}
