extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, TokenStream as TokenStream2, TokenTree};

pub fn debug_expand_simple(macro_name: &str, stream: TokenStream) -> TokenStream {
    debug_expand_simple2(macro_name, stream.into()).into()
}

pub fn debug_expand_simple2(macro_name: &str, stream: TokenStream2) -> TokenStream2 {
    // this is not generic as that would involve monomorphization in each downstream crate
    // along with unnecessary code duplication (logic is the same for both stream times)
    // instead we convert input/output to proc_macro2::TokenStream
    //
    // TODO: Add option to support expressions (right now expander always uses semicolon after include)
    if is_macro_debug_enabled(macro_name) {
        let id = format!("{macro_name}-{uid}", uid = rand_uid());
        let tokens = expander::Expander::new(&id)
            .fmt_full(expander::Channel::default(), expander::Edition::_2021, true)
            .dry(has_compile_error(stream.clone()))
            .write_to_out_dir(stream)
            .unwrap_or_else(|e| panic!("failed to expand {macro_name:?} to file: {e}"));
        let mut tokens = tokens.into_iter().collect::<Vec<_>>();
        let semi = tokens.pop().unwrap();
        assert!(matches!(semi, TokenTree::Punct(punct) if punct.as_char() == ';'));
        let group = tokens.pop().unwrap();
        let TokenTree::Group(group) = group else {
            panic!("{group:?}")
        };
        assert_eq!(group.delimiter(), Delimiter::Parenthesis);
        let inner = group.stream();
        tokens.extend(quote::quote!({ #inner }));
        tokens.into_iter().collect()
    } else {
        stream
    }
}

/// Determine if the token stream invokes [`std::compile_error!`].
///
/// If there is a compile error, expanding to a file is in actually harms debugging
/// by discarding span information.
///
/// TODO: Should we override this?
fn has_compile_error(x: TokenStream2) -> bool {
    use proc_macro2::TokenTree;
    let mut iter = x.into_iter().peekable();
    while let Some(item) = iter.next() {
        match item {
            TokenTree::Ident(name) => {
                if name == "compile_error" && matches!(iter.peek(), Some(TokenTree::Punct(p)) if p.as_char() == '!') {
                    return true;
                }
            }
            TokenTree::Group(ref group) => {
                if has_compile_error(group.stream()) {
                    return true;
                }
            }
            // neither a compile_error!, nor something nested
            TokenTree::Punct(_) | TokenTree::Literal(_) => {}
        }
    }
    false
}

const CONTROL_ENV_VAR: &str = "MACRO_EXPANDER_DEBUG";

pub fn is_macro_debug_enabled(macro_name: &str) -> bool {
    if cfg!(feature = "force-disable") {
        return false;
    }
    match env_var(CONTROL_ENV_VAR).as_deref() {
        Err(std::env::VarError::NotUnicode(_)) => {
            panic!("Expected environment variable {CONTROL_ENV_VAR:?} to be unicode!")
        }
        Err(std::env::VarError::NotPresent) => false,
        Ok("*") => true,
        Ok(s) => {
            if let Ok(num) = s.parse::<i64>() {
                num > 0
            } else {
                s.split(",").any(|part| macro_name == part)
            }
        }
    }
}

#[allow(dead_code)] // somewhat by design
pub fn env_var<K: AsRef<str>>(var_name: K) -> Result<String, std::env::VarError> {
    #[cfg(has_tracked_env_var)]
    {
        #[allow(clippy::incompatible_msrv)] // support for proc_macro::tracked implies rust > 1.56
        if proc_macro::is_available() {
            proc_macro::tracked::env_var(var_name.as_ref())
        } else {
            std::env::var(var_name.as_ref())
        }
    }
    #[cfg(not(has_tracked_env_var))]
    {
        std::env::var(var_name.as_ref())
    }
}

fn rand_uid() -> String {
    use nanorand::Rng;
    base36(nanorand::tls_rng().generate())
}
fn base36(mut x: u64) -> String {
    const RADIX: u64 = 36;
    const MAX_LEN: usize = 13;
    let mut chars = Vec::<u8>::new();
    while x != 0 {
        let digit = (x % RADIX) as u32;
        let digit = char::from_digit(digit, RADIX as u32).unwrap();
        assert!(digit.is_ascii_alphanumeric());
        let digit = digit.to_ascii_uppercase();
        chars.push(digit as u8);
        x /= RADIX;
    }
    assert!(chars.len() <= MAX_LEN);
    while chars.len() < MAX_LEN {
        chars.push(b'0');
    }
    chars.reverse();
    String::from_utf8(chars).expect("ascii conversion failed")
}
