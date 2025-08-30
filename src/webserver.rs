use std::collections::HashSet;
use std::convert::Infallible;
use std::ffi::OsStr;
use std::net::{TcpListener, ToSocketAddrs, TcpStream};
use std::io::{BufReader, BufRead, Write, Read};
use std::mem::MaybeUninit;
use std::path::Path;


use crate::{is_kind_of, or_continue, or_return, stat};
use crate::embedded_files;
use crate::csv::{CsvObject, DEFAULT_SEPARATOR};
use crate::songs::Song;

const MAX_BODY_SIZE: usize = 500_000_000;
static METHODS_WITH_BODY: &[&str] = &["POST"];

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    CannotBind,
    RequestReadFailed,
    InvalidRequest,
    CannotInferLength,
    InvalidUtf8,
    UnsupportedMethod,
    BodyTooLarge
}

pub type Headers = Vec<String>;
pub type Body = Vec<u8>;

#[derive(Debug)]
#[non_exhaustive]
pub enum Request {
    Get { uri: Uri, headers: Headers },
    Post { uri: Uri, headers: Headers, body: Body },
}

impl Request {
    pub fn from_stream(stream: &TcpStream) -> Result<Self, Error> {
        let mut buf_reader_bytes = BufReader::new(stream).bytes();

        let header_bytes = {
            let mut sequence_buf = Vec::with_capacity(4);
            let mut error = false;

            let header_bytes = buf_reader_bytes.by_ref().map_while(|b| match b {
                Ok(x) => if (
                    (x == b'\r' && (sequence_buf.ends_with(&[b'\n']) || sequence_buf.len() == 0)) ||
                        x == b'\n' && sequence_buf.ends_with(&[b'\r'])
                ) {
                    sequence_buf.push(x);
                    if sequence_buf.len() == 4 {
                        None
                    } else {
                        Some(x)
                    }
                } else {
                    sequence_buf.clear();
                    Some(x)
                },
                Err(_) => { error = true; None }
            }).collect::<Vec<_>>();

            if error {
                return Err(Error::RequestReadFailed);
            }

            header_bytes
        };

        let headers = String::from_utf8(header_bytes).map_err(|_| Error::InvalidUtf8)?;
        let (status_row, headers) = headers.split_once("\r\n").ok_or(Error::InvalidRequest)?;

        let headers = headers.split("\r\n").map(|s| s.to_string()).collect::<Vec<String>>();

        let [method, uri, _version] = {
            let x: [&str; 3] = status_row
                .splitn(3, ' ')
                .collect::<Vec<&str>>()
                .try_into()
                .map_err(|_| Error::InvalidRequest)?;
            x
        };

        let body = if METHODS_WITH_BODY.contains(&method) {
            let mut content_length: Option<usize> = None;

            for line in &headers {
                if let Some(value) = line.strip_prefix("Content-Length:") {
                    content_length = value.trim().parse().ok();
                    break;
                }
            }

            let content_length = content_length.ok_or(Error::CannotInferLength)?;

            if content_length > MAX_BODY_SIZE {
                return Err(Error::BodyTooLarge);
            }

            let body = {
                let mut error = false;
                let body = buf_reader_bytes.take(content_length).map(|v| match v {
                    Ok(v) => v,
                    Err(_) => {
                        error = true;
                        unsafe { MaybeUninit::uninit().assume_init() }
                    }
                }).collect::<Vec<_>>();

                if error {
                    return Err(Error::RequestReadFailed);
                }

                body
            };
            body
        } else {
            Vec::new()
        };

        Ok(match method {
            "GET" => Request::Get { uri: uri.into(), headers },
            "POST" => Request::Post { uri: uri.into(), headers, body },
            _ => { return Err(Error::UnsupportedMethod) }
        })
    }
}

#[derive(Debug)]
pub struct Uri(pub Box<str>);

impl<T: Into<Box<str>>> From<T> for Uri {
    fn from(s: T) -> Uri {
        Uri(s.into())
    }
}

impl Uri {
    pub fn without_query_parameters(&self) -> &str {
        self.0.splitn(2, '?').next().unwrap()
    }
}

pub struct Response {
    status_code: u8,
    reason: Box<str>,
    headers: Headers,
    body: Body
}

impl Response {
    const fn store_status_code(code: u16) -> Option<u8> {
        Some(match code {
            100..=103 => code - 100, //  0 -   3
            200..=226 => code - 196, //  4 -  30
            300..=308 => code - 269, // 31 -  39
            400..=451 => code - 360, // 40 -  91
            500..=511 => code - 408, // 92 - 103
            _ => return None
        } as u8)
    }

    const fn retrieve_status_code(code: u8) -> Option<u16> {
        let code = code as u16;
        Some(match code {
            0..=3 =>    code + 100, // 100 - 103
            4..=30 =>   code + 196, // 200 - 226
            31..=39 =>  code + 269, // 300 - 308
            40..=91 =>  code + 360, // 400 - 451
            92..=103 => code + 408, // 500 - 511
            _ => return None
        })
    }

    pub fn new(code: u16, reason: &str, headers: Headers, body: Body) -> Option<Response> {
        Some(Response {
            status_code: Self::store_status_code(code)?,
            reason: reason.into(),
            headers,
            body
        })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut result = format!(
            "HTTP/1.1 {} {}\r\n{}\r\n\r\n",
            Self::retrieve_status_code(self.status_code).unwrap(),
            self.reason,
            self.headers.join("\r\n")
        ).into_bytes();

        result.extend_from_slice(self.body.as_slice());

        result
    }

    pub fn ok(body: Body) -> Response {
        Response {
            status_code: Self::store_status_code(200).unwrap(),
            reason: "OK".into(),
            headers: Vec::new(),
            body
        }
    }

    pub fn empty_ok() -> Response {
        Self::ok(Vec::new())
    }

    pub fn not_found() -> Response {
        Response {
            status_code: Self::store_status_code(404).unwrap(),
            reason: "Not Found".into(),
            headers: Vec::new(),
            body: "404 Not Found".bytes().collect()
        }
    }

    pub fn not_implemented() -> Response {
        Response {
            status_code: Self::store_status_code(501).unwrap(),
            reason: "Not Implemented".into(),
            headers: Vec::new(),
            body: Vec::new()
        }
    }

    pub fn internal_server_error() -> Response {
        Response {
            status_code: Self::store_status_code(500).unwrap(),
            reason: "Internal Server Error".into(),
            headers: Vec::new(),
            body: "500 Internal Server Error".bytes().collect()
        }
    }

    pub fn bad_request() -> Response {
        Response {
            status_code: Self::store_status_code(400).unwrap(),
            reason: "Bad Request".into(),
            headers: Vec::new(),
            body: Vec::new()
        }
    }

    pub fn length_required() -> Response {
        Response {
            status_code: Self::store_status_code(411).unwrap(),
            reason: "Length Required".into(),
            headers: Vec::new(),
            body: Vec::new()
        }
    }

    pub fn content_too_large() -> Response {
        Response {
            status_code: Self::store_status_code(413).unwrap(),
            reason: "Content Too Large".into(),
            headers: Vec::new(),
            body: Vec::new()
        }
    }

    pub fn conflict() -> Response {
        Response {
            status_code: Self::store_status_code(409).unwrap(),
            reason: "Conflict".into(),
            headers: Vec::new(),
            body: Vec::new()
        }
    }
}

type Database = crate::database::SongDatabase;
pub fn listen<A: ToSocketAddrs, H: Fn(Result<Request, Error>, &mut Database) -> Response>(addr: A, handler: H, mut database: Database) -> Result<Infallible, Error> {
    let listener = TcpListener::bind(addr).map_err(|_| Error::CannotBind)?;

    for stream in listener.incoming() {
        let mut stream = or_continue!(stream.ok());

        let response = handler(Request::from_stream(&stream), &mut database);

        or_continue!(stream.write_all(response.as_bytes().as_slice()).ok())
    }

    unreachable!()
}

pub fn handle_request(request: Result<Request, Error>, database: &mut Database) -> Response {
    let request = match request {
        Ok(r) => r,
        Err(e) => return match e {
            Error::RequestReadFailed => Response::internal_server_error(),
            Error::InvalidUtf8 => Response::bad_request(),
            Error::InvalidRequest => Response::bad_request(),
            Error::CannotInferLength => Response::length_required(),
            Error::BodyTooLarge => Response::content_too_large(),
            Error::UnsupportedMethod => Response::not_implemented(),
            _ => unreachable!()
        }
    };

    match request {
        Request::Get { uri, headers } => handle_get(uri, headers, database),
        Request::Post { uri, headers, body } => handle_post(uri, headers, body, database),
        _ => return Response::not_implemented()
    }
}

fn handle_get(uri: Uri, headers: Headers, database: &Database) -> Response {
    let content_type: &'static str;

    let body = 'match_uri: {
        match uri.without_query_parameters() {
            "/" => { content_type = "text/html"; embedded_files::INDEX_HTML },
            "/files/styles.css" => { content_type = "text/css"; embedded_files::STYLES_CSS },
            "/files/script.js" => { content_type = "text/javascript"; embedded_files::SCRIPT_JS },
            "/files/favicon.svg" => { content_type = "image/svg+xml"; embedded_files::FAVICON_SVG },
            "/data/timetable.csv" => return Response::not_found(),
            "/data/breaks.csv" => return Response::not_found(),
            "/data/songs.csv" => break 'match_uri {
                content_type = "text/csv";
                CsvObject::serialize(
                    database.get_songs_css(),
                    DEFAULT_SEPARATOR,
                ).into_bytes()
            },
            _ => return Response::not_found(),
        }.to_vec()
    };

    let headers = vec![format!("Content-Type: {}", content_type)];
    // let headers = vec![];

    Response::new(200, "OK", headers, body).unwrap()
}

fn handle_post(uri: Uri, headers: Headers, body: Body, database: &mut Database) -> Response {
    // println!("URI: {:?}\nHeaders:\n{}Body length: {}\n", uri, headers.join("\n"), body.len());

    // println!("{:?}", String::from_utf8(body.clone()).or::<()>(Ok("".to_string())));

    match uri.without_query_parameters() {
        "/api/set-timetable" => Response::internal_server_error(),
        "/api/disable-songs" => {
            let mut success: u16 = 0;

            for name in (
                or_return!(
                    CsvObject::from_str(
                        or_return!(
                            str::from_utf8(body.as_slice()).ok(),
                            Response::bad_request()),
                        DEFAULT_SEPARATOR
                    ).into_iter().take(1).next(),
                    Response::bad_request()
                ).iter().filter_map(|v| v.as_string())
            ) {
                let songs = database
                    .inner()
                    .iter()
                    .map(|song| if song.filename() == OsStr::new(name) {
                        let mut s = song.clone();
                        s.disable();
                        success += 1;
                        s
                    } else {
                        song.clone()
                    })
                    .collect::<HashSet<_>>();

                *database.inner_mut() = songs;
            }

            if success == 0 {
                Response::new(404, "Not Found", Vec::new(), "All requests failed.".as_bytes().to_vec()).unwrap()
            } else {
                Response::ok(format!("{} successfully disabled", success).as_bytes().to_vec())
            }
        },
        "/api/enable-songs" => {
            let mut success: u16 = 0;

            for name in (
                or_return!(
                    CsvObject::from_str(
                        or_return!(
                            str::from_utf8(body.as_slice()).ok(),
                            Response::bad_request()),
                        DEFAULT_SEPARATOR
                    ).into_iter().take(1).next(),
                    Response::bad_request()
                ).iter().filter_map(|v| v.as_string())
            ) {
                let songs = database
                    .inner()
                    .iter()
                    .map(|song| if song.filename() == OsStr::new(name) {
                        let mut s = song.clone();
                        s.enable();
                        success += 1;
                        s
                    } else {
                        song.clone()
                    })
                    .collect::<HashSet<_>>();

                *database.inner_mut() = songs;
            }

            if success == 0 {
                Response::new(404, "Not Found", Vec::new(), "All requests failed.".as_bytes().to_vec()).unwrap()
            } else {
                Response::ok(format!("{} successfully enabled", success).as_bytes().to_vec())
            }
        },
        "/api/delete-songs" => {
            let mut success: u16 = 0;
            let mut error: u16 = 0;

            for name in (
                or_return!(
                    CsvObject::from_str(
                        or_return!(
                            str::from_utf8(body.as_slice()).ok(),
                            Response::bad_request()),
                        DEFAULT_SEPARATOR
                    ).into_iter().take(1).next(),
                    Response::bad_request()
                ).iter().filter_map(|v| v.as_string())
            ) {
                let file_path = Path::new(crate::SONG_FILES_DIR).join(name);

                match database.remove_entry(OsStr::new(name)).realize(database, false) {
                    Ok(_) => match std::fs::remove_file(file_path) {
                        Ok(_) => success += 1,
                        Err(_) => error += 1,
                    },
                    Err(_) => error += 1,
                };
            }

            if success == 0 {
                Response::new(404, "Not Found", Vec::new(), "All requests failed.".as_bytes().to_vec()).unwrap()
            } else {
                Response::ok(format!("{} successfully removed, {} errored", success, error).as_bytes().to_vec())
            }

        },
        "/api/add-song" => {
            let [filename, file_contents]: [&[u8]; 2] = or_return!(
                body
                    .splitn(2, |v| *v == b':')
                    .collect::<Vec<&[u8]>>()
                    .try_into()
                    .ok(),
                Response::bad_request()
            );
            let file_path = Path::new(crate::SONG_FILES_DIR)
                .join(or_return!(
                    str::from_utf8(filename).ok(),
                    Response::bad_request()
                ));

            match std::fs::File::create_new(&file_path) {
                Ok(mut file) => match file.write_all(file_contents) {
                    Ok(_) => {
                        database.add_file(file_path.into_boxed_path()).realize(database, false).unwrap();
                        Response::ok("File saved".as_bytes().to_vec())
                    },
                    Err(_) => { drop(file); std::fs::remove_file(file_path); Response::internal_server_error() }
                },
                Err(e) => match e.kind() {
                    std::io::ErrorKind::AlreadyExists => Response::conflict(),
                    _ => Response::internal_server_error()
                }
            }
        },
        _ => Response::not_found(),
    }
}