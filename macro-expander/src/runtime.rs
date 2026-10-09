extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

/// Write the output of a proc macro to a file, if debug expansion is enabled for it.
///
/// This is what [`#[debug_expand_macro]`](crate::debug_expand_macro) calls on the result of the function.
///
/// If [`is_macro_debug_enabled`] returns `true` for `macro_name`,
/// `stream` is written to a temporary build file and an `include!` of that file is returned.
/// Compiler errors in the generated code then point to a line in that file
/// rather than somewhere in the macro invocation.
/// This will only happen if the `MACRO_EXPANDER_DEBUG` environment variable enables debug expansion
/// for the macro and the `macro-expander/enable` cargo feature is enabled.
/// See [`#[debug_expand_macro]`](crate::debug_expand_macro#macro_expander_debug) for the
/// environment variable syntax.
///
/// If [`is_macro_debug_enabled`] returns false,
/// the macro `stream` is returned unchanged.
/// This will always happen if the `macro-expander/force-disable` feature is set.
///
/// If `stream` contains a [`compile_error!`], it is returned unchanged and no file is written,
/// as that would discard the span information of the error.
///
/// The file is placed in the build output directory of the `expander` crate,
/// within cargo's `target` directory.
///
/// # Panics
/// Panics if the file cannot be written,
/// or if `MACRO_EXPANDER_DEBUG` is set to a value that is not valid unicode.
///
/// Like any use of [`proc_macro::TokenStream`],
/// this panics if called outside of a procedural macro.
/// Use [`debug_expand_simple2`] if you need that.
pub fn debug_expand_simple(macro_name: &str, stream: TokenStream) -> TokenStream {
    debug_expand_simple2(macro_name, stream.into()).into()
}

/// Same as [`debug_expand_simple`], but for [`proc_macro2::TokenStream`] rather than [`proc_macro::TokenStream`].
///
/// Unlike [`debug_expand_simple`], this can also be called outside of a procedural macro,
/// such as from a unit test.
///
/// # Panics
/// Panics if the file cannot be written,
/// or if `MACRO_EXPANDER_DEBUG` is set to a value that is not valid unicode.
pub fn debug_expand_simple2(macro_name: &str, stream: TokenStream2) -> TokenStream2 {
    // this is not generic as that would involve monomorphization in each downstream crate
    // along with unnecessary code duplication (logic is the same for both stream types)
    // instead we convert input/output to proc_macro2::TokenStream
    //
    // TODO: Add option to support expressions (right now expander always uses semicolon after include)
    if is_macro_debug_enabled(macro_name) {
        let id = format!("{macro_name}-{uid}", uid = rand_uid());
        expander::Expander::new(&id)
            .fmt_full(expander::Channel::default(), expander::Edition::_2021, true)
            .dry(has_compile_error(stream.clone()))
            .write_to_out_dir(stream)
            .unwrap_or_else(|e| panic!("failed to expand {macro_name:?} to file: {e}"))
    } else {
        stream
    }
}

/// Determine if the token stream invokes [`std::compile_error!`].
///
/// If there is a compile error, expanding to a file actually harms debugging
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

/// Check if debug expansion is enabled for the macro named `macro_name`.
///
/// This is used by [`debug_expand_simple`] to decide whether to write its output to a file.
///
/// If the `macro-expander/enable` feature is active,
/// the result is determined by the `MACRO_EXPANDER_DEBUG` environment variable.
/// See [`#[debug_expand_macro]`](crate::debug_expand_macro#macro_expander_debug) macro for its syntax.
/// If the `force-disable` feature is active, this always returns `false`.
///
/// On nightly Rust, the environment variable is read using [`proc_macro::tracked::env_var`]
/// when called from a procedural macro,
/// so changing it causes the crate using the macro to be recompiled.
///
/// [`proc_macro::tracked::env_var`]: https://doc.rust-lang.org/nightly/proc_macro/tracked/fn.env_var.html
///
/// # Panics
/// Panics if `MACRO_EXPANDER_DEBUG` is set to a value that is not valid unicode.
pub fn is_macro_debug_enabled(macro_name: &str) -> bool {
    if cfg!(feature = "force-disable") {
        return false;
    }
    match env_var(CONTROL_ENV_VAR).as_deref() {
        Err(std::env::VarError::NotUnicode(_)) => {
            panic!("Expected environment variable {CONTROL_ENV_VAR:?} to be unicode!")
        }
        Err(std::env::VarError::NotPresent) => false,
        Ok(setting) => is_enabled_by_setting(setting, macro_name),
    }
}

/// Check if the value of `MACRO_EXPANDER_DEBUG` enables debug expansion for the specified macro.
fn is_enabled_by_setting(setting: &str, macro_name: &str) -> bool {
    match setting {
        "*" | "true" => true,
        "false" | "" => false,
        _ => {
            if let Ok(num) = setting.parse::<i64>() {
                num > 0
            } else {
                setting.split(',').any(|part| macro_name == part)
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
    const RADIX: u32 = 36;
    const MAX_LEN: usize = 13;
    let mut chars = Vec::<u8>::new();
    while x != 0 {
        #[allow(clippy::cast_possible_truncation)] // RADIX is small, so modulo cannot overflow
        let digit = (x % (RADIX as u64)) as u32;
        let digit = char::from_digit(digit, RADIX).unwrap();
        assert!(digit.is_ascii_alphanumeric());
        let digit = digit.to_ascii_uppercase();
        chars.push(digit as u8);
        x /= RADIX as u64;
    }
    assert!(chars.len() <= MAX_LEN);
    while chars.len() < MAX_LEN {
        chars.push(b'0');
    }
    chars.reverse();
    String::from_utf8(chars).expect("ascii conversion failed")
}

#[cfg(test)]
mod tests {
    use super::is_enabled_by_setting;

    #[test]
    fn setting_syntax() {
        for (setting, expected) in [
            ("*", true),
            ("true", true),
            ("false", false),
            ("1", true),
            ("0", false),
            ("-1", false),
            ("foo", true),
            ("bar,foo", true),
            ("bar", false),
            ("bar, foo", false),
            ("", false),
            ("True", false),
        ] {
            assert_eq!(is_enabled_by_setting(setting, "foo"), expected, "setting: {setting:?}");
        }
    }
}
