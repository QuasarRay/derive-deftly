use derive_deftly::pub_template_semver_check;

pub_template_semver_check!("0.2");
pub_template_semver_check!("0.2.3");
pub_template_semver_check!("1000000000.0");

pub_template_semver_check!(garbage);
pub_template_semver_check!("garbage");
pub_template_semver_check!("1.2.3.4");

fn main() {}
