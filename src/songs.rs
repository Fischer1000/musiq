extern crate rand;
extern crate minimp3;
extern crate cpal;

use std::ffi::OsStr;
use std::io::BufReader;
use std::path::Path;
use std::sync::Mutex;

use rand::{rng, seq::SliceRandom};
use minimp3::{Decoder, Frame};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, StreamConfig, Device};

use crate::database::SongDatabase;
use crate::{logln, or_return};
use crate::Error;
use crate::generated::TARGET_VOLUME;

/// Block a thread while a song is playing with this Mutex
pub static SONG_PLAYING_GATE: Mutex<()> = Mutex::new(());

#[derive(Debug, Eq, Clone)]
pub struct Song {
    filename: Box<OsStr>,
    metadata: u8,
}

impl Song {
    pub fn new(filename: &Path) -> Option<Self> {
        Some(Song { filename: filename.file_name()?.into(), metadata: Self::compose_metadata(false, false) })
    }

    fn compose_metadata(enabled: bool, was_played: bool) -> u8 {
        ((enabled as u8) << 1) | (was_played as u8)
    }

    fn destructure_metadata(data: u8) -> (bool, bool) {
        (data >> 1 != 0, data & 1 != 0)
    }

    #[inline]
    pub fn filename(&self) -> &OsStr {
        &self.filename
    }

    #[inline]
    pub fn enabled(&self) -> bool {
        self.metadata >> 1 != 0
    }

    #[inline]
    pub fn was_played(&self) -> bool {
        self.metadata & 1 != 0
    }

    #[inline]
    pub fn set_played(&mut self, played: bool) {
        if played {
            self.metadata |= 0b_0000_0001;
        } else {
            self.metadata &= 0b_1111_1110;
        }
    }

    #[inline]
    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled {
            self.metadata |= 0b_0000_0010;
        } else {
            self.metadata &= 0b_1111_1101;
        }
    }

    #[inline]
    #[deprecated(since = "0.3.6")]
    pub fn enable(&mut self) {
        self.metadata |= 2
    }

    #[inline]
    #[deprecated(since = "0.3.6")]
    pub fn disable(&mut self) {
        self.metadata &= !2
    }

    /// Plays this song
    /// # Warning this method blocks until the currently playing song is done playing
    pub fn play(&self, device: &Device) -> Result<(), Error> {
        let file_path = Path::new(crate::SONG_FILES_DIR).join(self.filename.as_ref());

        let file = or_return!(std::fs::File::open(&file_path).ok(), Err(Error::CannotReadFile));
        let mut decoder = Decoder::new(BufReader::new(file));
        let mut samples = Vec::new();

        let mut source_sample_rate = 44100;
        let mut source_channels = 2;

        let mut sq_sum = 0.0;

        while let Ok(Frame { data, sample_rate: sr, channels: ch, .. }) = decoder.next_frame() {
            samples.extend(data.iter().map(|&s| {
                let sample = s as f32 / i16::MAX as f32;
                sq_sum += sample * sample;
                sample
            }));
            source_sample_rate = sr;
            source_channels = ch;
        }

        sq_sum /= samples.len() as f32;
        let rms = sq_sum.sqrt();
        let scale_factor = TARGET_VOLUME / rms;

        // Scale all samples according to the calculated volume
        samples.iter_mut().for_each(|s| *s *= scale_factor);

        let mut index: usize = 0;

        let config = or_return!(
            or_return!(
                device.supported_output_configs().ok(),
                Err(Error::OutputDeviceConfigCannotBeQueried)
            ).filter( |conf| {
                conf.channels() as usize == source_channels &&
                match conf.sample_format() {
                    SampleFormat::F32 | SampleFormat::F64 => true,
                    _ => false
                }
            } )
            .max_by_key( |conf| conf.max_sample_rate() ),
            Err(Error::NoOutputDeviceConfigs)
        );

        let config: StreamConfig = or_return!(
            config.try_with_sample_rate(SampleRate(source_sample_rate as u32)),
            Err(Error::OutputDeviceConfigCannotBeSet)
        ).into();

        let duration_secs = samples.len() as f64 / (source_sample_rate as f64 * source_channels as f64);

        // Panic so that panics cascade over threads
        let _guard = SONG_PLAYING_GATE.lock().expect("Song playing guard was poisoned");

        let stream = or_return!(
            device.build_output_stream(
                &config,
                move |data: &mut [f32], _| {
                    if index >= samples.len() {
                        data.fill(0.0);
                        return;
                    }

                    let end = (index + data.len()).min(samples.len());
                    let slice = &samples[index..end];
                    data[..slice.len()].copy_from_slice(slice);

                    if slice.len() < data.len() {
                        data[slice.len()..].fill(0.0);
                    }

                    index += data.len();
                },
                |e| eprintln!("Unexpected error \"{e}\". This might be a panic in future versions."),
                None,
            ).ok(),
            Err(Error::StreamCannotBeBuilt)
        );

        or_return!(stream.play().ok(), Err(Error::StreamCannotBePlayed));

        logln!("Playing \"{}\" ({:.1} seconds, RMS = {rms}, α={scale_factor})", self.filename.display(), duration_secs);

        std::thread::sleep(std::time::Duration::from_secs_f64(duration_secs));

        logln!("Finished");

        Ok(())
    }
}

impl std::fmt::Display for Song {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.filename.display())
    }
}

impl std::hash::Hash for Song {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.filename.hash(state);
    }
}

impl std::cmp::PartialEq for Song {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename
    }
}

/// Composes a playlist with the given number of elements from a database's songs.
pub fn compose_playlist(elem_cnt: usize, database: &mut SongDatabase) -> Option<Vec<Song>> {
    let mut elems: Vec<_> = database
        .inner()
        .iter()
        .filter_map(|e| if e.enabled() && !e.was_played() { Some(e.to_owned()) } else { None })
        .collect();

    if elem_cnt > elems.len() {
        database.reset_played();
        let elems = or_return!(compose_playlist(elem_cnt, database), None);
        if elems.is_empty() { // If resetting solved it, no need to return None
            return None; // Prevent an infinite loop
        }
    }

    elems.shuffle(&mut rng());

    elems.truncate(elem_cnt);

    let elems = elems.iter().map(|e| { let mut tmp = (*e).clone(); tmp.set_played(true); tmp }).collect::<Vec<_>>();
    database.inner_mut().extend(elems.iter().cloned());

    Some(elems)
}

/// Plays each song in a playlist sequentially.
/// # Warning
/// This function blocks its thread while the songs are playing.
pub fn play_playlist(playlist: &[Song]) -> Result<(), Error> {
    let host = cpal::default_host();
    let device = or_return!(host.default_output_device(), Err(Error::NoOutputDevice));

    for song in playlist {
        song.play(&device)?;
    }

    Ok(())
}