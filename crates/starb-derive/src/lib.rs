use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(FromDeriveInput, Default, Debug)]
#[darling(default, attributes(patch))]
struct Atts {
    file: String,
}

#[proc_macro_derive(Patch, attributes(patch))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let atts = Atts::from_derive_input(&input).unwrap();
    let DeriveInput { ident, .. } = input;

    let data = atts.file.as_str();

    let x = quote! {
        include_bytes!(#data)
    };

    quote! {
        impl Patch for #ident {
            fn start(&self) {}
            
            fn enable(&self) {}

            fn disable(&self) {}
        }
    }
    .into()
}
