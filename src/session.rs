use regex::Regex;

use crate::logline::LogLine;

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
    pub fn is_new_session(&self, line: &LogLine) -> bool {
        match self {
            SessionStartDetector::Never => false,
            SessionStartDetector::Equals(s) => line.message() == *s,
            SessionStartDetector::Matches(rx) => rx.is_match(line.message()),
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
        let line = LogLine::from_message("any line");
        assert!(!detector.is_new_session(&line));
    }

    #[test]
    fn test_equals_should_break() {
        let detector = SessionStartDetector::from_message("start");
        assert!(detector.is_new_session(&LogLine::from_message("start")));
        assert!(!detector.is_new_session(&LogLine::from_message("not start")));
    }

    #[test]
    fn test_matches_should_break() {
        let rx = Regex::new(r"^start.*").unwrap();
        let detector = SessionStartDetector::from_rx(rx);
        assert!(detector.is_new_session(&LogLine::from_message("start session")));
        assert!(!detector.is_new_session(&LogLine::from_message("not start session")));
    }

    #[test]
    fn test_matches_empty_line() {
        let rx = Regex::new(r"^\s*$").unwrap();
        let detector = SessionStartDetector::from_rx(rx);
        assert!(detector.is_new_session(&LogLine::from_message("")));
        assert!(detector.is_new_session(&LogLine::from_message("   ")));
        assert!(!detector.is_new_session(&LogLine::from_message("not empty")));
    }

    #[test]
    fn test_equals_case_sensitive() {
        let detector = SessionStartDetector::from_message("Start");
        assert!(detector.is_new_session(&LogLine::from_message("Start")));
        assert!(!detector.is_new_session(&LogLine::from_message("start")));
    }
}
