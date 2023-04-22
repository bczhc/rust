use std::env;
use std::env::args_os;
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::SystemTime;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::group::FileEntry;

pub mod binary;
pub mod json;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    creation_time: String,
    cmd_args: Vec<OsString>,
    base_dir: Option<PathBuf>,
    groups: Vec<Group>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    file_size: u64,
    hash: String,
    files: Vec<OsString>,
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
