use std::path::PathBuf;
use thiserror::Error;

/// Status represents various error conditions that can arise in the application.
///
/// The enum provides error variants that describe specific failure scenarios,
/// along with optional context information for better debugging.
///
/// # Variants
///
/// * `EmptyList`
///   - Indicates that a required list was found to be empty.
///
/// * `WritingFile(PathBuf, String)`
///   - Indicates that a file could not be written. The `PathBuf` contains the path of the file
///     and the `String` provides a description of the failure.
///
/// # Example Usage
///
/// ```rust
/// use std::path::PathBuf;
/// use dmarc_rs::Status;
///
/// let error = Status::WritingFile(PathBuf::from("/path/to/file"), "Permission denied".to_string());
///
/// match error {
///     Status::EmptyList => println!("The list is empty."),
///     Status::WritingFile(path, message) => {
///         println!("Failed to write to {:?}: {}", path, message);
///     }
/// }
/// ```
///
#[derive(Debug, Error)]
pub enum Status {
    #[error("Empty list")]
    EmptyList,
    #[error("Can not write into {0}: {1}")]
    WritingFile(PathBuf, String),
}
