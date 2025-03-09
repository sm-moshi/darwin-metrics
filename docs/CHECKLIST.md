# Crate Release Checklist

## First published release

* Add Rustdoc root to the crate root

        #![doc(html_root_url = "https://docs.rs/<crate>/<version>")]

  (where `<crate>` is the crate name and `<version>` is the
  version name)
* Review the Rust API Guidelines
  [checklist](https://rust-lang.github.io/api-guidelines/checklist.html)
* Remove any `publish=false` from `Cargo.toml`
* Add `version-sync = "0.9"` to `dev-dependencies` and install the
  `version-sync.rs` test in `tests/`
* Set up CI
* Add maintenance badge to `Cargo.toml`

        [badges.maintenance]
        status = "actively-developed"

* Deal with the badge display mess. Add some badges to
  `README.tpl` or `README.md`. If `README.tpl`, do not use
  `cargo-readme`'s badge feature, which is currently kind of
  busted.

        ![Maintenance](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)
        [![CI](https://github.com/GenericPerson/{{crate}}/actions/workflows/main.yml/badge.svg)](https://github.com/GenericPerson/{{crate}}/actions)
        [![crates-io](https://img.shields.io/crates/v/{{crate}}.svg)](https://crates.io/crates/{{crate}})
        [![api-docs](https://docs.rs/{{crate}}/badge.svg)](https://docs.rs/{{crate}})

  If using `README.md` directly, replace all instances of
  `{{crate}}` with the crate name. If you are not
  `GenericPerson`, fix that too. In general, edit to taste.

* Continue with the per-release instructions.

## Every release

* Update `Cargo.toml` version
* Update `html_root_url` version in crate root
* Update `README` version
  * If using `cargo-readme`, run it
  * Otherwise update manually
* Run `grip README.md` to make sure it looks OK
* Run `cargo doc --open` and check that everything
  looks sane
* Grep for the old version number to see if anything
  has been left lying around.
* Run tests to check `version-sync`
* Run `cargo +nightly fmt --all` to check
* Run `cargo clippy --all` to check
* Git commit the new version
* Git tag the new version
* Push the release and make sure it looks OK on Github
* Wait for Github CI to finish
* Publish to `crates.io` with `--dry-run` and make sure it works
* Publish to `crates.io`
* Wait for `crates.io` to process, and check everything out
