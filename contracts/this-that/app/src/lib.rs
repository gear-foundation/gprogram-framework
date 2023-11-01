#![no_std]

use gmeta::{InOut, Metadata};
use gprogram_framework_macros::{command_handlers, gprogram};
use gstd::prelude::*;

pub struct ProgramMetadata;

impl Metadata for ProgramMetadata {
    type Init = ();
    type Handle = InOut<gprogram::Commands, gprogram::CommandResponses>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = InOut<gprogram::Queries, gprogram::QueryResponses>;
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

#[gprogram]
mod gprogram {
    use gprogram_framework_macros::query_handlers;

    use super::*;

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

    query_handlers!(
        use gstd::debug;

        // This
        fn this() -> Result<u32, String> {
            debug!("Handling 'this'");
            Ok(42)
        }

        // That
        fn that() -> Result<String, String> {
            debug!("Handling 'that'");
            Ok("Forty two".into())
        }

        // Fail
        fn fail() -> Result<(), String> {
            debug!("Handling 'fail'");
            Err("Failed".into())
        }
    );
}
