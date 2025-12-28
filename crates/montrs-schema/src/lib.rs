//! montrs-schema: Procedural macros for schema validation in MontRS.
//! This crate provides the `#[derive(Schema)]` macro which generates
//! compile-time validation logic for structs based on field attributes.

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, LitInt};

/// Procedural macro to derive validation logic for a struct.
/// Supported attributes:
/// - `#[schema(min_len = N)]`: Validates that a string has at least N characters.
/// - `#[schema(email)]`: Basic check for the presence of an '@' character.
/// - `#[schema(regex = "pattern")]`: Placeholder for regex-based validation.
/// - `#[schema(custom = "fn_name")]`: Calls a custom validation method on the struct.
#[proc_macro_derive(Schema, attributes(schema))]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let mut all_field_validations = Vec::new();

    // Parse the struct data and iterate over named fields.
    if let Data::Struct(data) = input.data {
        if let Fields::Named(fields) = data.fields {
            for f in fields.named {
                let field_name = f.ident.clone();
                // Iterate over attributes on each field.
                for attr in f.attrs {
                    if attr.path().is_ident("schema") {
                        // Use syn's nested meta parsing for schema attributes.
                        let _ = attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("min_len") {
                                // Extract integer value for min_len.
                                let value = meta.value()?;
                                let lit: LitInt = value.parse()?;
                                let min = lit.base10_parse::<usize>()?;
                                all_field_validations.push(quote! {
                                    if self.#field_name.len() < #min {
                                        return Err(format!("{} is too short (min {})", stringify!(#field_name), #min));
                                    }
                                });
                            } else if meta.path.is_ident("email") {
                                // Basic email presence check.
                                all_field_validations.push(quote! {
                                    if !self.#field_name.contains('@') {
                                        return Err(format!("{} must be a valid email", stringify!(#field_name)));
                                    }
                                });
                            } else if meta.path.is_ident("regex") {
                                        // Regex pattern extraction.
                                        let value = meta.value()?;
                                        let lit: syn::LitStr = value.parse()?;
                                        let regex_str = lit.value();
                                        all_field_validations.push(quote! {
                                            if !#regex_str.is_empty() && !self.#field_name.is_empty() {
                                                // TODO: Integration with regex crate for v0.2
                                            }
                                        });
                                    } else if meta.path.is_ident("custom") {
                                        // Custom function name extraction.
                                        let value = meta.value()?;
                                        let lit: syn::LitStr = value.parse()?;
                                        let custom_fn = syn::Ident::new(&lit.value(), lit.span());
                                        all_field_validations.push(quote! {
                                            if let Err(e) = self.#custom_fn() {
                                                return Err(format!("{}: {}", stringify!(#field_name), e));
                                            }
                                        });
                                    }
                            Ok(())
                        });
                    }
                }
            }
        }
    }

    // Generate the implementation of the validate() method.
    let expanded = quote! {
        impl #name {
            pub fn validate(&self) -> Result<(), String> {
                #(#all_field_validations)*
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}
