//! Test that:
//!
//!  * The latest old-b version used by `tests/compat/old-b`
//!    is accepted by the checking macro.
//!
//!  * All the versions in the CHANGELOG which re textually
//!    before (so, later than) that version are accepted.
//!
//!  * The textually next version in the CHANGELOG
//!    (which ought to be the one just before the breaking change)
//!    is rejected.
//!
//!  * The current version (from our toplevel Cargo.toml) is accepted.

use std::fs;

use super::*;

#[derive(Clone, Eq, PartialEq)]
struct Version(String);

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

struct TrackedVersion {
    v: Version,
    why: String,
}

impl TrackedVersion {
    fn new(v: impl Display, why: impl Display) -> Self {
        let v = Version(v.to_string());
        let why = why.to_string();
        println!("{why:25} {v}");
        TrackedVersion { v, why }
    }
}

impl ToTokens for TrackedVersion {
    fn to_tokens(&self, out: &mut TokenStream) {
        let lit = proc_macro2::Literal::string(&self.v.0);
        out.extend([TT::Literal(lit)]);
    }
}

fn read_some_cargo_toml(filename: &str) -> toml::value::Table {
    fs::read_to_string(filename)
        .unwrap()
        .parse::<toml::Value>()
        .unwrap()
        .as_table()
        .unwrap()
        .clone()
}

fn read_tested_back_compat() -> TrackedVersion {
    // We test (only) old-b here: that's the test case that tests
    // a template user using older d-d
    // depending on a template exporter using current d-d,
    // which is what the semver check (called in the exporter)
    // is guaranteeing will work.
    const PKG: &str = "tests/compat/old-b";
    let cargo = read_some_cargo_toml(&format!("../{PKG}/Cargo.toml"));
    let dep = &cargo["dependencies"] //
        .as_table()
        .unwrap()["derive-deftly"];
    let version = if let Some(t) = dep.as_table() {
        &t["version"]
    } else {
        dep
    };
    let version = version.as_str().unwrap().strip_prefix("=").unwrap();
    TrackedVersion::new(version, PKG)
}

fn read_current() -> TrackedVersion {
    let cargo = read_some_cargo_toml("../Cargo.toml");
    let version = cargo["package"] //
        .as_table()
        .unwrap()["version"]
        .as_str()
        .unwrap();
    TrackedVersion::new(version, "current")
}

fn read_changelog(
    back_compat: &Version,
) -> (Vec<TrackedVersion>, TrackedVersion) {
    const CHANGELOG: &str = "CHANGELOG.md";

    let file = File::open(format!("../{CHANGELOG}")).unwrap();
    let file = BufReader::new(file);
    let mut lines = file
        .lines()
        .enumerate()
        .map(|(lno, l)| (l.unwrap(), lno + 1));

    let mut good = vec![];

    loop {
        let (l, _lno) = lines.next().unwrap();
        if l.starts_with("## ") && l.contains("Changelog") {
            break;
        }
    }

    let mut had_back_compat = false;

    loop {
        let (l, lno) = lines.next().unwrap();
        assert!(!l.starts_with("## "));
        let Some(version) = l.strip_prefix("### ") else {
            continue;
        };
        let version_str =
            version.split_once(' ').map(|(l, _)| l).unwrap_or(version);
        let tracked_version = |status: &str| {
            TrackedVersion::new(
                version_str,
                format_args!("{CHANGELOG}:{lno} {status}"),
            )
        };
        if had_back_compat {
            return (good, tracked_version("broken"));
        } else {
            good.push(tracked_version("good"));
        }
        if &version_str == &back_compat.0 {
            had_back_compat = true;
        }
    }
}

fn check_1(tv: &TrackedVersion, exp: Result<(), ()>) {
    let got_full = if tv.v.0 == "UNRELEASED" {
        Ok(quote!(UNRELEASED OK))
    } else {
        macros::semver::template_export_semver_check_func_macro(
            quote! { #tv }, //
        )
    };
    let got_abbrev = got_full.as_ref().map_err(|_| ()).map(|_| ());
    eprintln!("{:25} {} exp={exp:?} got={got_abbrev:?}", tv.why, tv.v);
    assert_eq!(got_abbrev, exp, "at={} v={} got={got_full:?}", tv.why, tv.v);
}

#[test]
fn check() {
    println!("collecting:");
    let current = read_current();
    let back_compat = read_tested_back_compat();
    let (changelog_good, changelog_bad) = read_changelog(&back_compat.v);

    println!("checking:");
    for v in chain!([&current, &back_compat], &changelog_good) {
        check_1(v, Ok(()));
    }
    for v in [&changelog_bad] {
        check_1(v, Err(()));
    }
}
