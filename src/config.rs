use std::path::Path;
use std::fs::read;
use crate::{int_to_bool, is_kind_of, return_unless};

pub struct Configs {
    timetable: Timetable
}

#[allow(unreachable_code)]
impl Configs {
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> ConfigResult<Configs> {
        let contents = read(path).map_err(|_| Error::CannotReadFile)?;

        if contents.get(0..=5) != Some(b"MUSIQ\n") {
            return Err(Error::InvalidConfigFile);
        }

        let mut timetable: Option<Timetable> = None;

        let mut i = 6;
        'search: while i < contents.len() && !is_kind_of!(timetable, Some(_)) {
            match contents.get(i) {
                Some(b'T') => {
                    timetable = Timetable::from_bytes(contents
                        .get((i + 1)..=(i + 21))
                        .ok_or(Error::InvalidConfigFile)?
                    );
                    i += 22;
                    continue 'search;
                },
                Some(_) => todo!(),
                None => return Err(Error::InvalidConfigFile),
            }


            i += 1;
        }

        Ok(Configs { timetable: timetable.ok_or(Error::NoTimetableFound)? })
    }

    pub fn timetable(&self) -> &Timetable {
        &self.timetable
    }
}

#[derive(Debug)]
pub struct Timetable {
    days: [Day; 5],
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
            .map(Day::from_byte)
            .collect::<Vec<_>>()
            .try_into()
            .ok()?;

        Some(Timetable { days, breaks })
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
}

struct Break {
    value: u16
}

impl Break {
    fn new(start_hour: u8, start_minute: u8, end_hour: u8, end_minute: u8) -> Option<Break> {
        let start = Self::parse_time(start_hour, start_minute)?;
        let end = Self::parse_time(end_hour, end_minute)?;

        if start >= end {
            Some(Break { value: u16::from_be_bytes([start, end]) } )
        } else {
            None
        }
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

struct Day {
    data: u8
}

impl Day {
    fn from_byte(byte: &u8) -> Day {
        Day { data: *byte }
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
}

impl std::fmt::Debug for Day {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bools = self.to_bools();
        let mut buf = String::with_capacity(8);

        for (i, &v) in bools.iter().enumerate() {
            buf.push(if v { (b'0' + i as u8) as char } else { 'X' });
        }

        write!(f, "{}", buf)
    }
}

impl std::fmt::Display for Day {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug)]
pub enum Error {
    CannotReadFile,
    InvalidConfigFile,
    NoTimetableFound,
}

type ConfigResult<T> = Result<T, Error>;