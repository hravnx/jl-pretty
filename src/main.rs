use std::process::ExitCode;

use jl_pretty::{LineProcessor, SessionDetector};

// --------------------------------------------------------------------------

fn main() -> ExitCode {
    // grab and lock stdin
    let stdin = std::io::stdin();
    let input_handle = stdin.lock();

    // grab and lock stdout
    let stdout = std::io::stdout();
    let mut output_handle = stdout.lock();

    let processor = LineProcessor::new(SessionDetector::from_message(
        "Motion server startup initiated",
    ));
    match processor.process_lines(input_handle, &mut output_handle) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
