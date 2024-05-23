#[derive(PartialEq)]
pub enum Message {
    StartCapture,
    BackendReady(bool),

    BackendStatusRequest,
}