use clap::{Parser, Subcommand, ValueEnum};
use docata::catalog::Catalog;
use docata::error::Error;
use docata::graph::Graph;
use docata::scan::scan;
use serde::Serialize;
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::path::Path;

#[derive(Clone, Debug, ValueEnum)]
enum OutputFormat {
    #[value(name = "text")]
    Text,
    #[value(name = "json")]
    Json,
}

#[derive(Debug, Serialize)]
struct RelationItem {
    id: String,
    path: Option<String>,
    resolved: bool,
}

#[derive(Debug, Serialize)]
struct RelationMeta {
    missing_nodes: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RelationResponse {
    command: String,
    query_id: String,
    source_dir: String,
    count: usize,
    items: Vec<RelationItem>,
    meta: RelationMeta,
}

#[derive(Debug)]
enum RelationMode {
    Deps,
    Refs,
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
        #[arg(default_value = "./docs")]
        dir: String,
        #[arg(value_enum, long, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
    Refs {
        id: String,
        #[arg(default_value = "./docs")]
        dir: String,
        #[arg(value_enum, long, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
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
        Commands::Deps { id, dir, format } => {
            run_relation_command(&id, Path::new(&dir), RelationMode::Deps, format)
        },
        Commands::Refs { id, dir, format } => {
            run_relation_command(&id, Path::new(&dir), RelationMode::Refs, format)
        },
    }
}

fn run_relation_command(
    id: &str,
    dir: &Path,
    mode: RelationMode,
    format: OutputFormat,
) -> Result<(), Error> {
    let entries = scan(dir)?;
    let catalog = Catalog::from_entries(&entries);
    let graph = Graph::from_catalog(&catalog);

    let mut ids = match mode {
        RelationMode::Deps => graph.deps(id),
        RelationMode::Refs => graph.refs(id),
    };
    ids.sort();
    ids.dedup();

    match format {
        OutputFormat::Text => {
            for id in ids {
                println!("{id}");
            }
            Ok(())
        },
        OutputFormat::Json => {
            let node_paths = catalog
                .nodes
                .iter()
                .map(|node| (node.id.as_str(), node.path.as_str()))
                .collect::<HashMap<_, _>>();

            let mut missing_nodes = Vec::new();
            let mut items = Vec::with_capacity(ids.len());

            for id in ids {
                if let Some(path) = node_paths.get(id.as_str()) {
                    items.push(RelationItem {
                        id,
                        path: Some((*path).to_owned()),
                        resolved: true,
                    });
                } else {
                    missing_nodes.push(id.clone());
                    items.push(RelationItem {
                        id,
                        path: None,
                        resolved: false,
                    });
                }
            }

            missing_nodes.sort();

            let response = RelationResponse {
                command: match mode {
                    RelationMode::Deps => String::from("deps"),
                    RelationMode::Refs => String::from("refs"),
                },
                query_id: id.to_owned(),
                source_dir: dir.to_string_lossy().to_string(),
                count: items.len(),
                items,
                meta: RelationMeta { missing_nodes },
            };

            let mut stdout = io::stdout().lock();
            serde_json::to_writer_pretty(&mut stdout, &response)?;
            writeln!(&mut stdout)?;

            Ok(())
        },
    }
}
