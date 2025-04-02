use crate::send_event;
use eyre::Error;

const NODE_STATUS: &str = "web-node.status";

pub fn send_node_status_event(status: NodeStatus) -> Result<(), Error> {
    send_event(NODE_STATUS, &status)
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum NodeStatus {
    Online,
    Offline,
}
