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

use gprogram_framework_macros_core::{command_handlers_core, query_handlers_core};
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro]
pub fn command_handlers(tokens: TokenStream) -> TokenStream {
    command_handlers_core(tokens.into()).into()
}

#[proc_macro_error]
#[proc_macro]
pub fn query_handlers(tokens: TokenStream) -> TokenStream {
    query_handlers_core(tokens.into()).into()
}
