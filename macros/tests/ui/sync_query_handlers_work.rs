use gprogram_framework_macros::{gprogram, query_handlers};
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

    query_handlers!(
        fn this(p1: u32, p2: String) -> Result<(String, u32), u32> {
            Ok((p2, p1))
        }

        fn that(p1: DoThatParam) -> Result<(u32, String), String> {
            Ok((p1.p1, p1.p2))
        }
    );
}

fn main() {
    let _this_query = gprogram::Queries::This(1, "2".into());
    let _that_query = gprogram::Queries::That(DoThatParam {
        p1: 1,
        p2: "2".into(),
    });
}
