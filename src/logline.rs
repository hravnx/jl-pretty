use sonic_rs::Deserialize;

#[derive(Deserialize)]
pub struct LogLine<'a> {
    timestamp: &'a str,
    level: &'a str,
    message: &'a str,
}

impl<'a> LogLine<'a> {
    pub fn from_message(message: &'a str) -> Self {
        Self {
            timestamp: "2025-01-01T12:00:00.345Z",
            level: "info",
            message,
        }
    }

    pub fn timestamp(&self) -> &'a str {
        self.timestamp
    }
    pub fn level(&self) -> &'a str {
        self.level
    }
    pub fn message(&self) -> &'a str {
        self.message
    }
}
