# Changelog

Notable changes to this project should be documented in this file.
Make sure it is up to date before performing a release.

This project follows the [Keep a Changelog](https://keepachangelog.com/en/2.0.0/) format wherever that is reasonable.

The "title" of each release should be its first line.
A title is required for publishing a github release, so all versions should have one.

Most changes include the relevant [jj](https://jj-vcs.dev) change ids in parens. An example of a change id is wuoxvnsw.

## Unreleased

## v0.1.1 - 2026-10-08
Improve documentation and fix fallback macro.

### Changed
- Increased MSRV to 1.64 (xlonorrl)

### Added
- Accept `true` and `false` as values of `MACRO_EXPANDER_DEBUG` (ptyqomtq)
  - The empty string is also interpreted as `false`
- Improve crate-level docs by syncing with `README.md` (qwokxwsr)
- Add docs to all public functions in `macro-expander` (yxuyxklx)
  - Set `#[deny(missing_docs)]` so we don't forget docs in the future
  - Document the `MACRO_EXPANDER_DEBUG` syntax under `#[debug_expand_macro]`
  - Document that `#[proc_macro]` is only supported for macros expanding to items

### Fixed
- Fix optional dependency feature resolution on Cargo 1.64 (tmvrzrlo)
- Use the correct attribute name `#[debug_expand_macro]` in error messages (qkkruqlv)
- Correct LICENSE file to match README & Cargo.toml (qtssxqyq)
  - Accidentally used the license file from [unicodeit.rs](https://github.com/Techcable/unicodeit.rs/blob/v0.2.1/LICENSE.md) before this.
- Export the `#[debug_expand_macro]` fallback from `macro_expander` (mxnnnszo)
  - This means you can use `#[macro_expander::debug_expand_macro]` even when the `"macro-expander/enable"` feature is not enabled
  - The fallback implementation avoids any dependencies on `syn` or `proc-macro2`, and will just leave your code alone without doing any debug expansion.
  - The fallback implementation still requires the `"macro-expander/macro"` feature to be enabled.

## 0.1.0 - 2026-04-16
Initial release.

Implementation currently based on the [`expander` crate],
but this may change in the future.

[`expander` crate]: https://github.com/drahnr/expander
