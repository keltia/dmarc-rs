//! All the different deserialization types and structs.
//!
//! This is a direct translation of appendix C of [RFC-7489]
//!
//! [RFC-7489]: https://tools.ietf.org/html/rfc7489#appendix-C

// Standard library
//
use std::net::IpAddr;

// External crates
//
use serde::Deserialize;

/// Date range.
#[derive(Debug, Default, Deserialize)]
pub struct DateRange {
    /// Start of date period
    pub begin: u32,
    /// End of date period
    pub end: u32,
}

/// Report metadata.
#[derive(Debug, Default, Deserialize)]
pub struct ReportMetadata {
    /// Organisation name
    pub org_name: String,
    /// Registered email contact
    pub email: String,
    /// More contact information (possibly empty)
    pub extra_contact_info: Option<String>,
    /// Report ID
    pub report_id: String,
    /// Date range for the report
    pub date_range: DateRange,
    /// Errors if any
    pub errors: Option<Vec<String>>,
}

/// Alignment (strict or relaxed) for DKIM and SPF.
#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub enum Alignment {
    r,
    s,
}

/// The policy actions specified by p and sp in the DMARC record.
#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub enum Disposition {
    none,
    quarantine,
    reject,
}

/// The DMARC policy that applied to the messages in this report, as published in the DNS
#[derive(Debug, Deserialize)]
pub struct PolicyPublished {
    /// The domain at which the DMARC record was found.
    pub domain: String,
    /// The DKIM alignment mode.
    pub adkim: Option<Alignment>,
    /// The SPF alignment mode.
    pub aspf: Option<Alignment>,
    /// The policy to apply to messages from the domain.
    pub p: Disposition,
    /// The policy to apply to messages from subdomains.
    pub sp: Disposition,
    /// The percent of messages to which policy applies.
    pub pct: usize,
    /// Failure reporting options in effect.
    pub fo: Option<String>,
}

/// The DMARC-aligned authentication result
#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub enum DMARCResult {
    fail,
    pass,
}

/// Reasons that may affect DMARC disposition or execution thereof
#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub enum PolicyOverride {
    forwarded,
    sampled_out,
    trusted_forwarder,
    mailing_list,
    local_policy,
    other
}

/// How do we allow report generators to include new classes of override reasons if they
/// want to be more specific than "other"?
#[derive(Debug, Deserialize)]
pub struct PolicyOverrideReason {
    /// Type of override
    #[serde(rename="type")]
    pub ptype: PolicyOverride,
    /// Textual reason
    pub comment: Option<String>,
}

/// Taking into account everything else in the record, the results of applying DMARC.
#[derive(Debug, Deserialize)]
pub struct PolicyEvaluated {
    /// Action taken
    pub disposition: Disposition,
    /// Result for DKIM
    pub dkim: DMARCResult,
    /// Result for SPF
    pub spf: DMARCResult,
    /// List of possible reasons
    pub reason: Option<Vec<PolicyOverrideReason>>,
}

/// Row for each IP address
#[derive(Debug, Deserialize)]
pub struct Row {
    /// The connecting IP
    pub source_ip: IpAddr,
    /// The number of matching messages.
    pub count: u32,
    /// The DMARC disposition applying to matching messages.
    pub policy_evaluated: PolicyEvaluated,
}

/// Row for each IP address
#[derive(Debug, Default, Deserialize)]
pub struct Identifier {
    /// The envelope recipient domain.
    pub envelope_to: Option<String>,
    /// The RFC5321.MailFrom domain.
    pub envelope_from: String,
    /// The RFC5322.From domain.
    pub header_from: String,
}

/// DKIM verification result, according to RFC 7001 Section 2.6.1.
#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub enum DKIMResult {
    none,
    pass,
    fail,
    policy,
    neutral,
    temperror,
    permerror,
}

/// The DKIM Authentication result.
#[derive(Debug, Deserialize)]
pub struct DKIMAuthResult {
    /// The "d=" parameter in the signature.
    pub domain: String,
    /// The "s=" parameter in the signature.
    pub selector: Option<String>,
    /// The DKIM verification result.
    pub result: DKIMResult,
    /// Any extra information (e.g., from Authentication-Results).
    pub human_result: Option<String>,
}

/// SPF domain scope.
#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub enum  SPFDomainScope {
    helo,
    mfrom,
}

/// The SPF result.
#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub enum  SPFResult {
    none,
    neutral,
    pass,
    fail,
    softfail,
    temperror,
    permerror,
}

/// The SPF Authentication result.
#[derive(Debug, Deserialize)]
pub struct SPFAuthResult {
    /// The checked domain.
    pub domain: String,
    /// The scope of the checked domain.
    pub scope: SPFDomainScope,
    /// The SPF verification result
    pub result: SPFResult,
}

/// This element contains DKIM and SPF results, uninterpreted with respect to DMARC.
#[derive(Debug, Deserialize)]
pub struct AuthResult {
    /// There may be no DKIM signatures, or multiple DKIM signatures.
    pub dkim: Option<Vec<DKIMAuthResult>>,
    /// There will always be at least one SPF result.
    pub spf: Vec<SPFAuthResult>,
}

/// This element contains all the authentication results that were evaluated by the
/// receiving system for the given set of messages.
#[derive(Debug, Deserialize)]
pub struct Record {
    pub row: Row,
    pub identifiers: Identifier,
    pub auth_results: Vec<AuthResult>,
}

/// The whole report itself.
#[derive(Debug, Deserialize)]
pub struct Feedback {
    pub version: f32,
    pub report_metadata: ReportMetadata,
    pub policy_published: PolicyPublished,
    pub records: Vec<Record>,
}
