use crate::{ProcessError, ansi_color, logline::LogLine, session::SessionStartDetector};

// --------------------------------------------------------------------------

#[cfg(windows)]
const SESSION_BREAK_COLOR: &str = "\x1b[34m---------------------------------------------\x1b[m\r\n";

#[cfg(not(windows))]
const SESSION_BREAK_COLOR: &str = "\x1b[34m---------------------------------------------\x1b[m\n";

#[cfg(windows)]
const SESSION_BREAK: &str = "---------------------------------------------\r\n";

#[cfg(not(windows))]
const SESSION_BREAK: &str = "---------------------------------------------\n";

#[cfg(windows)]
const END_OF_LINE: &str = "\r\n";

#[cfg(not(windows))]
const END_OF_LINE: &str = "\n";

#[cfg(windows)]
const END_OF_LINE_COLOR: &str = "\x1b[m\r\n";

#[cfg(not(windows))]
const END_OF_LINE_COLOR: &str = "\x1b[m\n";

// --------------------------------------------------------------------------

const DEFAULT_LEVEL_TABLE: [&str; 8] = [
    " [unk] ", // Unknown
    " [err] ", // Error
    " [wrn] ", // Warning
    " [inf] ", // Info
    " [web] ", // Http
    " [vrb] ", // Verbose
    " [dbg] ", // Debug
    " [sil] ", // Silly
];

// See color table here https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit
//
// Color names are from https://colornamer.robertcooper.me/
const DEFAULT_LEVEL_TABLE_COLOR: [&str; 8] = [
    concat!(ansi_color!(fg: 5), " [unk] "),   // Unknown -> Purple
    concat!(ansi_color!(fg: 9), " [err] "),   // Error -> Red
    concat!(ansi_color!(fg: 11), " [wrn] "),  // Warning -> Yellow
    concat!(ansi_color!(fg: 254), " [inf] "), // Info -> Titanium White
    concat!(ansi_color!(fg: 97), " [web] "),  // Http -> Lusty Lavender
    concat!(ansi_color!(fg: 6), " [vrb] "),   // Verbose -> Teal
    concat!(ansi_color!(fg: 27), " [dbg] "),  // Debug -> Bright Blue
    concat!(ansi_color!(fg: 27), " [sil] "),  // Silly -> Bright Blue
];

// --------------------------------------------------------------------------

/// Processes lines from the log (in jsonl format)
pub struct LineProcessor<'a> {
    detector: SessionStartDetector,
    skip_invalid_lines: bool,
    timestamp_prefix: &'a str,
    level_table: [&'a str; 8],
    break_line: &'a str,
    eol: &'a str,
}

impl LineProcessor<'_> {
    /// Creates a new line processor
    ///
    pub fn new(detector: SessionStartDetector, skip_invalid_lines: bool, use_color: bool) -> Self {
        let (break_line, eol, level_table, timestamp_prefix) = if use_color {
            (
                SESSION_BREAK_COLOR,
                END_OF_LINE_COLOR,
                DEFAULT_LEVEL_TABLE_COLOR,
                ansi_color!(fg: 6),
            )
        } else {
            (SESSION_BREAK, END_OF_LINE, DEFAULT_LEVEL_TABLE, "")
        };

        Self {
            detector,
            skip_invalid_lines,
            break_line,
            eol,
            timestamp_prefix,
            level_table,
        }
    }

    fn get_level_label(&self, level: &str) -> &str {
        match level {
            "error" => self.level_table[1],
            "warn" => self.level_table[2],
            "info" => self.level_table[3],
            "http" => self.level_table[4],
            "verbose" => self.level_table[5],
            "debug" => self.level_table[6],
            "silly" => self.level_table[7],
            _ => self.level_table[0], // Unknown
        }
    }

    /// Processes lines from the given [`BufRead`] implementation and writes
    /// each parsed line to the given [`Write`] implementation.
    ///
    /// Aborts and returns an error if we could not read from `lines` or write
    /// to `w` at any point. If any of the lines are not parsable, we either
    /// abort and return an error, or skip, depending on the setting of
    /// `skip_invalid_lines`
    ///
    pub fn process_lines<R, W>(&self, lines: R, w: &mut W) -> crate::Result<()>
    where
        R: std::io::BufRead,
        W: std::io::Write,
    {
        // The line buffer is re-used for all lines, so giving it a large-ish
        // capacity will minimize runtime allocations, assuming the vast
        // majority of log lines will be less than 8Kib bytes long.
        let mut line_buffer = String::with_capacity(8192);
        // we track the line number of the log for debugging (invalid json lines
        // in the log, for example)
        let mut line_no = 1;
        for line in lines.lines() {
            line_buffer.clear();
            let line = line.map_err(|e| ProcessError::from_read_error(line_no, e))?;
            self.process_line(line_no, line, &mut line_buffer)?;
            w.write_all(line_buffer.as_bytes())
                .map_err(ProcessError::from_write_error)?;
            line_no += 1;
        }
        Ok(())
    }

    /// Helper function that processes a single line of the log
    ///
    fn process_line(
        &self,
        line_no: usize,
        line: String,
        line_buffer: &mut String,
    ) -> crate::Result<()> {
        // parse the JSON line
        match sonic_rs::from_slice::<LogLine>(line.as_bytes()) {
            Err(err) => {
                if !self.skip_invalid_lines {
                    Err(ProcessError::from_parse_error(line_no, err))
                } else {
                    line_buffer.push_str(format!("--- skipped line {line_no} ---").as_str());
                    line_buffer.push_str(self.eol);
                    Ok(())
                }
            }
            Ok(log_line) => {
                if self.detector.is_new_session(&log_line) {
                    line_buffer.push_str(self.break_line);
                }
                self.print_log_line(line_buffer, &log_line);
                Ok(())
            }
        }
    }

    fn print_log_line(&self, line_buffer: &mut String, log_line: &LogLine) {
        // timestamp
        line_buffer.push_str(self.timestamp_prefix);
        line_buffer.push_str(&log_line.timestamp()[11..]);

        // level
        line_buffer.push_str(self.get_level_label(log_line.level()));

        // message (reusing the color state from level)
        line_buffer.push_str(log_line.message());

        // reset colors and write new line
        line_buffer.push_str(self.eol);
    }
}
