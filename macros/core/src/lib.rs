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

use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort;
use quote::quote;
use syn;

mod gprogram;
mod handlers;

const COMMANDS_ENUM_NAME: &str = "Commands";
const COMMAND_RESPONSES_ENUM_NAME: &str = "CommandResponses";
const COMMAND_FN_NAME: &str = "handle_impl";
const COMMAND_FN_PARAM_NAME: &str = "command";
const QUERIES_ENUM_NAME: &str = "Queries";
const QUERY_RESPONSES_ENUM_NAME: &str = "QueryResponses";
const QUERY_FN_NAME: &str = "query_impl";
const QUERY_FN_PARAM_NAME: &str = "query";

pub fn gprogram_core(
    _attrs: TokenStream2,
    tokens: TokenStream2,
    command_handlers_macro_name: &str,
    query_handlers_macro_name: &str,
) -> TokenStream2 {
    let gprogram_mod = syn::parse2::<syn::ItemMod>(tokens.clone()).unwrap_or_else(|err| {
        abort!(
            err.span(),
            "Failed to parse module attributed with `gprogram`: {}",
            err
        )
    });

    let gprogram_mod_content = gprogram_mod
        .content
        .unwrap_or_else(|| abort!(tokens, "Module attributed with `gprogram` must be inline"))
        .1;

    gprogram::discover_handlers_block(
        &gprogram_mod_content,
        command_handlers_macro_name,
        query_handlers_macro_name,
    );

    tokens
}

pub fn gprogram_command_handlers_core(tokens: TokenStream2) -> TokenStream2 {
    if !gprogram::is_handlers_block_discovered(&tokens) {
        abort!(
            tokens,
            "The `command_handlers` block must be on the top level of `gprogram` module"
        );
    }
    command_handlers_core(tokens)
}

pub fn gprogram_query_handlers_core(tokens: TokenStream2) -> TokenStream2 {
    if !gprogram::is_handlers_block_discovered(&tokens) {
        abort!(
            tokens,
            "The `query_handlers` block must be on the top level of `gprogram` module"
        );
    }
    query_handlers_core(tokens)
}

fn command_handlers_core(tokens: TokenStream2) -> TokenStream2 {
    let fn_param_name = syn::Ident::new(COMMAND_FN_PARAM_NAME, proc_macro2::Span::call_site());

    handlers::generate(
        &tokens,
        COMMANDS_ENUM_NAME,
        COMMAND_RESPONSES_ENUM_NAME,
        COMMAND_FN_NAME,
        |context| {
            let request_enum_name = context.param_type_name;
            let fn_name = context.name;
            let (fn_call, entry_point_signature) = if context.is_async {
                // quote!(#fn_name(#fn_param_name).await),
                abort!(
                    tokens,
                    "Async command handlers are not implemented. Please use sync ones"
                )
            } else {
                (
                    quote!(#fn_name(#fn_param_name)),
                    quote!(#[no_mangle] extern "C" fn handle()),
                )
            };
            quote!(
                #entry_point_signature {
                    let #fn_param_name = gstd::msg::load::<#request_enum_name>().expect("This needs to be handled in a consistent way: input parse error");
                    let (result, is_error) = #fn_call;
                    let result = result.encode();
                    if is_error {
                        unsafe { gsys::gr_panic(result.as_ptr(), result.len() as u32); }
                    }
                    gstd::msg::reply(result, 0).expect("This needs to be handled in a consistent way: reply error");
                }
            )
        },
    )
}

fn query_handlers_core(tokens: TokenStream2) -> TokenStream2 {
    let fn_param_name = syn::Ident::new(QUERY_FN_PARAM_NAME, proc_macro2::Span::call_site());

    handlers::generate(
        &tokens,
        QUERIES_ENUM_NAME,
        QUERY_RESPONSES_ENUM_NAME,
        QUERY_FN_NAME,
        |context| {
            let request_enum_name = context.param_type_name;
            let fn_name = context.name;
            let (fn_call, entry_point_signature) = if context.is_async {
                abort!(
                    tokens,
                    "Async query handlers are not supported. Please use sync ones"
                );
            } else {
                (
                    quote!(#fn_name(#fn_param_name)),
                    quote!(#[no_mangle] extern "C" fn state()),
                )
            };
            quote!(
                #entry_point_signature {
                    let #fn_param_name = gstd::msg::load::<#request_enum_name>().expect("This needs to be handled in a consistent way: input parse error");
                    let (result, _) = #fn_call;
                    let result = result.encode();
                    gstd::msg::reply(result, 0).expect("This needs to be handled in a consistent way: reply error");
                }
            )
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn command_handlers_core_works() {
        let input = quote! {
            fn do_this() {}
        };
        let expected = quote! {
            extern crate parity_scale_codec as commands_scale_codec;
            extern crate scale_info as commands_scale_info;

            #[derive(commands_scale_codec::Encode, commands_scale_codec::Decode, commands_scale_info::TypeInfo)]
            pub enum Commands {
                DoThis(),
            }

            #[derive(commands_scale_codec::Encode, commands_scale_codec::Decode, commands_scale_info::TypeInfo)]
            pub enum CommandResponses {
                DoThis(()),
            }

            #[cfg(not(feature = "contract-io"))]
            pub mod commands_handlers_wasm {
                use super::*;

                extern crate gstd;
                extern crate gsys;

                fn handle_impl(request: Commands) -> (CommandResponses, bool) {
                    match request {
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
    fn query_handlers_core_works() {
        let input = quote! {
            fn this() {}
        };
        let expected = quote! {
            extern crate parity_scale_codec as queries_scale_codec;
            extern crate scale_info as queries_scale_info;

            #[derive(queries_scale_codec::Encode, queries_scale_codec::Decode, queries_scale_info::TypeInfo)]
            pub enum Queries {
                This(),
            }

            #[derive(queries_scale_codec::Encode, queries_scale_codec::Decode, queries_scale_info::TypeInfo)]
            pub enum QueryResponses {
                This(()),
            }

            #[cfg(not(feature = "contract-io"))]
            pub mod queries_handlers_wasm {
                use super::*;

                extern crate gstd;
                extern crate gsys;

                fn query_impl(request: Queries) -> (QueryResponses, bool) {
                    match request {
                        Queries::This() => {
                            let result: Result<_, _> = this();
                            let is_error = result.is_err();
                            (QueryResponses::This(result), is_error)
                        }
                    }
                }

                #[no_mangle]
                extern "C" fn state() {
                    let query = gstd::msg::load::<Queries>().expect("This needs to be handled in a consistent way: input parse error");
                    let (result, _) = query_impl(query);
                    let result = result.encode();
                    gstd::msg::reply(result, 0) .expect("This needs to be handled in a consistent way: reply error");
                }

                fn this() {}
            }
        };
        assert_eq!(expected.to_string(), query_handlers_core(input).to_string());
    }
}
