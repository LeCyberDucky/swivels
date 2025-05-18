use std::io::Write;

use swivels::core;
use color_eyre::eyre::{Context, ContextCompat, Result, eyre};
use wasapi::initialize_mta;

fn main() -> Result<()> {
    color_eyre::install()?;

    let spotify_process = core::find_spotify_process()
        .and_then(|pid| pid.context("No Spotify process identified."))?;

    let (spotify_sender, spotify_receiver): (
        std::sync::mpsc::SyncSender<Vec<u8>>,
        std::sync::mpsc::Receiver<Vec<u8>>,
    ) = std::sync::mpsc::sync_channel(2);
    let chunksize = 4096;

    // Capture
    let listening_thread = std::thread::Builder::new().spawn(move || -> Result<()> {
        let result = capture_loop(spotify_sender, chunksize, spotify_process)
            .context("Failed to capture Spotify")?;
        Ok(())
    });

    let mut outfile = std::fs::File::create("spotify.raw")?;

    loop {
        match spotify_receiver.recv() {
            Ok(chunk) => {
                outfile.write_all(&chunk)?;
            }
            Err(err) => {
                eyre!("Some error {}", err);
                return Ok(());
            }
        }
    }

    Ok(())
}

fn capture_loop(
    receiver: std::sync::mpsc::SyncSender<Vec<u8>>,
    chunksize: usize,
    process: sysinfo::Pid,
) -> Result<()> {
    initialize_mta().ok()?;

    // Settings
    let desired_format =
        wasapi::WaveFormat::new(32, 32, &wasapi::SampleType::Float, 48000, 2, None);
    let blockalign = desired_format.get_blockalign();
    let autoconvert = true;
    let include_tree = true;

    let mut audio_client =
        wasapi::AudioClient::new_application_loopback_client(process.as_u32(), include_tree)?;
    let mode = wasapi::StreamMode::EventsShared {
        autoconvert,
        buffer_duration_hns: 0,
    };
    audio_client.initialize_client(&desired_format, &wasapi::Direction::Capture, &mode)?;

    let h_event = audio_client.set_get_eventhandle()?;

    let capture_client = audio_client.get_audiocaptureclient().unwrap();

    // just eat the reallocation because querying the buffer size gives massive values.
    let mut sample_queue: std::collections::VecDeque<u8> = std::collections::VecDeque::new();

    audio_client.start_stream()?;

    loop {
        while sample_queue.len() > (blockalign as usize * chunksize) {
            let mut chunk = vec![0u8; blockalign as usize * chunksize];
            for element in chunk.iter_mut() {
                *element = sample_queue.pop_front().unwrap();
            }
            receiver.send(chunk).unwrap();
        }

        let new_frames = capture_client.get_next_packet_size()?.unwrap_or(0);
        let additional = (new_frames as usize * blockalign as usize)
            .saturating_sub(sample_queue.capacity() - sample_queue.len());
        sample_queue.reserve(additional);
        if new_frames > 0 {
            capture_client
                .read_from_device_to_deque(&mut sample_queue)
                .unwrap();
        }
        if h_event.wait_for_event(3000).is_err() {
            eyre!("timeout error, stopping capture");
            audio_client.stop_stream().unwrap();
            break;
        }
    }
    Ok(())
}
