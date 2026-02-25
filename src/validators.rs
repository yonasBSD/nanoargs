use std::fmt;
use std::sync::Arc;

type ValidatorFn = dyn Fn(&str) -> Result<(), String> + Send + Sync;

/// A value validator: a closure that checks a raw string value,
/// plus an optional hint string for help text.
#[derive(Clone)]
pub struct Validator {
    func: Arc<ValidatorFn>,
    hint: Option<String>,
}

impl Validator {
    /// Create a validator from a closure.
    pub fn new(f: impl Fn(&str) -> Result<(), String> + Send + Sync + 'static) -> Self {
        Self {
            func: Arc::new(f),
            hint: None,
        }
    }

    /// Create a validator from a closure with a hint string for help text.
    pub fn with_hint(hint: &str, f: impl Fn(&str) -> Result<(), String> + Send + Sync + 'static) -> Self {
        Self {
            func: Arc::new(f),
            hint: Some(hint.to_string()),
        }
    }

    /// Run the validator on a value.
    pub fn validate(&self, value: &str) -> Result<(), String> {
        (self.func)(value)
    }

    /// Returns the hint string, if any.
    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }
}

impl PartialEq for Validator {
    fn eq(&self, other: &Self) -> bool {
        self.hint == other.hint
    }
}

impl Eq for Validator {}

impl fmt::Debug for Validator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.hint {
            Some(h) => write!(f, "Validator({h})"),
            None => write!(f, "Validator(<custom>)"),
        }
    }
}

/// Returns a Validator that checks if the value parses as i64 and falls within [min, max].
pub fn range(min: i64, max: i64) -> Validator {
    let hint = format!("[{}..{}]", min, max);
    Validator::with_hint(&hint, move |v| match v.parse::<i64>() {
        Ok(n) if n >= min && n <= max => Ok(()),
        Ok(_) => Err(format!("value must be between {} and {}", min, max)),
        Err(_) => Err(format!("'{}' is not a valid number", v)),
    })
}

/// Returns a Validator that checks if the value is one of the allowed strings.
pub fn one_of(allowed: &[&str]) -> Validator {
    let owned: Vec<String> = allowed.iter().map(|s| s.to_string()).collect();
    let hint = owned.join("|");
    Validator::with_hint(&hint, move |v| {
        if owned.iter().any(|a| a == v) {
            Ok(())
        } else {
            Err(format!("must be one of: {}", owned.join(", ")))
        }
    })
}
/// Returns a Validator that rejects empty strings.
pub fn non_empty() -> Validator {
    Validator::with_hint("non-empty", |v| {
        if v.is_empty() {
            Err("value must not be empty".into())
        } else {
            Ok(())
        }
    })
}

/// Returns a Validator that rejects strings shorter than `n` bytes.
pub fn min_length(n: usize) -> Validator {
    Validator::with_hint(&format!("[min_length: {}]", n), move |v| {
        if v.len() >= n {
            Ok(())
        } else {
            Err(format!("value must be at least {} characters", n))
        }
    })
}

/// Returns a Validator that rejects strings longer than `n` bytes.
pub fn max_length(n: usize) -> Validator {
    Validator::with_hint(&format!("[max_length: {}]", n), move |v| {
        if v.len() <= n {
            Ok(())
        } else {
            Err(format!("value must be at most {} characters", n))
        }
    })
}

/// Returns a Validator that rejects values where the path does not exist.
///
/// This validator uses `std::path::Path::new(v).exists()` and is the only
/// convenience validator that requires `std`. In a future `no_std` build,
/// this function would need to be feature-gated.
pub fn path_exists() -> Validator {
    Validator::with_hint("existing path", |v| {
        if std::path::Path::new(v).exists() {
            Ok(())
        } else {
            Err(format!("path '{}' does not exist", v))
        }
    })
}
