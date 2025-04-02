use crate::send_event;
use eyre::Error;

const AI_JOB_STATUS: &str = "web-node.ai-job.status";

pub fn send_ai_job_status(status: &AiJob) -> Result<(), Error> {
    send_event(AI_JOB_STATUS, status)
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum AiJob {
    Started,
    Update { tps: f64 },
    Done,
}
