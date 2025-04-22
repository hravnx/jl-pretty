use crate::{ProcessError, ansi_color, logline::LogLine, session::SessionStartDetector};
use std::fmt::Write;

// --------------------------------------------------------------------------

// We use a plain '\n' as a line terminator (not '\r\n') even on Windows. Most
// Windows terminals handle that just fine, and the old ones that might not,
// also don't support ANSI colors, so it's OK for all practical purposes.
const END_OF_LINE: &str = "\n";

const END_OF_LINE_COLOR: &str = concat!(ansi_color!(), "\n");

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
//
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
    new_session_prefix: &'a str,
    eol: &'a str,
}

impl LineProcessor<'_> {
    /// Creates a new line processor
    ///
    pub fn new(detector: SessionStartDetector, skip_invalid_lines: bool, use_color: bool) -> Self {
        let (new_session_prefix, eol, level_table, timestamp_prefix) = if use_color {
            (
                ansi_color!(fg: 4),
                END_OF_LINE_COLOR,
                DEFAULT_LEVEL_TABLE_COLOR,
                ansi_color!(fg: 6),
            )
        } else {
            ("", END_OF_LINE, DEFAULT_LEVEL_TABLE, "")
        };

        Self {
            detector,
            skip_invalid_lines,
            timestamp_prefix,
            level_table,
            new_session_prefix,
            eol,
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
                    eprintln!("--- skipped non-json line @ {line_no} ---");
                    Ok(())
                }
            }
            Ok(log_line) => {
                self.write_log_line(line_buffer, &log_line);
                Ok(())
            }
        }
    }

    /// Write the parsed log line to the destination
    fn write_log_line(&self, dest: &mut String, log_line: &LogLine) {
        // if we discover a new session has started, we emit that as a line
        // before the first event line
        if self.detector.is_new_session(log_line) {
            dest.push_str(self.new_session_prefix);
            writeln!(
                dest,
                "---- Session started {} ----",
                &log_line.timestamp()[..10]
            )
            .unwrap();
        }

        // timestamp using only the time part, i.e. '2025-01-01T23:34:56.987Z'
        // emits as '23:34:56.987Z'
        dest.push_str(self.timestamp_prefix);
        dest.push_str(&log_line.timestamp()[11..]);

        // level, translated to a fixed-sized version
        dest.push_str(self.get_level_label(log_line.level()));

        // message (reusing the color state from level)
        dest.push_str(log_line.message());

        // reset colors and write new line
        dest.push_str(self.eol);
    }

    /// Get the fixed-size label for a given level string, possibly prefixed
    /// with color codes.
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
}

// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::io::{BufRead, Cursor};

    use super::*;

    #[test]
    fn test_process_lines_with_valid_lines() {
        let detector = SessionStartDetector::Never;
        let processor = LineProcessor::new(detector, false, false);

        let input =
            r#"{"timestamp":"2023-01-01T12:00:00Z","level":"info","message":"Test message"}"#;

        let mut output = Vec::new();
        processor
            .process_lines(Cursor::new(input), &mut output)
            .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, "12:00:00Z [inf] Test message\n");
    }

    const INVALID_LINES_SOURCE: &str = include_str!("test-data/log-with-invalid-lines.jsonl");

    #[test]
    fn test_process_lines_with_invalid_lines_skip() {
        let detector = SessionStartDetector::Never;
        let processor = LineProcessor::new(detector, true, false);

        let mut output = Vec::new();

        processor
            .process_lines(Cursor::new(INVALID_LINES_SOURCE), &mut output)
            .unwrap();

        let output_lines = Cursor::new(output)
            .lines()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(output_lines.len(), 3);
        assert_eq!(output_lines[0], "20:25:01.133Z [inf] First valid line");
        assert_eq!(output_lines[1], "20:26:01.133Z [wrn] Second valid line");
        assert_eq!(output_lines[2], "20:27:01.133Z [inf] Third valid line");
    }

    #[test]
    fn test_process_lines_with_invalid_lines_abort() {
        let detector = SessionStartDetector::Never;
        let processor = LineProcessor::new(detector, false, false);

        let mut output = Vec::new();

        let result = processor.process_lines(Cursor::new(INVALID_LINES_SOURCE), &mut output);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_lines_with_new_session() {
        let detector = SessionStartDetector::from_message("Test message");
        let processor = LineProcessor::new(detector, false, false);

        let input =
            r#"{"timestamp":"2023-01-01T12:00:00Z","level":"info","message":"Test message"}"#;
        let mut output = Vec::new();

        processor
            .process_lines(Cursor::new(input), &mut output)
            .unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("---- Session started "));
    }

    #[test]
    fn test_get_level_label() {
        let detector = SessionStartDetector::Never;
        let processor = LineProcessor::new(detector, false, false);

        assert_eq!(processor.get_level_label("info"), " [inf] ");
        assert_eq!(processor.get_level_label("error"), " [err] ");
        assert_eq!(processor.get_level_label("unknown"), " [unk] ");
    }

    #[test]
    fn test_get_level_label_color() {
        let detector = SessionStartDetector::Never;
        let processor = LineProcessor::new(detector, false, true);

        assert_eq!(
            processor.get_level_label("info"),
            concat!(ansi_color!(fg: 254), " [inf] ")
        );
        assert_eq!(
            processor.get_level_label("error"),
            concat!(ansi_color!(fg: 9), " [err] ")
        );
        assert_eq!(
            processor.get_level_label("unknown"),
            concat!(ansi_color!(fg: 5), " [unk] ")
        );
    }
}
