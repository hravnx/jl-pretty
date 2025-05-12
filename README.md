# jl-pretty

`jl-pretty` is a lightweight CLI tool that parses JSON-formatted log lines from
a a file or an input stream and pretty-prints them to the terminal as text.

## Features
- Colorization of lines for legibility.
- Detection of new sessions in the log stream.
- Optionally skip invalid JSON lines.
- Lightweight and fast, processing 500K+ lines per second on modern
  hardware.

## Installation

Make sure you have the Rust toolchain installed on your machine, see
[here](https://rustup.rs/) for instructions.

Install as a binary directly from the source:

```shell
cargo install --git https://github.com/hravnx/jl-pretty.git --branch main
```

Or clone the repository and build locally:

```shell
git clone https://github.com/hravnx/jl-pretty.git

cargo install --path ./jl-pretty
```

## Usage

Run on a log stream:
```shell
tail -f some-log.jsonl | jl-pretty --skip-invalid-lines
```

Run on a log file:
```shell
jl-pretty some-log.jsonl
```

See all options:

```shell
jl-pretty --help
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request to
suggest improvements or report bugs.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file
for details.
