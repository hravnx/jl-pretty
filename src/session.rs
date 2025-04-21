use regex::Regex;

/// When and how to detect a new session
pub enum SessionStartDetector {
    /// no new sessions are detected
    Never,
    /// If a message is exactly equal to this, it's a new session
    Equals(&'static str),
    /// If a message matches this regular expression, it's a new session
    Matches(Regex),
}

impl SessionStartDetector {
    /// Creates a new session detector that never detects a new session
    pub fn never() -> Self {
        Self::Never
    }

    /// Creates a new session detector that detects a new session if the message
    /// is equal to a given string
    pub fn from_message(s: &'static str) -> Self {
        Self::Equals(s)
    }

    /// Creates a new session detector that detects a new session if the message
    /// matches a given regular expression
    pub fn from_rx(rx: Regex) -> Self {
        Self::Matches(rx)
    }

    /// Checks if the given line indicates a new session
    pub fn is_new_session(&self, line: &str) -> bool {
        match self {
            SessionStartDetector::Never => false,
            SessionStartDetector::Equals(s) => line == *s,
            SessionStartDetector::Matches(rx) => rx.is_match(line),
        }
    }
}

// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_never_should_break() {
        let detector = SessionStartDetector::never();
        assert!(!detector.is_new_session("any line"));
    }

    #[test]
    fn test_equals_should_break() {
        let detector = SessionStartDetector::from_message("start");
        assert!(detector.is_new_session("start"));
        assert!(!detector.is_new_session("not start"));
    }

    #[test]
    fn test_matches_should_break() {
        let rx = Regex::new(r"^start.*").unwrap();
        let detector = SessionStartDetector::from_rx(rx);
        assert!(detector.is_new_session("start here"));
        assert!(!detector.is_new_session("not starting"));
    }

    #[test]
    fn test_matches_empty_line() {
        let rx = Regex::new(r"^\s*$").unwrap();
        let detector = SessionStartDetector::from_rx(rx);
        assert!(detector.is_new_session(""));
        assert!(detector.is_new_session("   "));
        assert!(!detector.is_new_session("not empty"));
    }

    #[test]
    fn test_equals_case_sensitive() {
        let detector = SessionStartDetector::from_message("Start");
        assert!(detector.is_new_session("Start"));
        assert!(!detector.is_new_session("start"));
    }
}
