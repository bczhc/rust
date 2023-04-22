//! JSON format output
//!
//! TODO: see known issues
//!
//! # Known issues
//!
//! serde_json will fail when serializing `Path`s containing invalid
//! UTF-8; also if serializing `OsString` fields, the json output
//! will not be human-readable (it's like this:
//! `"f":{"Unix":[112,97,115,115,119,100]}`)
//!
//! I haven't come up with an elegant idea to handle these
//! maybe-non-UTF8-encoded strings
