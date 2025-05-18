use color_eyre::{eyre::ContextCompat, Result};

use swivels::core;

fn main() -> Result<()> {
    color_eyre::install()?;

    let spotify_process = core::find_spotify_process()
        .and_then(|pid| pid.context("No Spotify process identified."))?;
    Ok(())
}
