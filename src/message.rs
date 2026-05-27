use thiserror::Error;

/// Errors returned by greeting helpers.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum GreetingError {
    /// The provided name was empty after trimming whitespace.
    #[error("name must not be empty")]
    EmptyName,
}

/// Build a friendly greeting for `name`.
///
/// # Errors
///
/// Returns [`GreetingError::EmptyName`] when `name` is empty or only whitespace.
pub fn greeting(name: &str) -> Result<String, GreetingError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(GreetingError::EmptyName);
    }

    Ok(format!("Hello, {trimmed}!"))
}

#[cfg(test)]
mod tests {
    use super::{GreetingError, greeting};

    #[test]
    fn greets_trimmed_name() {
        assert_eq!(greeting(" Ferris ").as_deref(), Ok("Hello, Ferris!"));
    }

    #[test]
    fn rejects_empty_name() {
        assert_eq!(greeting("  "), Err(GreetingError::EmptyName));
    }
}
