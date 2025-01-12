use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum Error {
    #[error("Overflow when converting types")]
    Overflow,
    #[error("Underflow when converting types")]
    Underflow,
    #[error("Error converting string {0} to U256")]
    ParseStringToU256(String),
}
