use clap::{Parser, Subcommand, ValueEnum};
use docata::{Error, OutputFormat, RelationKind};
use std::io;
use std::path::Path;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliOutputFormat {
    #[value(name = "text")]
    Text,
    #[value(name = "json")]
    Json,
}

impl From<CliOutputFormat> for OutputFormat {
    fn from(value: CliOutputFormat) -> Self {
        match value {
            CliOutputFormat::Text => Self::Text,
            CliOutputFormat::Json => Self::Json,
        }
    }
}

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build {
        #[arg(default_value = "./docs")]
        dir: String,
        #[arg(default_value = "./docs/catalog.json")]
        out_dir: String,
    },
    Deps {
        id: String,
        #[arg(default_value = "./docs/catalog.json")]
        catalog: String,
        #[arg(value_enum, long, default_value_t = CliOutputFormat::Json)]
        format: CliOutputFormat,
    },
    Refs {
        id: String,
        #[arg(default_value = "./docs/catalog.json")]
        catalog: String,
        #[arg(value_enum, long, default_value_t = CliOutputFormat::Text)]
        format: CliOutputFormat,
    },
}

/// Run the CLI.
///
/// # Errors
///
/// Returns `Error` when reading catalog files, writing catalog files, or
/// serializing output fails.
pub fn run() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { dir, out_dir } => {
            let dir = Path::new(&dir);
            let out_dir = Path::new(&out_dir);
            let mut file = std::fs::File::create(out_dir)?;
            docata::build_catalog(dir, &mut file)
        },
        Commands::Deps {
            id,
            catalog,
            format,
        } => {
            let mut stdout = io::stdout().lock();
            docata::query_catalog_relation(
                &id,
                Path::new(&catalog),
                RelationKind::Deps,
                format.into(),
                &mut stdout,
            )
        },
        Commands::Refs {
            id,
            catalog,
            format,
        } => {
            let mut stdout = io::stdout().lock();
            docata::query_catalog_relation(
                &id,
                Path::new(&catalog),
                RelationKind::Refs,
                format.into(),
                &mut stdout,
            )
        },
    }
}
