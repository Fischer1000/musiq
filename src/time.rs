use std::fmt::write;
use std::time::SystemTime;

/// Represents a time of day
pub struct Time {
    data: u32
}

impl Time {
    /// Seconds per minute
    const SECS_PER_MIN: u32 = 60;
    /// Seconds per hour
    const SECS_PER_HOUR: u32 = 3600;
    /// Seconds per day
    const SECS_PER_DAY: u32 = 86400;
    /// Minutes per hour
    const MINS_PER_HOUR: u32 = 60;
    /// Minutes per day
    const MINS_PER_DAY: u32 = 1440;
    /// Hours per day
    const HOURS_PER_DAY: u32 = 24;

    /// Returns the current time of day with a given offset (in hours) from UTC.
    /// # Panics
    /// If the system time is before the UNIX epoch.
    pub fn now(utc_offset: i8) -> Time {
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("The system time is before the UNIX epoch.").as_secs();

        let value =
            (add_offset(timestamp, utc_offset as i32 * Time::SECS_PER_HOUR as i32) % Self::SECS_PER_DAY as u64) as u32;

        Time { data: value }
    }

    /// Returns the stored time's current seconds.
    #[inline]
    pub fn seconds(&self) -> u32 {
        self.data % Self::SECS_PER_MIN
    }

    /// Returns the stored time's current minute.
    #[inline]
    pub fn minutes(&self) -> u32 {
        (self.data / Self::SECS_PER_MIN) % Self::MINS_PER_HOUR
    }

    /// Returns the stored time's current hour.
    #[inline]
    pub fn hours(&self) -> u32 {
        self.data / Self::SECS_PER_HOUR
    }

    /// Converts the underlying value to hours, minutes, and seconds.
    #[inline]
    pub fn to_hms(&self) -> (u8, u8, u8) {
        let hours = self.hours();
        let minutes = self.minutes();
        let seconds = self.seconds();

        (hours as u8, minutes as u8, seconds as u8)
    }

    /// Converts hours, minutes, and seconds into the internal representation and stores it.
    pub fn from_hms(hours: u8, minutes: u8, seconds: u8) -> Option<Time> {
        if hours   as u32 >= Self::HOURS_PER_DAY { return None }
        if minutes as u32 >= Self::MINS_PER_HOUR { return None }
        if seconds as u32 >= Self::SECS_PER_MIN  { return None }

        Some(Time { data: ((hours as u32 * Self::MINS_PER_HOUR) + minutes as u32) * Self::SECS_PER_MIN + (seconds as u32) })
    }
}

impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.hours(), self.minutes(), self.seconds())
    }
}

/// Represents a day of a week
pub enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday
}

impl Day {
    /// Returns this day's number starting from Monday = 0 to Sunday = 6
    #[inline]
    pub fn as_day_number(&self) -> u8 {
        match self {
            Day::Monday => 0,
            Day::Tuesday => 1,
            Day::Wednesday => 2,
            Day::Thursday => 3,
            Day::Friday => 4,
            Day::Saturday => 5,
            Day::Sunday => 6
        }
    }

    /// Returns a day from its number starting Monday = 0 to Sunday = 6
    #[inline]
    pub fn from_day_number(num: u8) -> Option<Day> {
        match num {
            0 => Some(Day::Monday),
            1 => Some(Day::Tuesday),
            2 => Some(Day::Wednesday),
            3 => Some(Day::Thursday),
            4 => Some(Day::Friday),
            5 => Some(Day::Saturday),
            6 => Some(Day::Sunday),
            _ => None
        }
    }

    /// Returns today's day with a specified offset (in hours) from UTC.
    pub fn today(utc_offset: i8) -> Day {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("The system time is before the UNIX epoch.")
            .as_secs();

        let days_since_epoch =
            add_offset(timestamp, utc_offset as i32 * Time::SECS_PER_HOUR as i32) / Time::SECS_PER_DAY as u64;

        // The UNIX epoch (1970. 01. 01.) was a Thursday, so an offset is needed

        Self::from_day_number(((days_since_epoch + 3) % 7) as u8).unwrap()
    }
}

/// Adds an offset of `i32` to a `u64`
fn add_offset(val: u64, offset: i32) -> u64 {
    match offset {
        i32::MIN..0 => val - offset.abs() as u64,
        0..=i32::MAX => val + offset as u64
    }
}