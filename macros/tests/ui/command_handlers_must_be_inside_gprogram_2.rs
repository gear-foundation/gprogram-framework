use gprogram_framework_macros::{command_handlers, gprogram};

#[gprogram]
mod gprogram {
    use super::*;

    mod nested {
        use super::*;

        command_handlers!(
            fn set_value(value: u32) -> Result<(), ()> {
                Ok(())
            }
        );
    }
}

fn main() {}
