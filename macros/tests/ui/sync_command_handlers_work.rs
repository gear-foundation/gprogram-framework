use gprogram_framework_macros::{command_handlers, gprogram};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct DoThatParam {
    p1: u32,
    p2: String,
}

#[gprogram]
mod gprogram {
    use super::*;

    command_handlers!(
        fn do_this(p1: u32, p2: String) -> Result<(String, u32), u32> {
            Ok((p2, p1))
        }

        fn do_that(p1: DoThatParam) -> Result<(u32, String), String> {
            Ok((p1.p1, p1.p2))
        }
    );
}

fn main() {
    let _do_this_cmd = gprogram::Commands::DoThis(1, "2".into());
    let _do_that_cmd = gprogram::Commands::DoThat(DoThatParam {
        p1: 1,
        p2: "2".into(),
    });
    //let _result = handle_impl(_do_this_cmd);
}
