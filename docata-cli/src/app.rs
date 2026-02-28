use clap::{Parser, Subcommand, ValueEnum};
use docata::{BuildOptions, Error, OutputFormat, QueryOptions, RelationKind};
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
        #[arg(long)]
        with_node_metadata: bool,
    },
    Check {
        #[arg(default_value = "./docs")]
        dir: String,
        #[arg(long)]
        catalog: Option<String>,
        #[arg(long)]
        with_node_metadata: bool,
    },
    Deps {
        id: String,
        #[arg(default_value = "./docs/catalog.json")]
        catalog: String,
        #[arg(value_enum, long, default_value_t = CliOutputFormat::Json)]
        format: CliOutputFormat,
        #[arg(long)]
        strict: bool,
    },
    Refs {
        id: String,
        #[arg(default_value = "./docs/catalog.json")]
        catalog: String,
        #[arg(value_enum, long, default_value_t = CliOutputFormat::Text)]
        format: CliOutputFormat,
        #[arg(long)]
        strict: bool,
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
        Commands::Build {
            dir,
            out_dir,
            with_node_metadata,
        } => {
            let dir = Path::new(&dir);
            let out_dir = Path::new(&out_dir);
            let mut file = std::fs::File::create(out_dir)?;
            docata::build_catalog_with_options(
                dir,
                &mut file,
                BuildOptions {
                    include_node_metadata: with_node_metadata,
                },
            )
        },
        Commands::Check {
            dir,
            catalog,
            with_node_metadata,
        } => {
            let dir = Path::new(&dir);
            let options = BuildOptions {
                include_node_metadata: with_node_metadata,
            };

            if let Some(catalog) = catalog {
                docata::check_catalog(dir, Path::new(&catalog), options)
            } else {
                docata::check_catalog_structure(dir)
            }
        },
        Commands::Deps {
            id,
            catalog,
            format,
            strict,
        } => {
            let mut stdout = io::stdout().lock();
            docata::query_catalog_relation_with_options(
                &id,
                Path::new(&catalog),
                RelationKind::Deps,
                format.into(),
                QueryOptions { strict },
                &mut stdout,
            )
        },
        Commands::Refs {
            id,
            catalog,
            format,
            strict,
        } => {
            let mut stdout = io::stdout().lock();
            docata::query_catalog_relation_with_options(
                &id,
                Path::new(&catalog),
                RelationKind::Refs,
                format.into(),
                QueryOptions { strict },
                &mut stdout,
            )
        },
    }
}
