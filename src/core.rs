use std::collections::HashSet;

use color_eyre::eyre::{ContextCompat, Result};
use sysinfo::System;

/// Attempts to find the PID of the main Spotify process
///
/// # Errors
///
/// This function will return an error if more than one possible main Spotify process is identified.
pub fn find_spotify_process() -> Result<Option<sysinfo::Pid>> {
    // There are a whole bunch of Spotify processes
    let (processes, parent_processes): (HashSet<_>, HashSet<_>) = System::new_all()
        .processes_by_exact_name("Spotify.exe".as_ref())
        .filter_map(|process| process.parent().map(|parent| (process.pid(), parent)))
        .unzip();

    // One of the processes is the parent of the others, though. I think that's the one we want
    let mut parents = processes.intersection(&parent_processes);
    let main_process = parents.next();
    (parents.count() == 0)
        .then_some(main_process.copied())
        .context("Multiple possible main processes found!")
}
