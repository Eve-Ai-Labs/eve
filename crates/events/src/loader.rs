use crate::send_event;
use eyre::Error;

const LOAD_STATUS: &str = "web-node.loading.progress";

pub fn send_load_status(status: &Result<LoadStatus, String>) -> Result<(), Error> {
    send_event(LOAD_STATUS, status)
}

#[derive(Debug, serde::Serialize, PartialEq, PartialOrd)]
pub enum LoadStatus {
    Start,
    Download(Progress),
    Progress(Progress),
    Compile,
    Done,
}

#[derive(Debug, serde::Serialize, PartialEq, PartialOrd)]
pub struct Progress {
    pub progress: f64,
}
