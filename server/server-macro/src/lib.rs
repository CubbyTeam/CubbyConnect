//! This is a collection of macros that is used in server
//!
//! - apply: this would

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
        let mut layers: Punctuated<Expr, Token![,]> = Punctuated::new();

        loop {
            layers.push_value(input.parse()?);

            if let Ok(punct) = input.parse() {
                layers.push_punct(punct);
            } else {
                input.parse::<to::to>()?;
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

/// Macro to connect layers and handler to one handler
///
/// This would use `cubby_connect_server_core::layer::connect` in the inside (when expansion).
///
/// # Examples
///
/// ```ignore
/// let handler = apply!(some_layer_1, some_layer_2, ..., some_layer_n to some_handler);
/// ```
///
/// ```
/// use cubby_connect_server_core::apply;
/// use cubby_connect_server_core::handler::Handler;
/// use std::fmt::Display;
///
/// async fn echo<T>(t: T) -> Result<T, ()> {
///     Ok(t)
/// }
///
/// async fn print<T: Display>(t: T) -> Result<(), ()> {
///     assert_eq!(t.to_string(), "Hello, World!");
///     print!("{t}");
///     Ok(())
/// }
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), ()> {
/// let e = apply!(echo to print);
/// e.call("Hello, World!").await?;
/// # Ok(())
/// # }
/// ```
#[proc_macro]
pub fn apply(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Args);
    quote!( #args ).into()
}

#[allow(dead_code)]
mod compile_fail_test {
    /// apply cannot be empty
    ///
    /// error: unexpected end of input, expected expression
    ///
    /// ```compile_error
    /// use cubby_connect_server_macro::apply;
    ///
    /// apply!()
    /// ```
    fn no_argument() {}

    /// apply should have a `to`
    ///
    /// error: unexpected end of input, expected `to`
    ///
    /// ```compile_error
    /// use cubby_connect_server_macro::apply;
    ///
    /// apply!(hello, world)
    /// ```
    fn no_to() {}
}
