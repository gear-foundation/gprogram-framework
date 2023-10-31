use gprogram_framework_macros::{command_handlers, gprogram};

#[gprogram]
mod gprogram {
    use super::*;

    command_handlers!(
        async fn do_this() -> Result<(), ()> {
            Ok(())
        }
    );
}

fn main() {}
