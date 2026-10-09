extern crate proc_macro;

use macro_expander::debug_expand_macro;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};

use syn::spanned::Spanned;

#[proc_macro]
#[debug_expand_macro]
pub fn awesome_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);
    if !input.is_empty() {
        return quote_spanned!(input.span() => compile_error!("Cannot handle arguments!!")).into();
    }
    quote! {
        struct Awesome(u32, u32);
        impl std::fmt::Display for Awesome {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("awesome!!")?;
                Ok(())
            }
        }
    }
    .into()
}
