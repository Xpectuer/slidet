use anyhow::{Context, Result};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "slidet")]
#[command(about = "A terminal markdown slide player")]
#[command(version)]
struct Cli {
    slides_dir: std::path::PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let slides = slidet::loader::load_slides(&cli.slides_dir)
        .with_context(|| format!("failed to load slides from {}", cli.slides_dir.display()))?;

    let mut terminal = slidet::ui::init_terminal()?;
    let result = slidet::app::run(&mut terminal, slides);
    slidet::ui::restore_terminal()?;
    result
}
