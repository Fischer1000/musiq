extern crate rand;
extern crate minimp3;
extern crate cpal;

use std::ffi::OsStr;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

use rand::{rng, seq::SliceRandom};
use minimp3::{Decoder, Frame};
use cpal::Device;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::database::SongDatabase;
use crate::{or, or_return, stat};
use crate::Error;

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

    pub fn set_played(&mut self, played: bool) {
        if played {
            self.metadata |= 1;
        } else {
            self.metadata &= !1;
        }
    }

    #[inline]
    pub fn enable(&mut self) {
        self.metadata |= 2
    }

    #[inline]
    pub fn disable(&mut self) {
        self.metadata &= !2
    }

    pub fn play(&self, device: &Device) -> Result<(), Error> {
        let file_path = Path::new(crate::SONG_FILES_DIR).join(self.filename.as_ref());

        let file = or_return!(std::fs::File::open(file_path).ok(), Err(Error::CannotReadFile));
        let mut decoder = Decoder::new(BufReader::new(file));
        let mut samples = Vec::new();

        let mut sample_rate = 44100;
        let mut channels = 2;

        while let Ok(Frame { data, sample_rate: sr, channels: ch, .. }) = decoder.next_frame() {
            samples.extend(data);
            sample_rate = sr;
            channels = ch;
        }

        let samples = Arc::new(samples);
        let index = Arc::new(AtomicUsize::new(0));

        let config = or_return!(device.default_output_config().ok(), Err(Error::DeviceConfigCannotBeSet));
        let sample_format = config.sample_format();
        let config = config.into();

        let stream = or_return!(match sample_format {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &config,
                {
                    let samples = samples.clone();
                    let index = index.clone();
                    move |data: &mut [f32], _| {
                        let i = index.fetch_add(data.len(), Ordering::SeqCst);

                        if i >= samples.len() {
                            // Already at or past end of buffer: silence
                            for out in data.iter_mut() {
                                *out = 0.0;
                            }
                            return;
                        }

                        let end = (i + data.len()).min(samples.len());
                        let slice = &samples[i..end];

                        // Write decoded samples
                        for (out, &s) in data.iter_mut().zip(slice.iter()) {
                            *out = s as f32 / i16::MAX as f32;
                        }

                        // If fewer samples remain, fill rest with silence
                        if slice.len() < data.len() {
                            for out in &mut data[slice.len()..] {
                                *out = 0.0;
                            }
                        }
                    }
                },
                |_err| {},
                None,
            ),
            _ => panic!("Unsupported format"),
        }.ok(), Err(Error::StreamCannotBeBuilt));

        or_return!(stream.play().ok(), Err(Error::StreamCannotBePlayed));

        let duration_secs = samples.len() as f64 / (sample_rate as f64 * channels as f64);

        std::thread::sleep(std::time::Duration::from_secs_f64(duration_secs));

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
        .filter_map(|e| if e.enabled() && !e.was_played() { Some(e) } else { None })
        .collect();

    if elem_cnt > elems.len() { return None; }

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