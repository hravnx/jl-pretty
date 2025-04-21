use std::{
    fs::File,
    io::{BufReader, IsTerminal},
    path::PathBuf,
    process::ExitCode,
};

use clap::Parser;
use jl_pretty::{LineProcessor, ProcessError, SessionStartDetector};

// --------------------------------------------------------------------------

fn main() -> ExitCode {
    let cli = Cli::parse();

    // TODO: let this be controlled by args
    let session_detector = SessionStartDetector::from_message("Motion server startup initiated");

    // grab and lock stdout
    let stdout = std::io::stdout();
    let mut output = stdout.lock();

    // determine if we should output ANSI color codes
    let use_color = {
        // by default, we color the output (with ANSI codes) when writing to
        // terminals and not when writing to redirects or pipes. This may be
        // overridden by the flags '--use-color' or '--no-color'.
        let is_term = stdout.is_terminal();
        (is_term && !cli.no_color) || (!is_term && cli.use_color)
    };

    let processor = LineProcessor::new(session_detector, cli.skip_invalid_lines, use_color);

    // we either use the given file or std in as input
    let result = if let Some(input_file) = cli.input_file {
        match File::open(&input_file) {
            Ok(file) => processor.process_lines(BufReader::new(file), &mut output),
            Err(err) => Err(ProcessError::InputNotFound(input_file, err)),
        }
    } else {
        let stdin_handle = std::io::stdin();
        processor.process_lines(stdin_handle.lock(), &mut output)
    };

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

// --------------------------------------------------------------------------

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// File to read from, if not given std in is used
    #[arg(short, long, value_name = "FILE")]
    input_file: Option<PathBuf>,

    /// Force color output, even if std out is not a terminal
    #[arg(long, conflicts_with = "no_color")]
    use_color: bool,

    /// Force no color output, even if std out is a terminal
    #[arg(long)]
    no_color: bool,

    /// Skip invalid JSON lines in input (default behavior is to abort)
    #[arg(long)]
    skip_invalid_lines: bool,
}
