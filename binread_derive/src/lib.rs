#![warn(rust_2018_idioms)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input,
    DeriveInput
};

mod codegen;
mod meta_attrs;
mod binread_endian;

use codegen::sanitization::*;
use meta_attrs::FieldLevelAttrs;
use proc_macro2::TokenStream as TokenStream2;

fn generate_impl(input: &DeriveInput) -> TokenStream {
    let codegen::GeneratedCode { read_opt_impl, arg_type } = codegen::generate(&input).unwrap_or_else(|e| {
        // If there is a parsing error, a BinRead impl still needs to be
        // generated to avoid misleading errors at all call sites that use the
        // BinRead trait
        codegen::GeneratedCode {
            arg_type: quote! { () },
            read_opt_impl: e.to_compile_error(),
        }
    });

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    quote!(
        #[allow(non_snake_case)]
        impl #impl_generics #TRAIT_NAME for #name #ty_generics #where_clause {
            type Args = #arg_type;

            fn read_options<R: #READ_TRAIT + #SEEK_TRAIT>
                (#READER: &mut R, #OPT: &#OPTIONS, #ARGS: Self::Args)
                -> #BIN_RESULT<Self>
            {
                #read_opt_impl
            }
        }
    ).into()
}

#[proc_macro_derive(BinRead, attributes(binread, br))]
pub fn derive_binread_trait(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    generate_impl(&input)
}

fn is_temp(field: &syn::Field) -> bool {
    FieldLevelAttrs::try_from_attrs(&field.attrs)
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
    let generated_impl = TokenStream2::from(generate_impl(&input));

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
        syn::Data::Union(ref mut union) => {
            for field in union.fields.named.iter_mut() {
                field.attrs.retain(is_not_binread_attr);
            }
        },
    }

    input.attrs.retain(is_not_binread_attr);

    quote!(
        #input
        #generated_impl
    ).into()
}
