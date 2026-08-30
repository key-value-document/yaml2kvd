//! `yaml2kvd` — convert a YAML document to KVD (spec §4 grammar).

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

/// Convert YAML documents to KVD — and KVD to YAML with --reverse.
///
/// Without --reverse, reads YAML and writes KVD to stdout.
/// With --schema, the YAML schema is converted to a KVD schema node and
/// the converted document is verified against it before emitting.
/// With --reverse, reads KVD and writes YAML to stdout.
#[derive(Parser, Debug)]
#[command(name = "yaml2kvd", version, about)]
struct Cli {
    /// Input file (YAML by default, KVD when --reverse is set)
    input: PathBuf,

    /// YAML schema file to verify against (only for YAML → KVD)
    #[arg(long, value_name = "FILE", conflicts_with = "reverse")]
    schema: Option<PathBuf>,

    /// Reverse direction: read KVD and write YAML
    #[arg(long)]
    reverse: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(&cli.input, cli.schema.as_deref(), cli.reverse) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("yaml2kvd: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(
    path: &std::path::Path,
    schema_path: Option<&std::path::Path>,
    reverse: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(path)?;
    if reverse {
        // KVD → YAML
        print!("{}", yaml2kvd::kvd_text_to_yaml(&text)?);
    } else {
        // YAML → KVD
        let schema_text = schema_path.map(std::fs::read_to_string).transpose()?;
        print!(
            "{}",
            yaml2kvd::yaml_text_to_kvd(&text, schema_text.as_deref())?
        );
    }
    Ok(())
}
