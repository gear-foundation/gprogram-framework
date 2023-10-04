#![no_std]

use gmeta::{InOut, Metadata};
use gprogram_framework_macros::command_handlers;
use gstd::prelude::*;

pub struct ProgramMetadata;

impl Metadata for ProgramMetadata {
    type Init = ();
    type Handle = InOut<Commands, CommandResponses>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = ();
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct DoThatParam {
    pub p1: u32,
    pub p2: String,
}

command_handlers!(
    use gstd::debug;

    // This
    fn do_this(p1: u32, p2: String) -> Result<(String, u32), String> {
        debug!("Handling do this: {} {}", p1, p2);
        Ok((p2, p1))
    }

    // That
    fn do_that(param: DoThatParam) -> Result<(String, u32), String> {
        debug!("Handling do that: {:?}", param);
        Ok((param.p2, param.p1))
    }
);
