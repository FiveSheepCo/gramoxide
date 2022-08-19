#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Request error: {error}")]
    ReqwestError {
        #[from]
        error: reqwest::Error,
    },
    #[error("IO error: {error}")]
    IoError {
        #[from]
        error: std::io::Error,
    },
    #[error("This should never happen")]
    ShortCircuit,
}
