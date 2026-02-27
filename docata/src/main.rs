use clap::{Parser, Subcommand};
use docata::catalog::Catalog;
use docata::error::Error;
use docata::graph::Graph;
use docata::scan::scan;
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
        #[arg(default_value = "./docs")]
        dir: String,
    },
    Refs {
        id: String,
        #[arg(default_value = "./docs")]
        dir: String,
    },
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { dir, out_dir } => {
            let dir = Path::new(&dir);
            let out_dir = Path::new(&out_dir);

            let entries = scan(dir)?;
            let () = Catalog::from_entries(&entries).write(out_dir)?;

            Ok(())
        },
        Commands::Deps { id, dir } => {
            let dir = Path::new(&dir);

            let entries = scan(dir)?;
            let catalog = Catalog::from_entries(&entries);
            let graph = Graph::from_catalog(&catalog);

            for dep in graph.deps(&id) {
                println!("{dep}");
            }

            Ok(())
        },
        Commands::Refs { id, dir } => {
            let dir = Path::new(&dir);

            let entries = scan(dir)?;
            let catalog = Catalog::from_entries(&entries);
            let graph = Graph::from_catalog(&catalog);

            for r#ref in graph.refs(&id) {
                println!("{ref}");
            }

            Ok(())
        },
    }
}
