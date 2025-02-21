use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_tree::HierarchicalLayer;

use dmarc_rs::{res_init, ResType, Solver};

use crate::cli::Opts;

/// Initializes the runtime environment including logging and resolver setup.
///
/// This function configures the logging system using the `tracing` crate.
/// It sets up a hierarchical logging layer with various options such as
/// verbose entry, exit, and field formatting. The logging filter can be
/// customized through environment variables using the `RUST_LOG` key.
///
/// Additionally, it initializes the resolver behavior based on the `--no-resolve`
/// flag. If this flag is set, a null resolver is used; otherwise, a real resolver
/// is initialized.
///
/// # Arguments
///
/// * `opts` - A reference to the parsed command-line options.
///
/// # Returns
///
/// Returns an `eyre::Result` that holds a `Solver` if initialization is successful,
/// or an error if something goes wrong during the setup process.
///
/// # Errors
///
/// Will return an error if any of the initialization steps fail, such as resolver
/// setup issues.
///
/// # Tracing
///
/// This function contains a tracing instrumentation for detailed logging of
/// the initialization process.
///
/// # Example
///
/// ```rust
/// use crate::init::init_runtime;
/// use crate::cli::Opts;
///
/// // Simulate CLI input
/// //
/// let opts = Opts {
///     debug: true,
///     noresolve: false,
///     verbose: 2,
///     quiet: false,
///     version: false,
///     itype: None,
///     files: None,
/// };
///
/// let result = init_runtime(&opts);
/// assert!(result.is_ok());
/// ```
///
#[tracing::instrument]
pub(crate) fn init_runtime(opts: &Opts) -> eyre::Result<Solver> {
    // Initialise logging.
    //
    let tree = HierarchicalLayer::new(2)
        .with_ansi(true)
        .with_span_retrace(true)
        .with_span_modes(true)
        .with_targets(true)
        .with_verbose_entry(true)
        .with_verbose_exit(true)
        .with_bracketed_fields(true);

    // Load filters from environment
    //
    let filter = EnvFilter::from_default_env();

    // Combine filter & specific format
    //
    tracing_subscriber::registry()
        .with(filter)
        .with(tree)
        .init();

    // Handle --no-resolv flag
    //
    let res = if opts.noresolve {
        info!("noresolv");
        res_init(ResType::Null)
    } else {
        info!("regular resolver");
        res_init(ResType::Real)
    };

    Ok(res)
}
