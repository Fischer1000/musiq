use std::collections::VecDeque;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::num::{NonZeroU16, NonZeroU64};
use std::path::{Path, PathBuf};
use cpal::Device;
use cpal::traits::HostTrait;
use crate::{logln, or_return, Error, EVENT_FILES_DIR, EVENT_QUEUE_FILE_NAME};
use crate::csv::CsvObject;
use crate::songs::play_mp3;

fn is_leap_year(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}
fn days_in_month(y: u64, m: u64) -> Option<u64> {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31u64),
        4 | 6 | 9 | 11 => Some(30u64),
        2 => Some(if is_leap_year(y) { 29u64 } else { 28u64 }),
        _ => None
    }
}

/// An event for playing arbitrary sounds
#[derive(Debug, Clone)]
pub struct Event {
    /// The trigger of the event
    trigger: Option<ScheduledTrigger>,
    /// The display name of the event
    name: Box<str>,
    // /// The name of the file to be played in the `events` directory
    // file_name: Box<OsStr>
}

impl Event {
    /// Creates a new event with a name and an optional trigger. Saves the specified file contents.
    pub fn new(
        trigger: Option<ScheduledTrigger>,
        name: Box<str>,
        file_contents: impl AsRef<[u8]>/*,
        file_name: Box<OsStr>*/
    ) -> Result<Self, Error> {
        let event = Self { trigger, name/*, file_name: OsString::from("").into_boxed_os_str()*/ };

        or_return!(
            std::fs::write(PathBuf::from(EVENT_FILES_DIR).join(event.obtain_filename().as_ref()), file_contents).ok(),
            Err(Error::CannotWriteFile)
        );

        Ok(event)
    }

    /// Triggers this event and returns whether it is necessary to remove it
    pub fn trigger_event(&mut self, device: &Device) -> Result<bool, Error> {
        let last_trigger = self.update_trigger_time();

        play_mp3(
            self.obtain_filename().as_ref(),
            &device,
            |rms, scale_factor, duration_secs|
                logln!(
                    "Playing \"{}\" ({:.1} seconds, RMS = {rms}, α={scale_factor})",
                    self.name,
                    duration_secs
                )
        )?;

        Ok(last_trigger)
    }

    /// Updates this event's trigger time and returns whether it is necessary to remove it
    pub fn update_trigger_time(&mut self) -> bool {
        let mut last_trigger = false;

        match self.trigger {
            Some(ref mut trigger) => {
                trigger.next_trigger += match trigger.trigger_period { // Add the trigger period if it is specified
                    Some(period) => period.get(),
                    None => { self.trigger = None; return false }
                };

                match trigger.triggers_remaining.map(|x| x.get()).unwrap_or(0) {
                    0 => (), // None
                    1 => last_trigger = true,
                    x => trigger.triggers_remaining = NonZeroU16::new(x-1)
                }
            },
            None => (),
        }

        last_trigger
    }

    /// Creates bytes from this instance to be saved to the disk
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = self.name.as_bytes().to_vec(); // Save name

        result.push(b'\0'); // Add end indication

        if let Some(trigger) = self.trigger { // Save trigger if any
            result.append(trigger.to_bytes().to_vec().as_mut());
        }

        result
    }

    /// Loads an event from bytes. This is unimplemented currently
    pub fn from_bytes(_bytes: &[u8]) -> Result<Self, Error> {
        unimplemented!()
    }

    /// This creates an event from its parts. Does not save a file.
    fn from_parts(name: Box<str>, trigger: Option<ScheduledTrigger>) -> Self {
        Self { name, trigger }
    }

    pub fn obtain_filename(&self) -> Box<str> {
        let mut hasher = DefaultHasher::new();
        self.name.hash(&mut hasher);
        format!("{:016x}.mp3", hasher.finish()).into_boxed_str()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.trigger.cmp(&other.trigger)
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.trigger == other.trigger
    }
}

impl Eq for Event {}

/// The type for defining the triggering of an event
#[derive(Debug, Copy, Clone)]
pub struct ScheduledTrigger {
    /// Seconds from the Unix epoch defining the next triggering
    next_trigger: u64,
    /// The seconds between consecutive triggerings
    trigger_period: Option<NonZeroU64>,
    /// The number of triggerings left if `Some(_)` or unlimited if `None`
    triggers_remaining: Option<NonZeroU16>,
    /// Whether to auto-delete this event after the last scheduled
    /// trigger of it
    auto_delete: bool
}

impl ScheduledTrigger {
    pub fn new(
        next_trigger: u64,
        trigger_period: Option<NonZeroU64>,
        triggers_remaining: Option<NonZeroU16>,
        auto_delete: bool
    ) -> Self {
        Self { next_trigger, trigger_period, triggers_remaining, auto_delete }
    }

    pub fn to_bytes(&self) -> [u8; 19] {
        let mut result = [0u8; 19];

        result[0..8].copy_from_slice(&self.next_trigger.to_be_bytes());
        result[8..16].copy_from_slice(
            &self.trigger_period
                .map(|x| x.get()) // NonZeroU64 -> u64
                .unwrap_or(0)
                .to_be_bytes()
        );
        result[16..18].copy_from_slice(
            &self.triggers_remaining
                .map(|x| x.get()) // NonZeroU16 -> u16
                .unwrap_or(0)
                .to_be_bytes()
        );
        result[18] = self.auto_delete as u8;

        result
    }

    pub fn from_bytes(bytes: [u8; 19]) -> Option<ScheduledTrigger> {
        let next_trigger = u64::from_be_bytes(bytes[0..8].try_into().unwrap());
        let trigger_period = NonZeroU64::new(
            u64::from_be_bytes(bytes[8..16].try_into().unwrap())
        );
        let triggers_remaining = NonZeroU16::new(
            u16::from_be_bytes(bytes[16..18].try_into().unwrap())
        );
        let auto_delete= match bytes[18] {
            0 => false,
            1 => true,
            _ => return None
        };

        Some(ScheduledTrigger { next_trigger, trigger_period, triggers_remaining, auto_delete })
    }

    /// Generates the internal trigger representation from a UNIX timestamp
    pub fn raw_next_trigger_from(raw: &str) -> Option<u64> {
        macro_rules! extract_datetime {
            ($trigger_time:ident, $range:expr) => {
                $trigger_time.get($range)?.parse::<u64>().ok()?
            };
        }

        let year   = extract_datetime!(raw,  0.. 4);
        let month  = extract_datetime!(raw,  5.. 7);
        let day    = extract_datetime!(raw,  8..10);
        let hour   = extract_datetime!(raw, 11..13);
        let minute = extract_datetime!(raw, 14..16);
        let second = raw.get(17..19).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);

        if !(0 < month && month <= 12) || // Check for invalid values
            !(0 < day && month <= days_in_month(year, month).unwrap())
            || hour >= 24 || minute >= 60 || second >= 60
        {
            return None
        }

        // Convert date to "days since 1970-01-01"
        // Adapted from Howard Hinnant’s civil_from_days() inverse.
        let y = if month <= 2 { (year as i64) - 1 } else { year as i64 };
        let m = if month <= 2 { (month as i64) + 12 } else { month as i64 };
        let d = day as i64;

        let era = y / 400;
        let yoe = y - era * 400;                        // [0, 399]
        let doy = ((153*(m - 3) + 2)/5 + d - 1) as i64; // [0, 365]
        let doe = yoe * 365 + yoe/4 - yoe/100 + doy;    // [0, 146096]
        let days = era * 146097 + doe - 719468;         // Days since 1970-01-01

        if days < 0 {
            return None; // before epoch
        }

        let total = days as u64 * 86400 + hour * 3600 + minute * 60 + second;

        Some(total)
    }

    pub fn next_trigger_raw(&self) -> u64 {
        self.next_trigger
    }

    /// Generates a UNIX timestamp from the internal trigger representation
    pub fn next_trigger(&self) -> Box<str> {
        let mut time = self.next_trigger;

        let days = time / (24 * 60 * 60);
        let mut seconds = time % (24 * 60 * 60);

        let hour = seconds / (60 * 60);
        seconds %= 60 * 60;

        let minute = seconds / 60;
        let second = seconds % 60;

        // Date calculation (proleptic Gregorian calendar)
        // Algorithm from Howard Hinnant’s "Civil From Days" (used in C++20)
        let z = days as i64 + 719468;                              // Days since 0000-03-01
        let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
        let doe = z - era * 146097;                                // Day of era
        let yoe = (doe - doe/1460 + doe/36524 - doe/146096) / 365; // Year of era
        let mut year = yoe + era * 400;
        let doy = doe - (365*yoe + yoe/4 - yoe/100);               // Day of year
        let mp = (5*doy + 2)/153;                                  // Month parameter
        let day = doy - (153*mp+2)/5 + 1;                          // Day of month
        let month = mp + if mp < 10 {3} else {-9};                 // Month number (1–12)
        if month <= 2 {
            year += 1;
        }

        format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}").into_boxed_str()
    }

    pub fn trigger_period(&self) -> Option<NonZeroU64> {
        self.trigger_period
    }

    pub fn triggers_remaining(&self) -> Option<NonZeroU16> {
        self.triggers_remaining
    }

    pub fn auto_delete(&self) -> bool {
        self.auto_delete
    }
}

impl Ord for ScheduledTrigger {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.next_trigger.cmp(&other.next_trigger)
    }
}

impl PartialOrd for ScheduledTrigger {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ScheduledTrigger {
    fn eq(&self, other: &Self) -> bool {
        self.next_trigger == other.next_trigger
    }
}

impl Eq for ScheduledTrigger {}


/// A queue for events to trigger
#[derive(Debug)]
pub struct EventQueue {
    /// Events with triggers
    queued: VecDeque<Event>,
    /// Events without triggers
    non_queued: Vec<Event>
}

impl EventQueue {
    pub fn new(events: Vec<Event>) -> Self {
        // let mut events = events.into_iter().filter(|x| x.trigger.is_some()).collect::<Vec<_>>();

        let mut queued = Vec::new();
        let mut non_queued = Vec::new();

        for event in events.into_iter() {
            if event.trigger.is_some() {
                queued.push(event);
            } else {
                non_queued.push(event);
            }
        }

        queued.sort_unstable();

        // let mut queue = VecDeque::with_capacity(events.len());

        let queued = VecDeque::from(queued);

        Self { queued, non_queued }
    }

    /// Triggers all events that have gone in the past
    pub fn trigger_events(&mut self, unix_time: u64) -> Vec<Event> {
        let mut to_trigger = Vec::new();

        loop {
            let mut pop = false;

            // Check if event is in the past
            if let Some(event) = self.queued.front() {
                let trigger = event.trigger.unwrap();
                let event_time = trigger.next_trigger;

                if event_time <= unix_time && trigger.triggers_remaining.map(|n| n.get() > 0).unwrap_or(true) {
                    pop = true;
                } else {
                    return to_trigger; // If one is in the future, the followings will be too
                }
            } else {
                return to_trigger; // If the queue emptied itself...
            }

            // If it is in the past
            if pop {
                let mut event = self.queued.pop_front().unwrap();

                let remove = event.update_trigger_time();

                to_trigger.push(event.clone());

                if !remove { // Insert back if necessary
                    self.insert_event(event);
                }
            }
        }
    }

    /// Inserts a new event into the queue. The runtime cost is `O(n)`.
    pub fn insert_event(&mut self, event: Event) {
        let position: usize;

        if event.trigger.is_none() {
            self.non_queued.push(event);
            return;
        }

        // No events
        if self.queued.len() == 0 {
            self.queued.push_back(event);
            return;
        }

        // Some events
        'pos_search: {
            for (i, e) in self.queued.iter().enumerate() {
                if event.trigger.unwrap() < e.trigger.unwrap() {
                    position = i;
                    break 'pos_search;
                }
            }

            self.queued.push_back(event);
            return;
        }

        self.queued.insert(position, event);
    }

    /// Saves the queue to the disk
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let mut data = Vec::new();

        for event in self.queued.iter() {
            data.append(event.to_bytes().as_mut());
        }

        data.push(b'\0');

        for event in self.non_queued.iter() {
            data.append(event.to_bytes().as_mut());
        }

        // PathBuf::from(EVENT_FILES_DIR).join(EVENT_QUEUE_FILE_NAME)
        std::fs::write(&path, data).or(Err(Error::CannotWriteFile))
    }

    /// Loads a queue from the disk
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<EventQueue, Error> {
        let contents = match std::fs::read(path) {
            Ok(data) => data,
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => return Ok(EventQueue::new(Vec::new())),
                _ => return Err(Error::EventQueueFileCannotBeRead),
            }
        };

        let mut entries = Vec::new();

        let mut start: usize = 0;

        let mut scheduled = true;

        for (i, &b) in contents.iter().enumerate() {
            if i < start { continue; }

            if i == start && b == b'\0' {
                scheduled = false;
                start = i + 1;
                continue;
            }

            if b == b'\0' {
                let name = or_return!(
                    String::from_utf8(contents[start..i].to_vec()).ok(),
                    Err(Error::InvalidUtf8)
                ).into_boxed_str();

                if scheduled {
                    let trigger = ScheduledTrigger::from_bytes(contents[(i+1)..=(i+19)].try_into().unwrap());

                    entries.push(Event::from_parts(name, trigger));

                    start = i + 20;
                } else {
                    entries.push(Event::from_parts(name, None));
                    start = i + 1;
                }
            }
        }

        Ok(EventQueue::new(entries))
    }

    /// Removes all events by this name
    pub fn remove_by_name(&mut self, name: &str) {
        let test = |e: &Event| if e.name.as_ref() == name {
            let _ = std::fs::remove_file(PathBuf::from(EVENT_FILES_DIR).join(e.obtain_filename().as_ref()));
            false
        } else {
            true
        };

        self.non_queued.retain(test);
        self.queued.retain(test);
    }

    pub fn get_queue_csv(&self) -> Vec<Vec<CsvObject>> {
        let mut result = Vec::new();

        let mut entries = self.queued.iter()
            .chain(self.non_queued.iter())
            .map(|e| (
                e.name(),
                e.trigger.map(|t| t.next_trigger())
                    .unwrap_or("Not Scheduled".into()))
            )
            .collect::<Vec<_>>();

        entries.sort_unstable_by(|(f0, ..), (f1, ..)| f0.cmp(f1));

        for (name, time) in entries {
            let name = name.into();
            let time = time.into();

            result.push(vec![name, time]);
        }

        result
    }
}

impl Drop for EventQueue {
    fn drop(&mut self) {
        self.save_to_file(PathBuf::from(EVENT_FILES_DIR).join(EVENT_QUEUE_FILE_NAME));
    }
}