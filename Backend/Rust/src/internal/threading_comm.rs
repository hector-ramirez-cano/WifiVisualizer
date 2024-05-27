#[derive(PartialEq)]
pub enum Message {
    StartCapture(crate::model::types::Project),
    BackendReady(bool),

    BackendStatusRequest,
}