extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input,
    DeriveInput
};

mod codegen;
mod meta_attrs;
mod binread_endian;
mod compiler_error;

use codegen::sanitization::*;
use meta_attrs::FieldLevelAttrs;
use proc_macro2::TokenStream as TokenStream2;
use compiler_error::{CompileError, SpanError};

fn generate_derive(input: DeriveInput, code: codegen::GeneratedCode) -> TokenStream {
    let codegen::GeneratedCode {
        read_opt_impl, after_parse_impl, arg_type
    } = code;

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    quote!(
        #[allow(warnings)]
        impl #impl_generics #TRAIT_NAME for #name #ty_generics #where_clause {
            type Args = #arg_type;

            fn read_options<R: #READ_TRAIT + #SEEK_TRAIT>
                (#READER: &mut R, #OPT: &#OPTIONS, #ARGS: Self::Args)
                -> #BIN_RESULT<Self>
            {
                #read_opt_impl
            }

            fn after_parse<R: #READ_TRAIT + #SEEK_TRAIT> (&mut self, #READER: &mut R,
                #OPT : &#OPTIONS, #ARGS : Self::Args) 
                -> #BIN_RESULT<()>
            {
                #after_parse_impl
            }
        }
    ).into()
}

#[proc_macro_derive(BinRead, attributes(binread, br))]
pub fn derive_binread_trait(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match codegen::generate(&input) {
        Ok(code) => {
            generate_derive(input, code)
        }
        Err(err) => {
            let error = match err {
                CompileError::SpanError(span_err) => {
                    let SpanError (span, error) = span_err;
                    let error: &str = &error;
                    quote_spanned!{ span =>
                        compile_error!(#error)
                    }
                }
                CompileError::Syn(syn_err) => syn_err.to_compile_error()
                
            };
            generate_derive(input, codegen::GeneratedCode::new(
                quote!(todo!()),
                error,
                quote!(())
            ))
        }
    }
}

fn is_temp(field: &syn::Field) -> bool {
    FieldLevelAttrs::from_field(field)
        .map(|attrs| attrs.temp)
        .unwrap_or(false)
}

fn is_not_binread_attr(attr: &syn::Attribute) -> bool {
    attr.path.get_ident().map(|ident| ident != "br" && ident != "binread").unwrap_or(true)
}

fn remove_br_attrs(fields: &mut syn::punctuated::Punctuated<syn::Field, syn::Token![,]>) {
    *fields = fields
        .clone()
        .into_pairs()
        .filter(|x| !is_temp(x.value()))
        .map(|mut field|{
            field.value_mut().attrs.retain(is_not_binread_attr);
            field
        })
        .collect()
}

fn remove_field_attrs(fields: &mut syn::Fields) {
    match fields {
        syn::Fields::Named(ref mut fields) => remove_br_attrs(&mut fields.named),
        syn::Fields::Unnamed(ref mut fields) => remove_br_attrs(&mut fields.unnamed),
        syn::Fields::Unit => ()
    }
}

#[proc_macro_attribute]
pub fn derive_binread(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    
    let derive: TokenStream2 = match codegen::generate(&input) {
        Ok(code) => {
            generate_derive(input.clone(), code)
        }
        Err(err) => {
            let error = match err {
                CompileError::SpanError(span_err) => {
                    let SpanError (span, error) = span_err;
                    let error: &str = &error;
                    quote_spanned!{ span =>
                        compile_error!(#error)
                    }
                }
                CompileError::Syn(syn_err) => syn_err.to_compile_error()
                
            };
            generate_derive(input.clone(), codegen::GeneratedCode::new(
                quote!(todo!()),
                error,
                quote!(())
            ))
        }
    }.into();

    match input.data {
        syn::Data::Struct(ref mut input_struct) => {
            input.attrs.retain(is_not_binread_attr);
            remove_field_attrs(&mut input_struct.fields)
        },
        syn::Data::Enum(ref mut input_enum) => {
            for variant in input_enum.variants.iter_mut() {
                variant.attrs.retain(is_not_binread_attr);
                remove_field_attrs(&mut variant.fields)
            }
        },
        _ => todo!("unions are not supported")
    }

    input.attrs.retain(is_not_binread_attr);

    quote!(
        #input

        #derive
    ).into()
}
