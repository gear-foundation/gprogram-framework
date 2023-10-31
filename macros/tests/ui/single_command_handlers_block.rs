use gprogram_framework_macros::{command_handlers, gprogram};

#[gprogram]
mod gprogram {
    use super::*;

    command_handlers!(
        fn do_this(value: u32) -> Result<(), ()> {
            Ok(())
        }
    );

    command_handlers!(
        fn do_that(value: u32) -> Result<(), ()> {
            Ok(())
        }
    );
}

fn main() {}
