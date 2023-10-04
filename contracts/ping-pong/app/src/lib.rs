#![no_std]

use gmeta::{InOut, Metadata};
use gstd::prelude::*;

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum Commands {
    Ping(String),
    DoThis(DoThisParam),
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum CommandResponses {
    Pong(Result<String, String>),
    DoThis(Result<DoThisParam, String>),
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct DoThisParam {
    pub p1: u32,
    pub p2: String,
}

pub struct ProgramMetadata;

impl Metadata for ProgramMetadata {
    type Init = ();
    type Handle = InOut<Commands, CommandResponses>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = ();
}

#[cfg(not(feature = "contract-io"))]
pub mod wasm {
    use super::*;

    #[no_mangle]
    extern "C" fn handle() {
        gstd::debug!("Handling some request");
        let command =
            gstd::msg::load::<Commands>().expect("This needs to be returned as some valid error");
        match command {
            Commands::Ping(msg) => {
                gstd::debug!("Handling ping: {}", msg);
                gstd::msg::reply(CommandResponses::Pong(Ok(msg)), 0)
                    .expect("This needs to be handled via signal");
            }
            Commands::DoThis(param) => {
                gstd::debug!("Handling do this: {:?}", param);
                do_this(&param);
                gstd::msg::reply(CommandResponses::DoThis(Ok(param)), 0)
                    .expect("This needs to be handled via signal");
            }
        }
    }

    fn do_this(_param: &DoThisParam) {}
}
