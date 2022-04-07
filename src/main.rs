use clap::crate_version;

/// Binary name, using a different binary name
pub(crate) const NAME: &str = "dmarc-cat";
/// Binary version
pub(crate) const VERSION: &str = crate_version!();

fn main() {
    println!("{}/{}", NAME, VERSION);
}
