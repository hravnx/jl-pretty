use sonic_rs::{JsonValueTrait, Value, from_str};

use crate::session_detector::SessionDetector;

// --------------------------------------------------------------------------

#[cfg(windows)]
const SESSION_BREAK: &str = "\x1b[34m---------------------------------------------\x1b[m\r\n";

#[cfg(not(windows))]
const SESSION_BREAK: &str = "\x1b[34m---------------------------------------------\x1b[m\n";

#[cfg(windows)]
const RESET_EOL: &str = "\x1b[m\r\n";

#[cfg(not(windows))]
const RESET_EOL: &str = "\x1b[m\n";

const DEFAULT_COLOR_TABLE: [&str; 8] = [
    "\x1b[38;5;5m",   // Unknown -> Purple
    "\x1b[38;5;9m",   // Error -> Red
    "\x1b[38;5;11m",  // Warning -> Yellow
    "\x1b[38;5;254m", // Info -> White
    "\x1b[38;5;97m",  // Http -> Purple-ish
    "\x1b[38;5;6m",   // Verbose -> Cyan
    "\x1b[38;5;27m",  // Debug -> Blue
    "\x1b[38;5;27m",  // Silly -> Blue
];

const DEFAULT_LEVEL_LABEL_TABLE: [&str; 8] = [
    " [unk] ", // Unknown
    " [err] ", // Error
    " [wrn] ", // Warning
    " [inf] ", // Info
    " [web] ", // Http
    " [vrb] ", // Verbose
    " [dbg] ", // Debug
    " [sil] ", // Silly
];

// --------------------------------------------------------------------------

pub struct LineProcessor<'a> {
    detector: SessionDetector,
    timestamp_color: &'a str,
    color_table: [&'a str; 8],
    label_table: [&'a str; 8],
}

pub enum ProcessError {
    Unknown,
}

impl<'a> LineProcessor<'a> {
    pub fn new(detector: SessionDetector) -> Self {
        Self {
            detector,
            timestamp_color: "\x1b[36m",
            color_table: DEFAULT_COLOR_TABLE,
            label_table: DEFAULT_LEVEL_LABEL_TABLE,
        }
    }

    fn get_level_index(level: &str) -> usize {
        match level {
            "error" => 1,
            "warn" => 2,
            "info" => 3,
            "http" => 4,
            "verbose" => 5,
            "debug" => 6,
            "silly" => 7,
            _ => 0, // Unknown
        }
    }

    pub fn process_lines<R, W>(&self, lines: R, w: &mut W) -> std::io::Result<()>
    where
        R: std::io::BufRead,
        W: std::io::Write,
    {
        let mut line_buffer = String::with_capacity(8192);
        for line in lines.lines() {
            line_buffer.clear();
            let buf = self.process_line(line?, &mut line_buffer);
            w.write_all(buf.as_bytes())?;
        }
        Ok(())
    }

    fn process_line(&'a self, line: String, line_buffer: &'a mut String) -> &'a str {
        // parse the JSON line
        let log_line: Value = from_str(line.as_str()).unwrap();

        //
        let ts = log_line.get("timestamp").unwrap().as_str().unwrap();
        let level = log_line.get("level").unwrap().as_str().unwrap();

        let message = log_line.get("message").unwrap().as_str().unwrap();
        if self.detector.is_new_session(message) {
            line_buffer.push_str(SESSION_BREAK);
        }

        let level_index = Self::get_level_index(level);
        let level_color = self.color_table[level_index];
        let level_label = self.label_table[level_index];

        // time stamp
        line_buffer.push_str(self.timestamp_color);
        line_buffer.push_str(&ts[11..]);

        // level
        line_buffer.push_str(level_color);
        line_buffer.push_str(level_label);

        // message (reusing the color from level)
        line_buffer.push_str(message);

        // reset colors and write new line
        line_buffer.push_str(RESET_EOL);

        // return the formatted line
        line_buffer.as_str()
    }
}
