use clap::ValueEnum;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum OutputFormat {
    #[value(name = "text")]
    Text,
    #[value(name = "json")]
    Json,
}
