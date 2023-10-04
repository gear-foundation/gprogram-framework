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

mod handler_parts;

pub fn command_handlers_core(tokens: TokenStream2) -> TokenStream2 {
    let handler_signatures = list_handler_signatures(&tokens);

    let request_enum_name = syn::Ident::new("Commands", proc_macro2::Span::call_site());
    let response_enum_name = syn::Ident::new("CommandResponses", proc_macro2::Span::call_site());

    let handler_parts = handler_signatures
        .map(|handler_signature| {
            handler_parts::HandlerParts::from(
                &request_enum_name,
                &response_enum_name,
                &handler_signature,
            )
        })
        .collect::<Vec<_>>();

    if handler_parts.is_empty() {
        abort!(tokens, "No command handlers found");
    }

    let request_enum_variants = handler_parts
        .iter()
        .map(|handler_part| handler_part.request_enum_variant());

    let response_enum_variants = handler_parts
        .iter()
        .map(|handler_part| handler_part.response_enum_variant());

    let call_match_arms = handler_parts
        .iter()
        .map(|handler_part| handler_part.call_match_arm());

    let has_async_handler = handler_parts
        .iter()
        .any(|handler_info| handler_info.is_async());

    let (handle_impl_signature, handle_impl_call, entry_point_signature) = if has_async_handler {
        (
            quote!(async fn handle_impl(command: #request_enum_name) -> (#response_enum_name, bool)),
            quote!(handle_impl(command).await),
            quote!(async fn main_sm(command: Commands)),
        )
    } else {
        (
            quote!(fn handle_impl(command: #request_enum_name) -> (#response_enum_name, bool)),
            quote!(handle_impl(command)),
            quote!(#[no_mangle] extern "C" fn handle()),
        )
    };

    quote!(
        extern crate gstd;
        extern crate gsys;
        extern crate parity_scale_codec;
        extern crate scale_info;

        #[derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)]
        pub enum #request_enum_name {
            #(#request_enum_variants)*
        }

        #[derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)]
        pub enum #response_enum_name {
            #(#response_enum_variants)*
        }

        #[cfg(not(feature = "contract-io"))]
        pub mod wasm {
            use super::*;

            #handle_impl_signature {
                match command {
                    #(#call_match_arms)*
                }
            }

            #entry_point_signature {
                let command = gstd::msg::load::<Commands>().expect("This needs to be handled in a consistent way: input parse error");
                let (result, is_error) = #handle_impl_call;
                let result = result.encode();
                if is_error {
                    unsafe { gsys::gr_panic(result.as_ptr(), result.len() as u32); }
                }
                gstd::msg::reply(result, 0).expect("This needs to be handled in a consistent way: reply error");
            }

            #tokens
        }
    )
}

pub fn query_handlers_core(tokens: TokenStream2) -> TokenStream2 {
    tokens
}

fn list_handler_signatures(tokens: &TokenStream2) -> impl Iterator<Item = syn::Signature> {
    let handlers_mod = syn::parse2::<syn::ItemMod>(quote!(mod __command_handlers {
            #tokens
    }))
    .unwrap_or_else(|err| abort!(err.span(), "Failed to parse handlers: {}", err));
    let handlers_items = handlers_mod
        .content
        .unwrap_or_else(|| abort!(tokens, "No handlers found"))
        .1;
    let handler_signatures = handlers_items.into_iter().filter_map(|item| match item {
        syn::Item::Fn(item_fn) => Some(item_fn.sig),
        _ => None,
    });
    handler_signatures
}

#[cfg(test)]
mod tests;
