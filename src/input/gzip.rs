//! Module to deal with gzip/deflate files
//!

use flate2::read::GzDecoder;
use std::io::prelude::*;
