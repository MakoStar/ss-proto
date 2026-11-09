mod convert;
mod emitter;
mod model;
mod parser;
mod raw;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use logger::{init_logger, succ};

use model::Syntax;

#[derive(Parser)]
#[command(
    name = "ss-proto",
    version,
    about = "Reverse-compile a .pb into .proto"
)]
struct Cli {
    #[arg(
        short = 'i',
        long,
        help = "Input .pb file (FileDescriptorSet or FileDescriptorProto)"
    )]
    input: PathBuf,

    #[arg(short = 'o', long, help = "Single-file output .proto path")]
    output: Option<PathBuf>,

    #[arg(
        long,
        default_value = "output",
        help = "Output directory for per-file mode"
    )]
    out_dir: PathBuf,

    #[arg(long, help = "Force single-file mode even without -o")]
    single: bool,

    #[arg(long, help = "Override syntax (proto2 / proto3)")]
    syntax: Option<String>,

    #[arg(long, help = "Override package name in single-file mode")]
    package: Option<String>,

    #[arg(long, help = "Force generic raw protobuf decoding")]
    raw: bool,

    #[arg(long, help = "Also print the result to stdout")]
    stdout: bool,
}

// 程序入口
fn main() -> Result<()> {
    init_logger!();
    let cli = Cli::parse();
    let data = std::fs::read(&cli.input)?;

    let source = cli
        .input
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| cli.input.display().to_string());

    let files = if cli.raw {
        vec![raw::parse_raw(&source, &data)]
    } else {
        match parser::parse(&data)? {
            parser::Parsed::Set(set) => convert::convert_set(set.as_ref()),
            parser::Parsed::Single(fd) => vec![convert::convert_file(fd.as_ref())],
        }
    };

    let syntax_override = match cli.syntax.as_deref() {
        Some("proto2") => Some(Syntax::Proto2),
        Some("proto3") => Some(Syntax::Proto3),
        _ => None,
    };

    if cli.output.is_some() || cli.single {
        let out_path = cli.output.clone().unwrap_or_else(|| {
            let stem = cli
                .input
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "output".to_string());
            PathBuf::from("output").join(format!("{}.proto", stem))
        });
        let text =
            emitter::emit_single_file(&files, syntax_override, cli.package.as_deref(), &source);
        if cli.stdout {
            print!("{}", text);
        }
        if out_path.to_string_lossy() != "-" {
            if let Some(parent) = out_path.parent()
                && !parent.as_os_str().is_empty()
            {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&out_path, text)?;
            succ!("Wrote single-file proto: {}", out_path.display());
        }
    } else {
        std::fs::create_dir_all(&cli.out_dir)?;
        for f in &files {
            let p = cli.out_dir.join(if f.name.is_empty() {
                "output.proto".to_string()
            } else {
                f.name.clone()
            });
            if let Some(parent) = p.parent()
                && !parent.as_os_str().is_empty()
            {
                std::fs::create_dir_all(parent)?;
            }
            let text = emitter::emit_file(f, syntax_override, &source);
            std::fs::write(&p, text)?;
            succ!("Wrote {}", p.display());
        }
        succ!(
            "Generated {} .proto file(s) into {}",
            files.len(),
            cli.out_dir.display()
        );
    }

    Ok(())
}
