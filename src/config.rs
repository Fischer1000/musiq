use std::path::Path;
use crate::Error;
use crate::csv::CsvObject;
use crate::{int_to_bool, or_return, return_unless};
use crate::time::{Day, Time};

#[derive(Debug)]
pub struct Configs {
    timetable: Timetable,
    file_path: Box<Path>,
    utc_offset: i8
}

#[allow(unreachable_code)]
impl Configs {
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Configs, Error> {
        let contents = or_return!(std::fs::read(&path).ok(), Err(Error::CannotReadFile));

        Self::from_bytes(&contents, path)
    }

    pub fn from_bytes<P: AsRef<Path>>(bytes: &[u8], file_path: P) -> Result<Configs, Error> {
        if bytes.get(0..=5) != Some(b"MUSIQ\n") {
            return Err(Error::InvalidConfigFile);
        }

        let mut timetable: Option<Timetable> = None;
        let mut utc_offset: Option<i8> = None;

        let mut i = 6;
        '_search: while i < bytes.len() {
            match bytes.get(i) {
                Some(b'T') => {
                    timetable = Timetable::from_bytes(bytes
                        .get((i + 1)..=(i + 45))
                        .ok_or(Error::InvalidConfigFile)?
                    );
                    i += 45;
                },
                Some(b'O') => {
                    utc_offset = Some(*bytes.get(i + 1)
                        .ok_or(Error::InvalidConfigFile)? as i8);
                    i += 1;
                },
                Some(_) => return Err(Error::InvalidConfigFile),
                None => return Err(Error::InvalidConfigFile),
            }

            i += 1;
        }

        let timetable = timetable.ok_or(Error::NoTimetableFound)?;
        let utc_offset = utc_offset.ok_or(Error::NoTimetableFound)?;

        Ok(Configs { timetable, utc_offset, file_path: Box::from(file_path.as_ref()) })
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let mut contents = b"MUSIQ\nT".to_vec();

        contents.append(&mut self.timetable.to_bytes());

        contents.push(b'O');
        contents.push(self.utc_offset as u8);

        or_return!(std::fs::write(path, contents).ok(), Err(Error::CannotWriteFile));

        Ok(())
    }

    pub fn timetable(&self) -> &Timetable {
        &self.timetable
    }

    pub fn get_timetable_csv(&self) -> Vec<Vec<CsvObject>> {
        let data = self.timetable.days.iter().map(|d| d.to_csv()).collect::<Vec<Vec<CsvObject>>>();
        let mut result: Vec<Vec<CsvObject>> = Vec::new();

        for i in 0..8usize {
            let mut result_row = Vec::with_capacity(5);
            for row in data.iter() {
                result_row.push(row[i].clone());
            }
            result.push(result_row);
        }

        result
    }

    pub fn get_breaks_csv(&self) -> Vec<Vec<CsvObject>> {
        self.timetable.breaks.iter().map(|b| b.to_csv()).collect::<Vec<Vec<CsvObject>>>()
    }

    pub fn set_timetable_from_csv(&mut self, data: Vec<Vec<CsvObject>>) -> Option<()> {
        let mut result: Vec<DailySchedule> = Vec::new();

        for i in 0..5 {
            let mut tmp = Vec::new();
            for row in data.iter() {
                tmp.push(row.get(i)?);
            }
            result.push(DailySchedule::from_csv(tmp)?);
        }

        self.timetable.days = result.try_into().ok()?;

        match self.save_to_file(self.file_path.as_ref()) {
            Ok(_) => (),
            Err(_) => println!("Config save failed.")
        };

        Some(())
    }

    pub fn set_breaks_from_csv(&mut self, data: Vec<Vec<CsvObject>>) -> Option<()> {
        let breaks: [Break; 8] = data
            .into_iter()
            .filter_map(|v| Break::from_csv(v))
            .collect::<Vec<Break>>()
            .try_into()
            .ok()?;

        self.timetable.breaks = breaks;

        match self.save_to_file(self.file_path.as_ref()) {
            Ok(_) => (),
            Err(_) => println!("Config save failed.")
        };

        Some(())
    }

    pub fn utc_offset(&self) -> i8 {
        self.utc_offset
    }

    pub unsafe fn set_utc_offset_unchecked(&mut self, utc_offset: i8) {
        self.utc_offset = utc_offset;
    }
}

#[derive(Debug)]
pub struct Timetable {
    days: [DailySchedule; 5],
    breaks: [Break; 8],
}

impl Timetable {
    pub fn from_bytes(bytes: &[u8]) -> Option<Timetable> {
        return_unless!(bytes.len() == 45, None);

        let breaks = bytes
            .get(0..40)?
            .chunks_exact(5)
            .filter_map(|x| {
                Break::from_bytes(x.try_into().expect("This should not fail"))
            })
            .collect::<Vec<Break>>()
            .try_into()
            .ok()?;

        let days = bytes
            .get(40..45)?
            .iter()
            .map(DailySchedule::from_byte)
            .collect::<Vec<_>>()
            .try_into()
            .ok()?;

        Some(Timetable { days, breaks })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();

        for b in &self.breaks {
            result.append(b.to_bytes().to_vec().as_mut());
        }

        for d in &self.days {
            result.push(d.to_byte());
        }

        result
    }

    pub fn display(&self) -> String {
        let conv = |d| {
            format!("{}", d)
                .chars()
                .map(|c| match c {
                    '0'..'9' => 'X',
                    'X' => 'O',
                    _ => unreachable!()
                })
                .collect::<Vec<char>>()
        };

        let days = self.days.iter().map(conv).collect::<Vec<_>>();

        let mut buf: String = "                 | M T W T F\n".into();

        for br in 0..8 {
            buf.push_str(
                format!(
                    "{}. ({}) | {} {} {} {} {}\n", // eg.: `1. (08:35-08:40) | X X O O X`
                    br,
                    self.breaks[br],
                    days[0][br],
                    days[1][br],
                    days[2][br],
                    days[3][br],
                    days[4][br]
                ).as_str()
            );
        }

        format!("{}", buf)
    }

    /// Returns `Option<true>` when a break should start and `Option<false>` when it should end.
    pub fn action(&self, time: &Time, day: &Day) -> Option<bool> {
        let break_enabled = self.days.get(day.as_day_number() as usize)?.to_bools();

        for i in 0..8 {
            if !break_enabled[i] { continue; }

            if &self.breaks[i].start == time { return Some(true); }
            if &self.breaks[i].end == time { return Some(false); }
        }

        None
    }
}

struct Break {
    start: Time,
    end: Time
}

impl Break {
    #[inline]
    const fn new(sh: u8, sm: u8, ss: u8, eh: u8, em: u8, es: u8) -> Option<Break> {
        Some(Break {
            start: or_return!(Time::from_hms(sh, sm, ss), None),
            end: or_return!(Time::from_hms(eh, em, es), None)
        })
    }

    fn to_hms_pair(&self) -> ((u8, u8, u8), (u8, u8, u8)) {
        ( self.start.to_hms(), self.end.to_hms() )
    }

    fn from_hms_pair(start: (u8, u8, u8), end: (u8, u8, u8)) -> Option<Break> {
        let (sh, sm, ss) = start;
        let (eh, em, es) = end;
        Some(Break {
            start: Time::from_hms(sh, sm, ss)?,
            end: Time::from_hms(eh, em, es)?
        })
    }

    fn to_csv(&self) -> Vec<CsvObject> {
        let ((sh, sm, ss), (eh, em, es)) = self.to_hms_pair();

        vec![format!("{sh:02}:{sm:02}:{ss:02}").into(), format!("{eh:02}:{em:02}:{es:02}").into()]
    }

    fn from_csv(csv: Vec<CsvObject>) -> Option<Break> {
        let [start, end] = csv.try_into().ok()?;

        match (start, end) {
            (CsvObject::String(start), CsvObject::String(end)) => {
                let ((sh, sm, ss), (eh, em, es)) = (Self::time_from_str(start.as_ref())?, Self::time_from_str(end.as_ref())?);
                Self::new(sh, sm, ss, eh, em, es)
            },
            _ => None
        }
    }

    const fn to_bytes(&self) -> [u8; 5] {
        let start_bytes: [_; 4] = (self.start.elapsed_seconds() << 15).to_be_bytes();
        let end_bytes: [_; 4] = (self.end.elapsed_seconds() & 0x0001FFFF).to_be_bytes();

        [start_bytes[0], start_bytes[1], start_bytes[2] | end_bytes[1], end_bytes[2], end_bytes[3]]
    }

    fn from_bytes(bytes: &[u8; 5]) -> Option<Break> {
        let start_bytes: [u8; 4] = bytes[0..4].try_into().unwrap();

        let end_bytes: [u8; 4] = bytes[1..5].try_into().unwrap();

        let start = u32::from_be_bytes(start_bytes) >> 15;
        let end = u32::from_be_bytes(end_bytes) & 0x0001FFFF;

        Some(Break { start: Time::from_seconds(start), end: Time::from_seconds(end) })
    }

    fn time_from_str(string: &str) -> Option<(u8, u8, u8)> {
        let (h, m_s) = string.split_once(':')?;
        let (m, s) = m_s.split_once(':')?;

        Some((h.parse().ok()?, m.parse().ok()?, s.parse().ok()?))
    }
}

impl std::fmt::Debug for Break {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ((sh, sm, _ss), (eh, em, _es)) = self.to_hms_pair();
        write!(f, "{:02}:{:02}-{:02}:{:02}", sh, sm, eh, em)
    }
}

impl std::fmt::Display for Break {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

struct DailySchedule {
    data: u8
}

impl DailySchedule {
    const fn from_byte(byte: &u8) -> DailySchedule {
        DailySchedule { data: *byte }
    }

    fn to_byte(&self) -> u8 {
        self.data
    }

    fn to_bools(&self) -> [bool; 8] {
        let mut res: [bool; 8] = [false; 8];
        let mut data = *&self.data;

        for i in (0..8).rev() {
            *res.get_mut(i).unwrap() = int_to_bool!(data & 1).unwrap();
            data >>= 1;
        }

        res
    }

    fn from_bools(bools: [bool; 8]) -> DailySchedule {
        let mut result = bools[0] as u8;

        for i in 1..8 {
            result <<= 1;
            result |= bools[i] as u8;
        }

        DailySchedule { data: result }
    }

    fn to_csv(&self) -> Vec<CsvObject> {
        self.to_bools().to_vec().iter().map(|b| CsvObject::from(*b)).collect::<Vec<CsvObject>>()
    }

    fn from_csv(csv: Vec<&CsvObject>) -> Option<DailySchedule> {
        Some(Self::from_bools(
            csv
                .into_iter()
                .filter_map(|v| v.as_bool())
                .collect::<Vec<_>>()
                .try_into()
                .ok()?
        ))
    }
}

impl std::fmt::Debug for DailySchedule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bools = self.to_bools();
        let mut buf = String::with_capacity(8);

        for (i, &v) in bools.iter().enumerate() {
            buf.push(if v { (b'0' + i as u8) as char } else { 'X' });
        }

        write!(f, "{}", buf)
    }
}

impl std::fmt::Display for DailySchedule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

pub const fn default_config_bytes() -> [u8; 54] {
    let mut result = [0; 54];

    let header = b"MUSIQ\nT";

    let breaks = [
        Break::new(07, 41, 00, 07, 50, 00).unwrap().to_bytes(),
        Break::new(08, 36, 00, 08, 40, 00).unwrap().to_bytes(),
        Break::new(09, 26, 00, 09, 35, 00).unwrap().to_bytes(),
        Break::new(10, 21, 00, 10, 30, 00).unwrap().to_bytes(),
        Break::new(11, 16, 00, 11, 25, 00).unwrap().to_bytes(),
        Break::new(12, 11, 00, 12, 15, 00).unwrap().to_bytes(),
        Break::new(13, 01, 00, 13, 30, 00).unwrap().to_bytes(),
        Break::new(14, 11, 00, 14, 20, 00).unwrap().to_bytes()
    ];

    let breaks = breaks.as_flattened(); // &[u8; 40]

    let days = [0b01111100; 5];

    let mut i = 0;

    // 0..7
    while i < header.len() {
        result[i] = header[i];
        i += 1;
    }

    // 7..47
    while i < header.len() + breaks.len() {
        result[i] = breaks[i-header.len()];
        i += 1;
    }

    // 47..52
    while i < breaks.len() + header.len() + days.len() {
        result[i] = days[i-header.len()-breaks.len()];
        i += 1;
    }

    // 52
    result[52] = b'O';

    // 53
    result[53] = 0x02;

    result
}