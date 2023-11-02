use gprogram_framework_macros::{gprogram, query_handlers};

#[gprogram]
mod gprogram {
    use super::*;

    query_handlers!();
}

fn main() {}
