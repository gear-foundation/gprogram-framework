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

//! Supporting functions and structures for the `command_handlers` and `query_handlers` macros.

use convert_case::{Case, Casing};
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort;
use quote::quote;
use syn::{self, spanned::Spanned};

pub(super) struct GeneratedFuncInfo<'a> {
    pub(super) name: &'a syn::Ident,
    pub(super) param_type_name: &'a syn::Ident,
    pub(super) is_async: bool,
}

pub(super) fn generate(
    tokens: &TokenStream2,
    request_enum_name: &str,
    response_enum_name: &str,
    func_name: &str,
    generate_entry_point: impl FnOnce(&GeneratedFuncInfo) -> TokenStream2,
) -> TokenStream2 {
    let request_enum_ident = syn::Ident::new(request_enum_name, proc_macro2::Span::call_site());
    let response_enum_ident = syn::Ident::new(response_enum_name, proc_macro2::Span::call_site());
    let func_ident = syn::Ident::new(func_name, proc_macro2::Span::call_site());

    let full_handler_parts = FullHandlerParts::from(
        &tokens,
        &request_enum_ident,
        &response_enum_ident,
        &func_ident,
    );

    let request_enum = full_handler_parts.request_enum;
    let response_enum = full_handler_parts.response_enum;
    let function = full_handler_parts.function;

    let scale_codec_crate_ident = get_scale_codec_crate_ident(request_enum_name);
    let scale_info_crate_ident = get_scale_info_crate_ident(request_enum_name);
    let module_name = get_handlers_mod_ident(request_enum_name);

    let entry_point = generate_entry_point(&GeneratedFuncInfo {
        param_type_name: &request_enum_ident,
        name: &func_ident,
        is_async: full_handler_parts.is_async,
    });

    quote!(
        extern crate parity_scale_codec as #scale_codec_crate_ident;
        extern crate scale_info as #scale_info_crate_ident;

        #[derive(#scale_codec_crate_ident::Encode, #scale_codec_crate_ident::Decode, #scale_info_crate_ident::TypeInfo)]
        #request_enum

        #[derive(#scale_codec_crate_ident::Encode, #scale_codec_crate_ident::Decode, #scale_info_crate_ident::TypeInfo)]
        #response_enum

        #[cfg(not(feature = "contract-io"))] // TODO: Make this configurable?
        pub mod #module_name {
            use super::*;

            extern crate gstd;
            extern crate gsys;

            #function

            #entry_point

            #tokens
        }
    )
}

struct FullHandlerParts {
    request_enum: TokenStream2,
    response_enum: TokenStream2,
    function: TokenStream2,
    is_async: bool,
}

impl FullHandlerParts {
    fn from(
        tokens: &TokenStream2,
        request_enum_ident: &syn::Ident,
        response_enum_ident: &syn::Ident,
        func_name: &syn::Ident,
    ) -> FullHandlerParts {
        let handlers_signatures = list_handlers_signatures(&tokens);

        let handlers_parts = handlers_signatures
            .map(|handler_signature| {
                SubHandlerParts::from(
                    &request_enum_ident,
                    &response_enum_ident,
                    &handler_signature,
                )
            })
            .collect::<Vec<_>>();

        if handlers_parts.is_empty() {
            abort!(
                tokens,
                "No handlers found. Please either define one or remove the macro usage"
            );
        }

        let request_enum_variants = handlers_parts
            .iter()
            .map(|handler_parts| &handler_parts.request_enum_variant);

        let response_enum_variants = handlers_parts
            .iter()
            .map(|handler_parts| &handler_parts.response_enum_variant);

        let call_match_arms = handlers_parts
            .iter()
            .map(|handler_parts| &handler_parts.call_match_arm);

        let has_async_handler = handlers_parts
            .iter()
            .any(|handler_parts| handler_parts.is_async);

        let fn_signature = if has_async_handler {
            quote!(async fn #func_name(request: #request_enum_ident) -> (#response_enum_ident, bool))
        } else {
            quote!(fn #func_name(request: #request_enum_ident) -> (#response_enum_ident, bool))
        };

        FullHandlerParts {
            request_enum: quote!(
                pub enum #request_enum_ident {
                    #(#request_enum_variants)*
                }
            ),
            response_enum: quote!(
                pub enum #response_enum_ident {
                    #(#response_enum_variants)*
                }
            ),
            function: quote!(
                #fn_signature {
                    match request {
                        #(#call_match_arms)*
                    }
                }
            ),
            is_async: has_async_handler,
        }
    }
}

struct SubHandlerParts {
    request_enum_variant: TokenStream2,
    response_enum_variant: TokenStream2,
    call_match_arm: TokenStream2,
    is_async: bool,
}

impl SubHandlerParts {
    fn from(
        request_enum_ident: &syn::Ident,
        response_enum_ident: &syn::Ident,
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
                #request_enum_ident::#enum_variant_name(#(#call_params),*) => {
                    let result: Result<_, _> = #call;
                    let is_error = result.is_err();
                    (#response_enum_ident::#enum_variant_name(result), is_error)
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

fn list_handlers_signatures(tokens: &TokenStream2) -> impl Iterator<Item = syn::Signature> {
    let handlers_mod = syn::parse2::<syn::ItemMod>(quote!(mod __handlers {
            #tokens
    }))
    .unwrap_or_else(|err| abort!(err.span(), "Failed to parse handlers: {}", err));
    let handlers_items = handlers_mod
        .content
        .unwrap_or_else(|| abort!(tokens, "Handlers must be inlined"))
        .1;
    let handler_signatures = handlers_items.into_iter().filter_map(|item| match item {
        syn::Item::Fn(item_fn) => Some(item_fn.sig),
        _ => None,
    });
    handler_signatures
}

fn get_scale_codec_crate_ident(prefix: &str) -> syn::Ident {
    syn::Ident::new(
        format!("{}_scale_codec", prefix.to_case(Case::Snake)).as_str(),
        proc_macro2::Span::call_site(),
    )
}

fn get_scale_info_crate_ident(prefix: &str) -> syn::Ident {
    syn::Ident::new(
        format!("{}_scale_info", prefix.to_case(Case::Snake)).as_str(),
        proc_macro2::Span::call_site(),
    )
}

fn get_handlers_mod_ident(prefix: &str) -> syn::Ident {
    syn::Ident::new(
        format!("{}_handlers_wasm", prefix.to_case(Case::Snake)).as_str(),
        proc_macro2::Span::call_site(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sub_handler_parts_works_for_func_with_default_return_type() {
        let signature = syn::parse2::<syn::Signature>(quote! {
            fn do_this(p1: u32, p2: String)
        })
        .unwrap();

        let handler_parts = SubHandlerParts::from(
            &syn::Ident::new("Commands", proc_macro2::Span::call_site()),
            &syn::Ident::new("CommandResponses", proc_macro2::Span::call_site()),
            &signature,
        );

        assert_eq!(
            quote!(DoThis(u32, String,),).to_string(),
            handler_parts.request_enum_variant.to_string()
        );
        assert_eq!(
            quote!(DoThis(()),).to_string(),
            handler_parts.response_enum_variant.to_string()
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
            handler_parts.call_match_arm.to_string()
        );
        assert_eq!(false, handler_parts.is_async);
    }

    #[test]
    fn sub_handler_parts_works_for_func_without_args() {
        let signature = syn::parse2::<syn::Signature>(quote! {
            fn do_this()
        })
        .unwrap();

        let handler_parts = SubHandlerParts::from(
            &syn::Ident::new("Commands", proc_macro2::Span::call_site()),
            &syn::Ident::new("CommandResponses", proc_macro2::Span::call_site()),
            &signature,
        );

        assert_eq!(
            quote!(DoThis(),).to_string(),
            handler_parts.request_enum_variant.to_string()
        );
        assert_eq!(
            quote!(DoThis(()),).to_string(),
            handler_parts.response_enum_variant.to_string()
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
            handler_parts.call_match_arm.to_string()
        );
        assert_eq!(false, handler_parts.is_async);
    }

    #[test]
    fn sub_handler_parts_works_for_async_func() {
        let signature = syn::parse2::<syn::Signature>(quote! {
            async fn do_this(p1: (u32, u8))
        })
        .unwrap();

        let handler_parts = SubHandlerParts::from(
            &syn::Ident::new("Commands", proc_macro2::Span::call_site()),
            &syn::Ident::new("CommandResponses", proc_macro2::Span::call_site()),
            &signature,
        );

        assert_eq!(
            quote!(DoThis((u32, u8),),).to_string(),
            handler_parts.request_enum_variant.to_string()
        );
        assert_eq!(
            quote!(DoThis(()),).to_string(),
            handler_parts.response_enum_variant.to_string()
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
            handler_parts.call_match_arm.to_string()
        );
        assert_eq!(true, handler_parts.is_async);
    }
}
