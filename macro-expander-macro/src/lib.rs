extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn debug_expand_macro(attr: TokenStream, input: TokenStream) -> TokenStream {
    #[cfg(feature = "enable")]
    {
        let input: proc_macro2::TokenStream = input.into();
        TokenStream::from(
            debug_expand::expand(attr.into(), input.clone()).unwrap_or_else(|error| {
                let error = error.into_compile_error();
                quote::quote! {
                    // still include the original input, hopefully avoiding undefined function errors
                    #input
                    #error
                }
            }),
        )
    }
    #[cfg(not(feature = "enable"))]
    {
        let _ = attr;
        input
    }
}

#[cfg(feature = "enable")]
mod debug_expand {
    use proc_macro2::TokenStream;
    use quote::ToTokens;
    use syn::parse_quote;
    use syn_mid::ItemFn;

    const MACRO_NAME: &str = "debug_expand_macro";
    pub fn expand(attrs: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
        if !attrs.is_empty() {
            return Err(syn::Error::new_spanned(
                attrs,
                format!("#[{MACRO_NAME}] does not accept arguments"),
            ));
        }
        // TODO: Should we consider cfg!(feature = "force-disable")?
        let mut func = syn::parse2::<ItemFn>(input)?;
        // TODO: For derive macros, use trait name instead of fn name
        let func_name = func.sig.ident.to_string();
        let orig_block = func.block.clone();
        func.block = parse_quote!({
            extern crate proc_macro;
            extern crate macro_expander;
            let stream: proc_macro::TokenStream = (|| -> proc_macro::TokenStream  {
                #orig_block
            })();
            macro_expander::debug_expand_simple(
                #func_name,
                stream
            )
        });
        Ok(func.into_token_stream())
    }
}
