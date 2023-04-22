//! TODO: see known issues
//!
//! # Known issues
//!
//! serde_json will fail when serializing `Path`s containing invalid
//! UTF-8; also if serializing `OsString` fields, the json output
//! will not be human-readable (it's like this:
//! `"f":{"Unix":[112,97,115,115,119,100]}`)
//!
//! Also `bincode` v2.0 removes ser/de capabilities for `OsString`s.
//! Currently `bincode` v1 is still used.
//!
//! I haven't come up with an elegant idea to handle these
//! maybe-non-UTF8-encoded strings

use std::env;
use std::env::args_os;
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::SystemTime;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::group::FileEntry;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub creation_time: String,
    pub cmd_args: Vec<OsString>,
    pub base_dir: Option<PathBuf>,
    pub groups: Vec<Group>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub file_size: u64,
    pub hash: String,
    pub files: Vec<OsString>,
}

pub fn build_output<const L: usize>(groups: &[([u8; L], Vec<FileEntry>)]) -> Output {
    let time = DateTime::<Local>::from(SystemTime::now()).to_rfc3339();

    let groups = groups
        .iter()
        .map(|x| Group {
            hash: hex::encode(x.0),
            file_size: x.1[0].size,
            files: x
                .1
                .iter()
                .map(|x| x.path.clone().into_os_string())
                .collect(),
        })
        .collect::<Vec<_>>();

    let cmd_args = args_os().skip(1).collect::<Vec<_>>();

    Output {
        creation_time: time,
        cmd_args,
        base_dir: env::current_dir().ok(),
        groups,
    }
}
