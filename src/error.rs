use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum VanaspatiError {
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("growth error: {0}")]
    GrowthError(String),
    #[error("computation error: {0}")]
    ComputationError(String),
}

pub type Result<T> = std::result::Result<T, VanaspatiError>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn error_display() {
        let e = VanaspatiError::GrowthError("drought".into());
        assert!(e.to_string().contains("drought"));
    }
}
