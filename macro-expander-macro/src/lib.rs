//! Proc macro for the `macro-expander` crate.
//!
//! Use this through the re-export `macro_expander::debug_expand_macro`
//! rather than depending on this crate directly.

#![deny(missing_docs)]

extern crate proc_macro;

use proc_macro::TokenStream;

/// Write the output of a proc macro to a file, so errors point to a precise location.
///
/// Apply this to the function implementing a proc macro,
/// below the `#[proc_macro]`, `#[proc_macro_derive]`, or `#[proc_macro_attribute]` attribute:
///
/// ```ignore
/// #[proc_macro_derive(Visit)]
/// #[macro_expander::debug_expand_macro]
/// fn derive_visit(input: TokenStream) -> TokenStream {
///    unimplemented!("your code here")
/// }
/// ```
///
/// The function must return a [`proc_macro::TokenStream`].
/// The attribute does not accept any arguments.
///
/// Function-like macros (`#[proc_macro]`) are currently only supported if they expand to items,
/// such as structs, impls, or functions.
/// When debug expansion is enabled, macros expanding to an expression, a type, or statements
/// fail to compile.
///
/// When debug expansion is enabled for the macro,
/// its output is written to a file and replaced by an `include!` of that file.
/// Compiler errors in the generated code then point to a line in that file
/// rather than to the macro invocation.
///
/// If the output contains a [`compile_error!`], it is left as-is and no file is written,
/// as that would discard the span information of the error.
///
/// # Features
/// The attribute only does anything if the `enable` feature of `macro-expander` is active.
/// Without it, the function is passed through unchanged
/// and the attribute has no dependency on `syn`.
///
/// If the `force-disable` feature is active,
/// debug expansion never happens, regardless of the environment variable.
///
/// # `MACRO_EXPANDER_DEBUG`
/// Whether debug expansion happens is controlled by the `MACRO_EXPANDER_DEBUG` environment variable,
/// which is read when the macro is invoked (when the crate *using* the macro is compiled):
///
/// | Value                   | Effect                                                      |
/// |-------------------------|-------------------------------------------------------------|
/// | unset                   | Disabled.                                                   |
/// | `*`                     | Enabled for all macros.                                     |
/// | an integer, e.g. `1`    | Enabled for all macros if positive, disabled otherwise.     |
/// | `name1,name2,...`       | Enabled only for the macros with the listed names.          |
///
/// The name of a macro is the name of the function this attribute is applied to
/// (`derive_visit` in the example above), not the name of the derived trait or invoked macro.
/// Names must match exactly; whitespace around the commas is not trimmed.
/// Any other value, such as `true` or the empty string,
/// is interpreted as a list of names and so usually disables expansion.
/// A value that is not valid unicode causes a panic.
///
/// On stable Rust, changing the variable may not cause the crate using the macro to be recompiled.
/// Touch a source file of that crate or run `cargo clean` to force the macro to run again.
///
/// The expanded files are placed in the build output directory of the `expander` crate,
/// within cargo's `target` directory.
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
