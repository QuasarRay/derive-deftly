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

impl ToTokens for Version {
    fn to_tokens(&self, out: &mut TokenStream) {
        let lit = proc_macro2::Literal::string(&self.0);
        out.extend([TT::Literal(lit)]);
    }
}

fn read_tested_back_compat() -> Version {
    let cargo = fs::read_to_string("../tests/compat/old-b/Cargo.toml")
        .unwrap()
        .parse::<toml::Value>()
        .unwrap();
    let dep = &cargo //
        .as_table()
        .unwrap()["dependencies"]
        .as_table()
        .unwrap()["derive-deftly"];
    let version = if let Some(t) = dep.as_table() {
        &t["version"]
    } else {
        dep
    };
    let version = version.as_str().unwrap().strip_prefix("=").unwrap();
    println!("tested back compat    {version}");
    Version(version.to_owned())
}

fn read_current() -> Version {
    let cargo = fs::read_to_string("../Cargo.toml")
        .unwrap()
        .parse::<toml::Value>()
        .unwrap();
    let version = cargo //
        .as_table()
        .unwrap()["package"]
        .as_table()
        .unwrap()["version"]
        .as_str()
        .unwrap();
    println!("current               {version}");
    Version(version.to_owned())
}

fn read_changelog(back_compat: &Version) -> (Vec<Version>, Version) {
    let file = File::open("../CHANGELOG.md").unwrap();
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
        let version = version
            .split_once(' ')
            .map(|(l, _)| l)
            .unwrap_or(version)
            .to_owned();
        let version = Version(version);
        if had_back_compat {
            println!("changelog {lno:3} broken  {version}");
            return (good, version);
        } else {
            println!("changelog {lno:3} good    {version}");
            good.push(version.clone());
        }
        if &version == back_compat {
            had_back_compat = true;
        }
    }
}

fn check_1(v: &Version, exp: Result<(), ()>) {
    let got = macros::semver::pub_template_semver_check_func_macro(
        quote! { #v }, //
    );
    eprintln!("{v:10} exp={exp:?} got={got:?}");
    assert_eq!(got.is_ok(), exp.is_ok());
}

#[test]
fn check() {
    let current = read_current();
    let back_compat = read_tested_back_compat();
    let (changelog_good, changelog_bad) = read_changelog(&back_compat);

    for v in chain!([&current, &back_compat], &changelog_good) {
        check_1(v, Ok(()));
    }
    for v in [&changelog_bad] {
        check_1(v, Err(()));
    }
}
