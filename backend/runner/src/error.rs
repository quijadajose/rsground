#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum RunnerError {
    Container(#[from] hakoniwa::Error),

    #[error("Status code not successful ({}): {}", .0.status.reason, String::from_utf8_lossy(&.0.stderr))]
    NotOk(hakoniwa::Output),
}
