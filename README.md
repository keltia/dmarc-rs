<!-- omit in TOC -->
# dmarc-rs

> **Command-line analyze and display of DMARC reports**

[![Build Status](https://api.cirrus-ci.com/github/keltia/dmarc-rs.svg?branch=main)](https://cirrus-ci.org/keltia/dmarc-rs)
[![Crates.io](https://img.shields.io/crates/v/dmarc-rs.svg)](https://crates.io/crates/docs_rs)
[![Docs](https://img.shields.io/docsrs/dmarc-rs)](https://docs.rs/dmarc-rs)
[![GitHub release](https://img.shields.io/github/release/keltia/dmarc-rs.svg)](https://github.com/keltia/dmarc-rs/releases/)
[![GitHub issues](https://img.shields.io/github/issues/keltia/dmarc-rs.svg)](https://github.com/keltia/dmarc-rs/issues)
[![dmarc-rs: 1.56+]][Rust 1.56]
[![SemVer](https://img.shields.io/badge/semver-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)
[![License](https://img.shields.io/crates/l/mit)](https://opensource.org/licenses/MIT)

Licensed under the [MIT](LICENSE).

1. [About](#about)
2. [Installation](#installation)
3. [Usage](#usage)
4. [Output format](#columns)
5. [References](#references)
6. [MSRV](#msrv)
7. [Contributing](#contributing)

## About

`dmarc-rs` is a small command-line utility to analyze and display in a usable manner the content of the DMARC XML reports sent by the various email providers around the globe.  Should work properly on UNIX (FreeBSD, Linux, etc.) and Windows systems.  This is a Rust port of the [Go version](https://github.com/keltia/dmarc-cat/) utility.

## Supported platforms

* Unix (tested on FreeBSD, Linux and macOS)
* Windows
    * cmd.exe
    * Powershell

## Notes

The package is named `dmarc_rs` to distinguish it from the [Go] version but the binary will remain the same (`dmarc-cat`) and can totally replace it.

## Installation

As with many Rust utilities, a simple

    cargo install dmarc-rs

is enough to fetch, build and install.

On Windows systems, the above `cargo` command should work directly in a Powershell window.

### Packaging

I will insert here references to the binary packages in different distributions when available.

## Dependencies

The main XML parsing stuff is done by `serde` & associates and CLI handling is done with `clap`:

- [clap](https://lib.rs/crates/clap)
- [serde](https://libs.rs/crates/serde)
- [serde-xmls-rs](https://libs.rs/crates/serde-xml-rs)

`dmarc-rs` uses the following crates to enable reading zip & gzip files:

- [zip](https://lib.rs/crates/zip)
- [flate2](https://lib.rs/crates/flate2)

It also use the following crates for DNS resolving/threading from the report.

- [dns-lookup](https://lib.rs/crates/dns-lookup)
- [ThreadPool](https://lib.rs/crates/threadpool)

and a few other helper crates, especially if you want to run the tests.

## Usage

SYNOPSIS
```console
dmarc-cat 0.2.0
Ollivier Robert <roberto@keltia.net>
Rust utility to decode and display DMARC reports.

USAGE:
    dmarc-cat [OPTIONS] [FILES]...

ARGS:
    <FILES>...    Filenames (possibly none or -)

OPTIONS:
    -D, --debug                 debug mode
    -h, --help                  Print help information
    -j, --jobs <JOBS>           Use this many parallel jobs for resolving IP [default: 6]
    -N, --no-resolve            Do not resolve IP to names
    -t, --input-type <ITYPE>    Specify the type of input data
    -v, --verbose               Verbose mode
    -V, --version               Display version and exit
```
        	
Example:
```console
$ dmarc-cat /tmp/yahoo.com\!keltia.net\!1518912000\!1518998399.xml

Reporting by: Yahoo! Inc. — postmaster@dmarc.yahoo.com
From 2018-02-18 01:00:00 +0100 CET to 2018-02-19 00:59:59 +0100 CET

Domain: keltia.net
Policy: p=none; dkim=r; spf=r

Reports(1):
IP            Count   From       RFrom      RDKIM   RSPF
88.191.250.24 1       keltia.net keltia.net neutral pass
```

## Columns

The full XML grammar is available [here](https://tools.ietf.org/html/rfc7489#appendix-C) and there is a local
copy in the `doc/` directory in the source.

The report has several columns:

- `IP` is matching IP address
- `Count` is the number of times this IP was present
- `From` is the `From:` header value
- `RFrom` is the envelope `From` value
- `RDKIM` is the result from DKIM checking
- `RSPF` is the result from SPF checking

## Supported formats

The file sent by MTAs can differ in format, some providers send zip files with both csv and XML files, some directly send compressed XML files.  This utility should handle the different format but you will have to use `-t TYPE` if you want to read from standard input.

## Tests

Tests are available as unit-tests for the library part and as integration tests for the CLI interaction (see `tests/cli.rs`).

## References

- [DMARC]
- [SPF]
- [DKIM]

## MSRV

The Minimum Supported Rust Version is 1.56 due to the 2021 Edition. 

## Contributing

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for some simple rules.

I use Git Flow for this package so please use something similar or the usual github workflow.

1. Fork it ( https://github.com/keltia/dmarc-rs/fork )
2. Checkout the develop branch (`git checkout develop`)
3. Create your feature branch (`git checkout -b my-new-feature`)
4. Commit your changes (`git commit -am 'Add some feature'`)
5. Push to the branch (`git push origin my-new-feature`)
6. Create a new Pull Request

[DMARC]: https://dmarc.org/
[SPF]: http://www.rfc-editor.org/info/rfc7208
[DKIM]: http://www.rfc-editor.org/info/rfc6376
[dmarc-rs: 1.56+]: https://img.shields.io/badge/Rust%20version-1.56%2B-lightgrey
[Rust 1.56]: https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html
[Go]: https://golang.org/
