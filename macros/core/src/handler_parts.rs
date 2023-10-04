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

use convert_case::{Case, Casing};
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort;
use quote::quote;
use syn::{self, spanned::Spanned};

pub(super) struct HandlerParts {
    request_enum_variant: TokenStream2,
    response_enum_variant: TokenStream2,
    call_match_arm: TokenStream2,
    is_async: bool,
}

impl HandlerParts {
    pub(super) fn request_enum_variant(&self) -> &TokenStream2 {
        &self.request_enum_variant
    }

    pub(super) fn response_enum_variant(&self) -> &TokenStream2 {
        &self.response_enum_variant
    }

    pub(super) fn call_match_arm(&self) -> &TokenStream2 {
        &self.call_match_arm
    }

    pub(super) fn is_async(&self) -> bool {
        self.is_async
    }

    pub(super) fn from(
        request_enum_name: &syn::Ident,
        response_enum_name: &syn::Ident,
        handler_signature: &syn::Signature,
    ) -> Self {
        let enum_variant_name = syn::Ident::new(
            &handler_signature.ident.to_string().to_case(Case::Pascal),
            proc_macro2::Span::call_site(),
        );

        let response_enum_variant = {
            let response_type = Self::response_type(handler_signature);
            quote!(
                #enum_variant_name(#response_type),
            )
        };

        let (arg_types, arg_types_count) = Self::arg_types(handler_signature);

        let request_enum_variant = {
            // Check later if this works.
            // quote!(
            //      #enum_variant_name(#(#arg_types),*),
            // )
            let request_enum_variant_params = arg_types.map(|arg_type| quote!(#arg_type,));
            quote!(
                #enum_variant_name(#(#request_enum_variant_params)*),
            )
        };

        let call_match_arm = {
            let call_params = (0..arg_types_count)
                .map(|idx| syn::Ident::new(&format!("v{}", idx), proc_macro2::Span::call_site()))
                //.map(|param_ident| quote!(#param_ident,))
                .collect::<Vec<_>>();
            let call_name = &handler_signature.ident;
            let call = if handler_signature.asyncness.is_some() {
                quote!(#call_name(#(#call_params),*).await)
            } else {
                quote!(#call_name(#(#call_params),*))
            };
            quote!(
                #request_enum_name::#enum_variant_name(#(#call_params),*) => {
                    let result: Result<_, _> = #call;
                    let is_error = result.is_err();
                    (#response_enum_name::#enum_variant_name(result), is_error)
                }
            )
        };

        Self {
            request_enum_variant,
            response_enum_variant,
            call_match_arm,
            is_async: handler_signature.asyncness.is_some(),
        }
    }

    fn arg_types(
        handler_signature: &syn::Signature,
    ) -> (impl Iterator<Item = &syn::Type> + '_, usize) {
        (
            handler_signature.inputs.iter().map(Self::arg_type),
            handler_signature.inputs.len(),
        )
    }

    fn response_type(handler_signature: &syn::Signature) -> syn::Type {
        Self::return_type(&handler_signature.output)
    }

    fn arg_type(arg: &syn::FnArg) -> &syn::Type {
        if let syn::FnArg::Typed(arg) = arg {
            arg.ty.as_ref()
        } else {
            abort!(arg.span(), "Arguments of the Self type are not supported");
        }
    }

    fn return_type(output: &syn::ReturnType) -> syn::Type {
        if let syn::ReturnType::Type(_, ty) = output {
            ty.as_ref().clone()
        } else {
            syn::parse2::<syn::Type>(quote!(()))
                .unwrap_or_else(|err| abort!(err.span(), "Failed to parse return type: {}", err))
        }
    }
}
