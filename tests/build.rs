// build.rs for tests

// We make extensive use of cargo features, and compile various
// of the same code in different cargo packages with different feature sets.
// This means that cargo doesn't always know the features we're testing
// really exist.  Likewise with our `--cfg=derive_deftly_dprint`.
//
// Here, we tell rustc, via cargo, that these are all allowed.

// This is a superset of macros/build.rs, in a more sophisticated style.

use std::fmt::Write as _;

fn main() {
    let features = "bizarre expect skip_imported_driver"
        .split_ascii_whitespace()
        .map(|f| format!(r#""{}", "#, f))
        .collect::<String>();

    let mut o = String::new();
    let mut pc = |s| writeln!(o, "cargo::rustc-check-cfg=cfg({})", s).unwrap();

    pc("derive_deftly_dprint");
    pc(&format!("feature, values({})", features));
    print!("{}", o);
}
