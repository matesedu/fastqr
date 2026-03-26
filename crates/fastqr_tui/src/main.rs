use std::{fmt, fs, io::IsTerminal, path::Path};

use fastqr_core::{EncodeOptions, ErrorCorrectionLevel, encode_text};
use fastqr_image::{RasterFormat, RenderOptions, decode_file, write_to_bytes, write_to_path};
use fastqr_tui::{TuiRenderOptions, render_to_ansi_string, render_to_string};

fn main() {
    if let Err(error) = run() {
        if error.exit_code == 0 {
            print!("{}", error.message);
        } else {
            eprintln!("{}", error.message);
        }
        std::process::exit(error.exit_code);
    }
}

fn run() -> Result<(), CliError> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        None | Some("help") | Some("-h") | Some("--help") => {
            print!("{}", usage());
            Ok(())
        }
        Some("version") | Some("-V") | Some("--version") => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("encode") => execute_encode(parse_encode(args)?),
        Some("render") => execute_render(parse_render(args)?),
        Some("decode") => execute_decode(parse_decode(args)?),
        Some(command) => Err(CliError::usage(format!(
            "unknown command `{command}`\n\n{}",
            usage()
        ))),
    }
}

#[derive(Debug)]
struct CliError {
    exit_code: i32,
    message: String,
}

impl CliError {
    fn usage(message: impl Into<String>) -> Self {
        Self {
            exit_code: 2,
            message: message.into(),
        }
    }

    fn help(message: impl Into<String>) -> Self {
        Self {
            exit_code: 0,
            message: message.into(),
        }
    }

    fn command(message: impl Into<String>) -> Self {
        Self {
            exit_code: 1,
            message: message.into(),
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CliError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputFormat {
    Png,
    Jpeg,
    WebP,
    Svg,
}

impl OutputFormat {
    fn from_str(value: &str) -> Result<Self, CliError> {
        match value.to_ascii_lowercase().as_str() {
            "png" => Ok(Self::Png),
            "jpg" | "jpeg" => Ok(Self::Jpeg),
            "webp" | "wep" => Ok(Self::WebP),
            "svg" => Ok(Self::Svg),
            _ => Err(CliError::usage(format!(
                "unknown output format `{value}`\n\n{}",
                usage_render()
            ))),
        }
    }

    fn to_raster_format(self) -> Result<RasterFormat, CliError> {
        match self {
            Self::Png => Ok(RasterFormat::Png),
            Self::Jpeg => Ok(RasterFormat::Jpeg),
            Self::WebP => Ok(RasterFormat::WebP),
            Self::Svg => Err(CliError::usage(
                "svg output is not a raster format and must be handled separately",
            )),
        }
    }
}

#[derive(Debug)]
struct EncodeCommand {
    text: String,
    error_correction: Option<ErrorCorrectionLevel>,
    quiet_zone: usize,
    invert: bool,
}

#[derive(Debug)]
struct RenderCommand {
    text: String,
    output: String,
    error_correction: Option<ErrorCorrectionLevel>,
    scale: u32,
    border: u32,
    format: Option<OutputFormat>,
}

#[derive(Debug)]
struct DecodeCommand {
    path: String,
}

fn execute_encode(command: EncodeCommand) -> Result<(), CliError> {
    let code = encode_text(&command.text, encode_options(command.error_correction))
        .map_err(to_command_error)?;
    let options = TuiRenderOptions {
        quiet_zone: command.quiet_zone,
        invert: command.invert,
    };
    let rendered = if std::io::stdout().is_terminal() {
        render_to_ansi_string(&code, options)
    } else {
        render_to_string(&code, options)
    };
    print!("{rendered}");
    Ok(())
}

fn execute_render(command: RenderCommand) -> Result<(), CliError> {
    let code = encode_text(&command.text, encode_options(command.error_correction))
        .map_err(to_command_error)?;
    let format = resolve_output_format(&command.output, command.format)?;

    if format == OutputFormat::Svg {
        fs::write(&command.output, code.to_svg_string(command.border as usize))
            .map_err(to_command_error)?;
        return Ok(());
    }

    let render = RenderOptions {
        scale: command.scale,
        border: command.border,
        ..RenderOptions::default()
    };

    if command.format.is_some() {
        let bytes =
            write_to_bytes(&code, format.to_raster_format()?, render).map_err(to_command_error)?;
        fs::write(&command.output, bytes).map_err(to_command_error)?;
    } else {
        write_to_path(&code, &command.output, render).map_err(to_command_error)?;
    }
    Ok(())
}

fn execute_decode(command: DecodeCommand) -> Result<(), CliError> {
    let decoded = decode_file(&command.path).map_err(to_command_error)?;
    if let Some(text) = decoded.text {
        println!("{text}");
    } else {
        println!("{}", bytes_to_hex(&decoded.bytes));
    }
    Ok(())
}

fn parse_encode(args: impl Iterator<Item = String>) -> Result<EncodeCommand, CliError> {
    let mut text = None;
    let mut error_correction = None;
    let mut quiet_zone = TuiRenderOptions::default().quiet_zone;
    let mut invert = false;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Err(CliError::help(usage_encode())),
            "--ecc" => {
                let value = next_value(&mut args, "--ecc", usage_encode())?;
                error_correction = Some(parse_ecc(&value)?);
            }
            "--quiet-zone" => {
                let value = next_value(&mut args, "--quiet-zone", usage_encode())?;
                quiet_zone = parse_usize(&value, "--quiet-zone", usage_encode())?;
            }
            "--invert" => invert = true,
            _ if text.is_none() => text = Some(arg),
            _ => {
                return Err(CliError::usage(format!(
                    "unexpected argument `{arg}`\n\n{}",
                    usage_encode()
                )));
            }
        }
    }

    Ok(EncodeCommand {
        text: text.ok_or_else(|| CliError::usage(usage_encode()))?,
        error_correction,
        quiet_zone,
        invert,
    })
}

fn parse_render(args: impl Iterator<Item = String>) -> Result<RenderCommand, CliError> {
    let mut text = None;
    let mut output = None;
    let mut error_correction = None;
    let mut scale = RenderOptions::default().scale;
    let mut border = RenderOptions::default().border;
    let mut format = None;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Err(CliError::help(usage_render())),
            "--ecc" => {
                let value = next_value(&mut args, "--ecc", usage_render())?;
                error_correction = Some(parse_ecc(&value)?);
            }
            "--scale" => {
                let value = next_value(&mut args, "--scale", usage_render())?;
                scale = parse_u32(&value, "--scale", usage_render())?;
            }
            "--border" => {
                let value = next_value(&mut args, "--border", usage_render())?;
                border = parse_u32(&value, "--border", usage_render())?;
            }
            "--format" => {
                let value = next_value(&mut args, "--format", usage_render())?;
                format = Some(OutputFormat::from_str(&value)?);
            }
            _ if text.is_none() => text = Some(arg),
            _ if output.is_none() => output = Some(arg),
            _ => {
                return Err(CliError::usage(format!(
                    "unexpected argument `{arg}`\n\n{}",
                    usage_render()
                )));
            }
        }
    }

    Ok(RenderCommand {
        text: text.ok_or_else(|| CliError::usage(usage_render()))?,
        output: output.ok_or_else(|| CliError::usage(usage_render()))?,
        error_correction,
        scale,
        border,
        format,
    })
}

fn parse_decode(args: impl Iterator<Item = String>) -> Result<DecodeCommand, CliError> {
    let mut path = None;
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => return Err(CliError::help(usage_decode())),
            _ if path.is_none() => path = Some(arg),
            _ => {
                return Err(CliError::usage(format!(
                    "unexpected argument `{arg}`\n\n{}",
                    usage_decode()
                )));
            }
        }
    }

    Ok(DecodeCommand {
        path: path.ok_or_else(|| CliError::usage(usage_decode()))?,
    })
}

fn next_value(
    args: &mut std::iter::Peekable<impl Iterator<Item = String>>,
    flag: &str,
    usage: &'static str,
) -> Result<String, CliError> {
    args.next()
        .ok_or_else(|| CliError::usage(format!("missing value for `{flag}`\n\n{usage}")))
}

fn parse_ecc(value: &str) -> Result<ErrorCorrectionLevel, CliError> {
    match value.to_ascii_uppercase().as_str() {
        "L" | "LOW" => Ok(ErrorCorrectionLevel::Low),
        "M" | "MEDIUM" => Ok(ErrorCorrectionLevel::Medium),
        "Q" | "QUARTILE" => Ok(ErrorCorrectionLevel::Quartile),
        "H" | "HIGH" => Ok(ErrorCorrectionLevel::High),
        _ => Err(CliError::usage(format!(
            "unknown error correction level `{value}`\nexpected one of: L, M, Q, H"
        ))),
    }
}

fn parse_u32(value: &str, flag: &str, usage: &'static str) -> Result<u32, CliError> {
    value
        .parse::<u32>()
        .map_err(|_| CliError::usage(format!("invalid value for `{flag}`: `{value}`\n\n{usage}")))
}

fn parse_usize(value: &str, flag: &str, usage: &'static str) -> Result<usize, CliError> {
    value
        .parse::<usize>()
        .map_err(|_| CliError::usage(format!("invalid value for `{flag}`: `{value}`\n\n{usage}")))
}

fn encode_options(error_correction: Option<ErrorCorrectionLevel>) -> EncodeOptions {
    let mut options = EncodeOptions::default();
    if let Some(level) = error_correction {
        options.min_error_correction = level;
    }
    options
}

fn resolve_output_format(
    output: &str,
    explicit: Option<OutputFormat>,
) -> Result<OutputFormat, CliError> {
    match explicit {
        Some(format) => Ok(format),
        None => infer_output_format(output).ok_or_else(|| {
            CliError::usage(format!(
                "could not infer output format from `{output}`\n\n{}",
                usage_render()
            ))
        }),
    }
}

fn infer_output_format(path: &str) -> Option<OutputFormat> {
    let extension = Path::new(path).extension()?.to_str()?;
    match extension.to_ascii_lowercase().as_str() {
        "png" => Some(OutputFormat::Png),
        "jpg" | "jpeg" => Some(OutputFormat::Jpeg),
        "webp" | "wep" => Some(OutputFormat::WebP),
        "svg" => Some(OutputFormat::Svg),
        _ => None,
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn to_command_error(error: impl ToString) -> CliError {
    CliError::command(error.to_string())
}

fn usage() -> &'static str {
    "usage:
  fastqr encode <text> [--ecc <L|M|Q|H>] [--quiet-zone <modules>] [--invert]
  fastqr render <text> <output> [--format <png|jpeg|webp|svg>] [--ecc <L|M|Q|H>] [--scale <pixels>] [--border <modules>]
  fastqr decode <image-path>
  fastqr help
  fastqr version
"
}

fn usage_encode() -> &'static str {
    "usage:
  fastqr encode <text> [--ecc <L|M|Q|H>] [--quiet-zone <modules>] [--invert]
"
}

fn usage_render() -> &'static str {
    "usage:
  fastqr render <text> <output> [--format <png|jpeg|webp|svg>] [--ecc <L|M|Q|H>] [--scale <pixels>] [--border <modules>]
"
}

fn usage_decode() -> &'static str {
    "usage:
  fastqr decode <image-path>
"
}

#[cfg(test)]
mod tests {
    use super::{
        OutputFormat, bytes_to_hex, infer_output_format, parse_decode, parse_ecc, parse_encode,
        parse_render,
    };
    use fastqr_core::ErrorCorrectionLevel;

    #[test]
    fn infers_output_format_from_extension() {
        assert_eq!(infer_output_format("code.png"), Some(OutputFormat::Png));
        assert_eq!(infer_output_format("code.webp"), Some(OutputFormat::WebP));
        assert_eq!(infer_output_format("code.svg"), Some(OutputFormat::Svg));
        assert_eq!(infer_output_format("code.unknown"), None);
    }

    #[test]
    fn encodes_bytes_as_hex() {
        assert_eq!(bytes_to_hex(&[0x00, 0x7f, 0xab, 0xff]), "007fabff");
    }

    #[test]
    fn parses_encode_command_options() {
        let command = parse_encode(
            ["hello", "--ecc", "H", "--quiet-zone", "4", "--invert"]
                .into_iter()
                .map(str::to_owned),
        )
        .expect("parses");
        assert_eq!(command.text, "hello");
        assert_eq!(command.error_correction, Some(ErrorCorrectionLevel::High));
        assert_eq!(command.quiet_zone, 4);
        assert!(command.invert);
    }

    #[test]
    fn parses_render_command_options() {
        let command = parse_render(
            [
                "hello", "code.svg", "--format", "svg", "--ecc", "Q", "--scale", "12", "--border",
                "3",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .expect("parses");
        assert_eq!(command.text, "hello");
        assert_eq!(command.output, "code.svg");
        assert_eq!(command.format, Some(OutputFormat::Svg));
        assert_eq!(
            command.error_correction,
            Some(ErrorCorrectionLevel::Quartile)
        );
        assert_eq!(command.scale, 12);
        assert_eq!(command.border, 3);
    }

    #[test]
    fn parses_decode_command() {
        let command = parse_decode(["code.png"].into_iter().map(str::to_owned)).expect("parses");
        assert_eq!(command.path, "code.png");
    }

    #[test]
    fn parses_error_correction_aliases() {
        assert_eq!(parse_ecc("low").expect("parses"), ErrorCorrectionLevel::Low);
        assert_eq!(
            parse_ecc("quartile").expect("parses"),
            ErrorCorrectionLevel::Quartile
        );
    }
}
