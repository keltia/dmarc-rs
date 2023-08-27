//! versioning stuff

use clap::crate_description;

use crate::cli::{AUTHORS, NAME, VERSION};

/// Display our version banner
///
/// Example:
/// ```
/// # use dmarc_rs::version::version;
///
/// println!(version());
/// ```
///
#[inline]
pub fn version() -> String {
    format!(
        "{}/{}/j{} by {}\n{}",
        NAME,
        VERSION,
        num_cpus::get_physical(),
        AUTHORS,
        crate_description!()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(version().contains(NAME));
        assert!(version().contains(VERSION));
        assert!(version().contains(AUTHORS))
    }
}
