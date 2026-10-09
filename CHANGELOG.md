# Changelog

Notable changes to this project should be documented in this file.
Make sure it is up to date before performing a release.

This project follows the [Keep a Changelog](https://keepachangelog.com/en/2.0.0/) format wherever that is reasonable.

The "title" of each release should be its first line.
A title is required for publishing a github release, so all versions should have one.

Most changes include the relevant [jj](https://jj-vcs.dev) change ids in parens. An example of a change id is wuoxvnsw.

## Unreleased

### Added
- Improve crate-level docs by syncing with `README.md` (qwokxwsr)

### Fixed
- Correct LICENSE file to match README & Cargo.toml (qtssxqyq)
  - Accidentally used the license file from [unicodeit.rs](https://github.com/Techcable/unicodeit.rs/blob/v0.2.1/LICENSE.md) before this.

## 0.1.0 - 2026-04-16
Initial release.

Implementation currently based on the [`expander` crate],
but this may change in the future.

[`expander` crate]: https://github.com/drahnr/expander
