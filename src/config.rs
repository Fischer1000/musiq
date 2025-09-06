use std::path::Path;

use crate::Error;
use crate::csv::CsvObject;
use crate::{int_to_bool, is_kind_of, or_return, return_unless, stat};
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

        if contents.get(0..=5) != Some(b"MUSIQ\n") {
            return Err(Error::InvalidConfigFile);
        }

        let mut timetable: Option<Timetable> = None;
        let mut utc_offset: Option<i8> = None;

        let mut i = 6;
        'search: while i < contents.len() {
            match contents.get(i) {
                Some(b'T') => {
                    timetable = Timetable::from_bytes(contents
                        .get((i + 1)..=(i + 21))
                        .ok_or(Error::InvalidConfigFile)?
                    );
                    i += 22;
                    continue 'search;
                },
                Some(b'O') => {
                    utc_offset = Some(*contents.get(i + 1)
                        .ok_or(Error::InvalidConfigFile)? as i8);
                    i += 2;
                    continue 'search;
                },
                Some(_) => todo!(),
                None => return Err(Error::InvalidConfigFile),
            }

            i += 1;
        }

        let timetable = timetable.ok_or(Error::NoTimetableFound)?;
        let utc_offset = utc_offset.ok_or(Error::NoTimetableFound)?;

        Ok(Configs { timetable, utc_offset, file_path: Box::from(path.as_ref()) })
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
        return_unless!(bytes.len() == 21, None);

        let breaks = bytes
            .get(0..=15)?
            .chunks_exact(2)
            .filter_map(|x| {
                if let [a, b] = x {
                    Break::from_bytes(*a, *b)
                } else {
                    unreachable!()
                }
            })
            .collect::<Vec<Break>>()
            .try_into()
            .ok()?;

        let days = bytes
            .get(16..=20)?
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
            let (hi, lo) = b.to_bytes();
            result.push(hi);
            result.push(lo);
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

            let (sh, sm, eh, em) = self.breaks[i].to_time();
            let (hour, minute, _) = time.to_hms();

            if hour == sh && minute == sm { return Some(true); }
            if hour == eh && minute == em { return Some(false); }
        }

        None
    }
}

struct Break {
    value: u16
}

impl Break {
    fn new(start_hour: u8, start_minute: u8, end_hour: u8, end_minute: u8) -> Option<Break> {
        let start = Self::parse_time(start_hour, start_minute)?;
        let end = Self::parse_time(end_hour, end_minute)?;

        Some(Break { value: u16::from_be_bytes([start, end]) } )
    }

    fn parse_time(hour: u8, minute: u8) -> Option<u8> {
        if minute >= 60 { return None }

        match hour {
            3..21 => if minute % 5 == 0 { // [0; 216[
                Some((hour - 3) * 12 + (minute / 5))
            } else {
                None
            },
            21..24 => if minute % 10 == 0 { // [216; 234[
                Some((hour - 21) * 6 + (minute / 10) + 216)
            } else {
                None
            },
            0..3 => if minute % 10 == 0 { // [238; 0[
                Some((hour) * 6 + (minute / 10) + 238)
            } else {
                None
            },
            _ => None
        }
    }

    fn value_to_time(value: u8) -> Option<(u8, u8)> { // (hour, minute)
        let (sh, sm);

        match value {
            0..216 => {
                sm = (value % 12) * 5;
                sh = value / 12 + 3
            }
            216..234 => {
                let hi = value - 120;
                sm = (hi % 6) * 10;
                sh = hi / 6 + 21
            }
            238.. => {
                let hi = value - 238;
                sm = (hi % 6) * 10;
                sh = hi / 6
            }
            _ => return None
        }

        Some((sh, sm))
    }

    fn to_time(&self) -> (u8, u8, u8, u8) { // sh, sm, eh, em
        let [hi, lo] = self.value.to_be_bytes();

        let ((sh, sm), (eh, em)) = (
            Self::value_to_time(hi).expect("The underlying value was invalid"),
            Self::value_to_time(lo).expect("The underlying value was invalid")
        );

        (sh, sm, eh, em)
    }

    fn from_bytes(start: u8, end: u8) -> Option<Break> {
        if (start >= 234 && start < 238) || (end >= 234 && end < 238) {
            None
        } else {
            Some( Break { value: ((start as u16) << 8) | (end as u16) } )
        }
    }

    fn to_bytes(&self) -> (u8, u8) {
        let [hi, lo] = self.value.to_be_bytes();
        (hi, lo)
    }

    fn to_csv(&self) -> Vec<CsvObject> {
        let (sh, sm, eh, em) = self.to_time();
        vec![format!("{sh:02}:{sm:02}").into(), format!("{eh:02}:{em:02}").into()]
    }

    fn from_csv(csv: Vec<CsvObject>) -> Option<Break> {
        let [start, end] = csv.try_into().ok()?;

        match (start, end) {
            (CsvObject::String(start), CsvObject::String(end)) => {
                let ((sh, sm), (eh, em)) = (Self::time_from_str(start.as_ref())?, Self::time_from_str(end.as_ref())?);
                Self::new(sh, sm, eh, em)
            },
            _ => None
        }
    }

    fn time_from_str(s: &str) -> Option<(u8, u8)> {
        let (h, m) = s.split_once(':')?;

        Some((h.parse().ok()?, m.parse().ok()?))
    }
}

impl std::fmt::Debug for Break {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (sh, sm, eh, em) = self.to_time();
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
    fn from_byte(byte: &u8) -> DailySchedule {
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

/*
#[derive(Debug)]
pub enum Error {
    CannotReadFile,
    InvalidConfigFile,
    NoTimetableFound,
    CannotWriteFile
}

type ConfigResult<T> = Result<T, Error>;
*/