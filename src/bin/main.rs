use std::collections::HashSet;

use color_eyre::eyre::{self, Result};
use sysinfo::System;

fn find_spotify_process() -> Result<Option<sysinfo::Pid>> {
    // There are a whole bunch of Spotify processes
    let (processes, parent_processes): (HashSet<_>, HashSet<_>) = System::new_all()
        .processes_by_exact_name("Spotify.exe".as_ref())
        .filter_map(|process| process.parent().map(|parent| (process.pid(), parent)))
        .unzip();

    // One of the processes is the parent of the others, though. I think that's the one we want
    let mut parents = processes.intersection(&parent_processes);
    let main_process = parents.next();
    if parents.count() == 0 {
        Ok(main_process.copied())
    } else {
        eyre::bail!("Multiple possible main processes found!");
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let spotify_process = find_spotify_process();
    dbg!(spotify_process);
    Ok(())
}
