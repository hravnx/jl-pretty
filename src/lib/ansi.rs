#[doc(hidden)]
#[macro_export]
macro_rules! csi {
    () => {
        "\x1b[m"
    };

    ($cmd:literal, $params:literal) => {
        concat!("\x1b[", $cmd, $params, "m")
    };
}

/// Macro to generate ANSI color codes at compile time
///
/// See https://en.wikipedia.org/wiki/ANSI_escape_code for details
///
#[macro_export]
macro_rules! ansi_color {
    // reset
    () => {
        $crate::csi!()
    };
    // set foreground color from 256 color mode
    (fg: $n:literal) => {
        $crate::csi!("38;5;", $n)
    };
    // set background color from 256 color mode
    (bg: $n:literal) => {
        $crate::csi!("48;5;", $n)
    };
}

// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {

    #[test]
    fn test_ansi_color_reset() {
        assert_eq!(ansi_color!(), "\x1b[m");
    }

    #[test]
    fn test_ansi_color_background() {
        const BG_FMT: &str = ansi_color!(bg: 10);
        assert_eq!(BG_FMT, "\x1b[48;5;10m");
    }

    #[test]
    fn test_ansi_color_foreground() {
        const FG_FMT: &str = ansi_color!(fg: 5);
        assert_eq!(FG_FMT, "\x1b[38;5;5m");
    }
}
