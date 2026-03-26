use fastqr_core::QrCode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TuiRenderOptions {
    pub quiet_zone: usize,
    pub invert: bool,
}

impl Default for TuiRenderOptions {
    fn default() -> Self {
        Self {
            quiet_zone: 2,
            invert: false,
        }
    }
}

pub fn render_to_string(code: &QrCode, options: TuiRenderOptions) -> String {
    let width = code.size() + options.quiet_zone * 2;
    let rows = width.div_ceil(2);
    let mut out = String::with_capacity(rows * (width + 1));
    for row in (0..width).step_by(2) {
        for column in 0..width {
            let top = module_with_border(code, column, row, options.quiet_zone);
            let bottom = module_with_border(code, column, row + 1, options.quiet_zone);
            let top = if options.invert { !top } else { top };
            let bottom = if options.invert { !bottom } else { bottom };
            let ch = match (top, bottom) {
                (false, false) => ' ',
                (true, false) => '▀',
                (false, true) => '▄',
                (true, true) => '█',
            };
            out.push(ch);
        }
        out.push('\n');
    }
    out
}

pub fn render_to_ansi_string(code: &QrCode, options: TuiRenderOptions) -> String {
    let width = code.size() + options.quiet_zone * 2;
    let rows = width.div_ceil(2);
    let mut out = String::with_capacity(rows * (width * 8 + 5));
    let mut current_style = Style::RESET;

    for row in (0..width).step_by(2) {
        for column in 0..width {
            let top = module_with_border(code, column, row, options.quiet_zone);
            let bottom = module_with_border(code, column, row + 1, options.quiet_zone);
            let top = if options.invert { !top } else { top };
            let bottom = if options.invert { !bottom } else { bottom };
            let cell = ansi_cell(top, bottom);
            if cell.style != current_style {
                out.push_str(cell.style.escape());
                current_style = cell.style;
            }
            out.push(cell.ch);
        }
        if current_style != Style::RESET {
            out.push_str(Style::RESET.escape());
            current_style = Style::RESET;
        }
        out.push('\n');
    }

    out
}

fn module_with_border(code: &QrCode, x: usize, y: usize, quiet_zone: usize) -> bool {
    if x < quiet_zone
        || y < quiet_zone
        || x >= code.size() + quiet_zone
        || y >= code.size() + quiet_zone
    {
        return false;
    }
    code.module(x - quiet_zone, y - quiet_zone)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Style {
    fg: Option<Color>,
    bg: Option<Color>,
}

impl Style {
    const RESET: Self = Self { fg: None, bg: None };

    fn escape(self) -> &'static str {
        match (self.fg, self.bg) {
            (None, None) => "\x1b[0m",
            (Some(Color::Black), Some(Color::White)) => "\x1b[30;47m",
            (Some(Color::White), Some(Color::Black)) => "\x1b[37;40m",
            (None, Some(Color::Black)) => "\x1b[40m",
            (None, Some(Color::White)) => "\x1b[47m",
            (Some(Color::Black), Some(Color::Black)) => "\x1b[30;40m",
            (Some(Color::White), Some(Color::White)) => "\x1b[37;47m",
            (Some(Color::Black), None) => "\x1b[30m",
            (Some(Color::White), None) => "\x1b[37m",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Color {
    Black,
    White,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AnsiCell {
    ch: char,
    style: Style,
}

fn ansi_cell(top_dark: bool, bottom_dark: bool) -> AnsiCell {
    match (top_dark, bottom_dark) {
        (true, true) => AnsiCell {
            ch: ' ',
            style: Style {
                fg: None,
                bg: Some(Color::Black),
            },
        },
        (false, false) => AnsiCell {
            ch: ' ',
            style: Style {
                fg: None,
                bg: Some(Color::White),
            },
        },
        (true, false) => AnsiCell {
            ch: '▀',
            style: Style {
                fg: Some(Color::Black),
                bg: Some(Color::White),
            },
        },
        (false, true) => AnsiCell {
            ch: '▀',
            style: Style {
                fg: Some(Color::White),
                bg: Some(Color::Black),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use fastqr_core::{EncodeOptions, encode_text};

    use super::{TuiRenderOptions, render_to_ansi_string, render_to_string};

    #[test]
    fn plain_renderer_does_not_emit_ansi_sequences() {
        let code = encode_text("FASTQR", EncodeOptions::default()).expect("encodes");
        let rendered = render_to_string(&code, TuiRenderOptions::default());
        assert!(!rendered.contains("\x1b["));
    }

    #[test]
    fn ansi_renderer_emits_explicit_black_and_white_cells() {
        let code = encode_text("FASTQR", EncodeOptions::default()).expect("encodes");
        let rendered = render_to_ansi_string(&code, TuiRenderOptions::default());
        assert!(rendered.contains("\x1b[40m") || rendered.contains("\x1b[30;47m"));
        assert!(rendered.contains("\x1b[47m") || rendered.contains("\x1b[37;40m"));
        assert!(rendered.ends_with("\n"));
    }
}
