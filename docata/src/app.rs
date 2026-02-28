use crate::build;
use crate::catalog::Catalog;
use crate::catalog_presentation;
use crate::error::Error;
use crate::format::OutputFormat;
use crate::graph::Graph;
use crate::relation;
use clap::{Parser, Subcommand};
use std::io;
use std::path::Path;

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
        #[arg(value_enum, long, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
    Refs {
        id: String,
        #[arg(default_value = "./docs/catalog.json")]
        catalog: String,
        #[arg(value_enum, long, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
}

fn load_index(catalog_path: &Path) -> Result<(Catalog, Graph), Error> {
    let mut file = std::fs::File::open(catalog_path)?;
    let catalog = catalog_presentation::read_catalog(&mut file)?;
    let graph = Graph::from_catalog(&catalog);

    Ok((catalog, graph))
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
            build::run(dir, &mut file)
        },
        Commands::Deps {
            id,
            catalog,
            format,
        } => {
            let catalog = Path::new(&catalog);
            let (catalog, graph) = load_index(catalog)?;

            let mut stdout = io::stdout().lock();
            relation::run(
                &id,
                &catalog,
                &graph,
                relation::RelationKind::Deps,
                format,
                &mut stdout,
            )
        },
        Commands::Refs {
            id,
            catalog,
            format,
        } => {
            let catalog = Path::new(&catalog);
            let (catalog, graph) = load_index(catalog)?;

            let mut stdout = io::stdout().lock();
            relation::run(
                &id,
                &catalog,
                &graph,
                relation::RelationKind::Refs,
                format,
                &mut stdout,
            )
        },
    }
}
