#![allow(clippy::non_minimal_cfg)]

#[cfg(all(
    feature = "recent", // we want toml parser etc.
))]
mod check_semver_check;
