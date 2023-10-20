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
    pub p3: ManyVariants,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct TupleStruct(bool);

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum ManyVariants {
    One,
    Two(u32),
    Three(Option<u32>),
    Four { a: u32, b: Option<u16> },
    Five(String, u32),
    Six((u32,)),
}

command_handlers!(
    use gstd::debug;

    // This
    fn do_this(
        p1: u32,
        p2: String,
        p3: (Option<String>, u8),
        p4: TupleStruct,
    ) -> Result<(String, u32), String> {
        debug!("Handling 'do_this': {}, {}", p1, p2);
        Ok((p2, p1))
    }

    // That
    fn do_that(param: DoThatParam) -> Result<(String, u32), (String,)> {
        debug!("Handling 'do_that': {:?}", param);
        Ok((param.p2, param.p1))
    }

    // Fail
    fn fail(message: String) -> Result<(), String> {
        debug!("Handling 'fail': {}", message);
        Err(message)
    }
);
