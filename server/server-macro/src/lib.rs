use proc_macro::TokenStream;

use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Expr, Token};

mod to {
    use syn::custom_keyword;

    custom_keyword!(to);
}

struct Args {
    layers: Punctuated<Expr, Token![,]>,
    handler: Expr,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut layers = Punctuated::new();

        loop {
            layers.push_value(input.parse()?);

            if let Ok(punct) = input.parse() {
                layers.push_punct(punct);
            } else {
                input.parse::<to::to>().expect("to is expected");
                break;
            }
        }

        let handler = input.parse()?;

        Ok(Args { layers, handler })
    }
}

impl ToTokens for Args {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let last_layer = self.layers.last().unwrap();
        let handler = &self.handler;
        let mut ret =
            quote!( cubby_connect_server_core::layer::connect( #last_layer, #handler ).await? );

        for i in self.layers.iter().rev().skip(1) {
            ret = quote!( cubby_connect_server_core::layer::connect( #i, #ret ).await? );
        }

        ret.to_tokens(tokens);
    }
}

#[proc_macro]
pub fn apply(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Args);
    quote!( #args ).into()
}

#[allow(dead_code)]
mod compile_fail_test {
    /// ```compile_fail
    /// apply!()
    /// ```
    fn no_argument() {}
}
