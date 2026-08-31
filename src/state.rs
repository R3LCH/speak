#[derive(Debug)]
pub enum State {
    Idle,
    Recording,
    Transcribing,
    Error,
}
