// This file is part of Gear.

// Copyright (C) 2021-2023 Gear Technologies Inc.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use super::*;
use quote::quote;
use syn;

#[test]
fn command_handlers_works() {
    let input = quote! {
        fn do_this() {}
    };
    let expected = quote! {
        extern crate gstd;
        extern crate gsys;
        extern crate parity_scale_codec;
        extern crate scale_info;

        #[derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)]
        pub enum Commands {
            DoThis(),
        }

        #[derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)]
        pub enum CommandResponses {
            DoThis(()),
        }

        #[cfg(not(feature = "contract-io"))]
        pub mod wasm {
            use super::*;

            fn handle_impl(command: Commands) -> (CommandResponses, bool) {
                match command {
                    Commands::DoThis() => {
                        let result: Result<_, _> = do_this();
                        let is_error = result.is_err();
                        (CommandResponses::DoThis(result), is_error)
                    }
                }
            }

            #[no_mangle]
            extern "C" fn handle() {
                let command = gstd::msg::load::<Commands>().expect("This needs to be handled in a consistent way: input parse error");
                let (result, is_error) = handle_impl(command);
                let result = result.encode();
                if is_error {
                    unsafe { gsys::gr_panic(result.as_ptr(), result.len() as u32); }
                }
                gstd::msg::reply(result, 0) .expect("This needs to be handled in a consistent way: reply error");
            }

            fn do_this() {}
        }
    };
    assert_eq!(
        expected.to_string(),
        command_handlers_core(input).to_string()
    );
}

#[test]
fn query_handlers_works() {
    let input = quote! {
        fn this() {}
    };
    let expected = quote! {
        fn this() {}
    };
    assert_eq!(expected.to_string(), query_handlers_core(input).to_string());
}

#[test]
fn handler_parts_from_works_for_func_with_default_return_type() {
    let signature = syn::parse2::<syn::Signature>(quote! {
        fn do_this(p1: u32, p2: String)
    })
    .unwrap();

    let handler_parts = handler_parts::HandlerParts::from(
        &syn::Ident::new("Commands", proc_macro2::Span::call_site()),
        &syn::Ident::new("CommandResponses", proc_macro2::Span::call_site()),
        &signature,
    );

    assert_eq!(
        quote!(DoThis(u32, String,),).to_string(),
        handler_parts.request_enum_variant().to_string()
    );
    assert_eq!(
        quote!(DoThis(()),).to_string(),
        handler_parts.response_enum_variant().to_string()
    );
    assert_eq!(
        quote!(
            Commands::DoThis(v0, v1) => {
                let result: Result<_, _> = do_this(v0, v1);
                let is_error = result.is_err();
                (CommandResponses::DoThis(result), is_error)
            }
        )
        .to_string(),
        handler_parts.call_match_arm().to_string()
    );
    assert_eq!(false, handler_parts.is_async());
}

#[test]
fn handler_parts_from_works_for_func_without_args() {
    let signature = syn::parse2::<syn::Signature>(quote! {
        fn do_this()
    })
    .unwrap();

    let handler_parts = handler_parts::HandlerParts::from(
        &syn::Ident::new("Commands", proc_macro2::Span::call_site()),
        &syn::Ident::new("CommandResponses", proc_macro2::Span::call_site()),
        &signature,
    );

    assert_eq!(
        quote!(DoThis(),).to_string(),
        handler_parts.request_enum_variant().to_string()
    );
    assert_eq!(
        quote!(DoThis(()),).to_string(),
        handler_parts.response_enum_variant().to_string()
    );
    assert_eq!(
        quote!(
            Commands::DoThis() => {
                let result: Result<_, _> = do_this();
                let is_error = result.is_err();
                (CommandResponses::DoThis(result), is_error)
            }
        )
        .to_string(),
        handler_parts.call_match_arm().to_string()
    );
    assert_eq!(false, handler_parts.is_async());
}

#[test]
fn handler_parts_from_works_for_async_func() {
    let signature = syn::parse2::<syn::Signature>(quote! {
        async fn do_this(p1: (u32, u8))
    })
    .unwrap();

    let handler_parts = handler_parts::HandlerParts::from(
        &syn::Ident::new("Commands", proc_macro2::Span::call_site()),
        &syn::Ident::new("CommandResponses", proc_macro2::Span::call_site()),
        &signature,
    );

    assert_eq!(
        quote!(DoThis((u32, u8),),).to_string(),
        handler_parts.request_enum_variant().to_string()
    );
    assert_eq!(
        quote!(DoThis(()),).to_string(),
        handler_parts.response_enum_variant().to_string()
    );
    assert_eq!(
        quote!(
            Commands::DoThis(v0) => {
                let result: Result<_, _> = do_this(v0).await;
                let is_error = result.is_err();
                (CommandResponses::DoThis(result), is_error)
            }
        )
        .to_string(),
        handler_parts.call_match_arm().to_string()
    );
    assert_eq!(true, handler_parts.is_async());
}
