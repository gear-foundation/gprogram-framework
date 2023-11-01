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

//! Supporting functions and structures for the `gprogram` macro.

use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use syn::{self, spanned::Spanned};

static mut DISCOVERED_COMMAND_HANDLERS_BLOCK_HASH: Option<u64> = None;
static mut DISCOVERED_QUERY_HANDLERS_BLOCK_HASH: Option<u64> = None;
// This is a hash of the handlers block macro which will be executed last
static mut FINALIZING_HANDLERS_BLOCK_HASH: Option<u64> = None;

pub(super) fn discover_handlers_blocks(
    gprogram_mod_content: &Vec<syn::Item>,
    command_handlers_macro_name: &str,
    query_handlers_macro_name: &str,
) {
    for item in gprogram_mod_content.iter() {
        if let syn::Item::Macro(syn::ItemMacro {
            mac:
                syn::Macro {
                    path: item_macro_path,
                    tokens: item_macro_tokens,
                    ..
                },
            ..
        }) = item
        {
            let item_macro_ident = item_macro_path.get_ident();
            if item_macro_ident.is_none() {
                continue;
            }
            let item_macro_name = item_macro_ident.unwrap().to_string();
            let handlers_block_hash_ref = if item_macro_name == command_handlers_macro_name {
                command_handlers_block_hash_ref()
            } else if item_macro_name == query_handlers_macro_name {
                query_handlers_block_hash_ref()
            } else {
                continue;
            };
            if handlers_block_hash_ref.is_some() {
                abort!(
                    item.span(),
                    "There must be a single `{}` block in the `gprogram` module",
                    item_macro_name
                )
            }
            let handlers_block_hash = hash_of(&item_macro_tokens);
            handlers_block_hash_ref.replace(handlers_block_hash);
            if finishing_handlers_block_hash_ref().is_none() {
                finishing_handlers_block_hash_ref().replace(handlers_block_hash);
            }
        }
    }
}

pub(super) fn is_handlers_block_discovered(handlers_block_tokens: &TokenStream2) -> bool {
    let handlers_block_hash = hash_of(handlers_block_tokens);
    &Some(handlers_block_hash) == command_handlers_block_hash_ref()
        || &Some(handlers_block_hash) == query_handlers_block_hash_ref()
}

pub(super) fn __is_handlers_block_finalizing(handlers_block_tokens: &TokenStream2) -> bool {
    let handlers_block_hash = hash_of(handlers_block_tokens);
    &Some(handlers_block_hash) == finishing_handlers_block_hash_ref()
}

fn command_handlers_block_hash_ref() -> &'static mut Option<u64> {
    unsafe { &mut DISCOVERED_COMMAND_HANDLERS_BLOCK_HASH }
}

fn query_handlers_block_hash_ref() -> &'static mut Option<u64> {
    unsafe { &mut DISCOVERED_QUERY_HANDLERS_BLOCK_HASH }
}

fn finishing_handlers_block_hash_ref() -> &'static mut Option<u64> {
    unsafe { &mut FINALIZING_HANDLERS_BLOCK_HASH }
}

fn hash_of(tokens: &TokenStream2) -> u64 {
    let mut hasher = DefaultHasher::new();
    tokens.to_string().hash(&mut hasher);
    hasher.finish()
}
