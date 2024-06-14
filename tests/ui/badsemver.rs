use derive_deftly::{pub_template_semver_check, template_export_semver_check};

template_export_semver_check!("0.2");
template_export_semver_check!("0.2.3");
template_export_semver_check!("1000000000.0");

template_export_semver_check!(garbage);
template_export_semver_check!("garbage");
pub_template_semver_check!("1.2.3.4");

fn main() {}
