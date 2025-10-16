use std::convert::Infallible;
use std::ffi::OsStr;
use std::net::{TcpListener, ToSocketAddrs, TcpStream};
use std::io::{BufReader, Write, Read};
use std::mem::MaybeUninit;
use std::path::Path;

use crate::{generated, logln, or_continue, or_return, songs, time};
use crate::config::Configs;
use crate::csv::{CsvObject, DEFAULT_SEPARATOR, DEFAULT_STR_MARKER};
use crate::Error;
use crate::songs::Song;
use crate::generated::{Encoding, ENCODING};

const MAX_BODY_SIZE: usize = 500_000_000;
static METHODS_WITH_BODY: &[&str] = &["POST"];

/*
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    CannotBind,
    CannotSetNonblocking,
    RequestReadFailed,
    InvalidRequest,
    CannotInferLength,
    InvalidUtf8,
    UnsupportedMethod,
    BodyTooLarge,
    InvalidConfigFile
}
*/

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
                #[allow(unused_parens)]
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
                    #[allow(invalid_value)] // Body is never read if at least one error has occurred
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

    pub fn unprocessable_request() -> Response {
        Response {
            status_code: Self::store_status_code(422).unwrap(),
            reason: "Unprocessable Content".into(),
            headers: Vec::new(),
            body: Vec::new()
        }
    }

    pub fn permanent_redirect(to: &str) -> Response {
        Response {
            status_code: Self::store_status_code(308).unwrap(),
            reason: "Permanent Redirect".into(),
            headers: vec![format!("Location: {to}")],
            body: Vec::new()
        }
    }
}

type Database = crate::database::SongDatabase;
pub fn start_server<A: ToSocketAddrs, H: Fn(Result<Request, Error>, &mut Database, &mut Configs) -> Response>(addr: A, handler: H, mut database: Database, mut configs: Configs) -> Result<Infallible, Error> {
    let listener = TcpListener::bind(addr).map_err(|_| Error::CannotBind)?;

    for stream in listener.incoming() {
        let mut stream = or_continue!(stream.ok());

        let response = handler(Request::from_stream(&stream), &mut database, &mut configs);

        or_continue!(stream.write_all(response.as_bytes().as_slice()).ok())
    }

    unreachable!()
}

#[must_use = "Requests must be replied to"]
pub fn handle_request(request: Result<Request, Error>, database: &mut Database, configs: &mut Configs) -> Response {
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
        Request::Get { uri, headers } => handle_get(uri, headers, database, configs),
        Request::Post { uri, headers, body } => handle_post(uri, headers, body, database, configs),
        // _ => return Response::not_implemented()
    }
}

fn handle_get(uri: Uri, _headers: Headers, database: &Database, configs: &Configs) -> Response {
    let content_type: &'static str;
    let content_encoding: Option<&'static str>;

    let body = 'match_uri: {
        let embedded_encoding = match ENCODING {
            Encoding::Brotli => Some("br"),
            Encoding::Gzip => Some("gzip"),
            Encoding::None => None
        };

        match uri.without_query_parameters() {
            "/" => {
                content_type = "text/html";
                content_encoding = embedded_encoding;
                generated::embedded_files::INDEX_HTML
            },
            "/files/styles.css" => {
                content_type = "text/css";
                content_encoding = embedded_encoding;
                generated::embedded_files::STYLES_CSS
            },
            "/files/script.js" => {
                content_type = "text/javascript";
                content_encoding = embedded_encoding;
                generated::embedded_files::SCRIPT_JS
            },
            "/files/favicon.svg" => {
                content_type = "image/svg+xml";
                content_encoding = embedded_encoding;
                generated::embedded_files::FAVICON_SVG
            },
            #[allow(unused_parens)]
            "/data/timetable.csv" => break 'match_uri ({
                content_type = "text/csv";
                content_encoding = None;
                CsvObject::serialize(
                    configs.get_timetable_csv(),
                    DEFAULT_SEPARATOR,
                    DEFAULT_STR_MARKER
                ).into_bytes()
            }),
            #[allow(unused_parens)]
            "/data/breaks.csv" => break 'match_uri ({
                content_type = "text/csv";
                content_encoding = None;
                CsvObject::serialize(
                    configs.get_breaks_csv(),
                    DEFAULT_SEPARATOR,
                    DEFAULT_STR_MARKER
                ).into_bytes()
            }),
            "/data/utc-offset.bin" => return Response::new(
                200,
                "OK",
                vec![
                    "Content-Type: application/octet-stream".into(),
                    "Content-Length: 1".into()
                ],
                vec![
                    configs.utc_offset() as u8
                ]
            ).unwrap(),
            #[allow(unused_parens)]
            "/data/songs.csv" => break 'match_uri ({
                content_type = "text/csv";
                content_encoding = None;
                CsvObject::serialize(
                    database.get_songs_csv(),
                    DEFAULT_SEPARATOR,
                    DEFAULT_STR_MARKER
                ).into_bytes()
            }),
            // "/data/server-time" => return Response::ok(format!("{}", time::Time::now(configs.utc_offset())).into_bytes()),
            "/data/server-time" => {
                let body = time::Time::now(configs.utc_offset()).display().as_bytes().to_vec();
                return Response::new(200, "OK", vec![
                    "Content-Type: text/plain".into(),
                    format!("Content-Length: {}", body.len()),
                ], body).unwrap()
            },
            "/data/server-time-seconds" => {
                let body = time::Time::now(configs.utc_offset()).elapsed_seconds().to_string().as_bytes().to_vec();
                return Response::new(200, "OK", vec![
                    "Content-Type: text/plain".into(),
                    format!("Content-Length: {}", body.len()),
                ], body).unwrap()
            },
            _ => return Response::not_found(),
        }.to_vec()
    };

    let mut headers = vec![
        format!("Content-Type: {}", content_type),
        format!("Content-Length: {}", body.len()),
    ];

    if let Some(content_encoding) = content_encoding {
        headers.push(format!("Content-Encoding: {}", content_encoding));
    }

    Response::new(200, "OK", headers, body).unwrap()
}

macro_rules! csv_from_utf8_or_return {
    ($bytes:expr, $error:expr) => {
        CsvObject::from_str(
            or_return!(
                str::from_utf8($bytes).ok(),
                $error
            ),
            DEFAULT_SEPARATOR,
            DEFAULT_STR_MARKER
        )
    };
}

macro_rules! first_line_from_utf8_csv_or_return {
    ($bytes:expr, $error:expr) => {
        or_return!(
            csv_from_utf8_or_return!($bytes, $error)
                .into_iter()
                .next(),
            $error
        ).iter().filter_map(|v| v.as_string())
    };
}

fn handle_post(uri: Uri, _headers: Headers, body: Body, database: &mut Database, configs: &mut Configs) -> Response {
    // println!("URI: {:?}\nHeaders:\n{}Body length: {}\n", uri, headers.join("\n"), body.len());

    // println!("{:?}", String::from_utf8(body.clone()).or::<()>(Ok("".to_string())));

    match uri.without_query_parameters() {
        "/api/set-timetable" => {
            or_return!(configs.set_timetable_from_csv(
                csv_from_utf8_or_return!(body.as_slice(), Response::bad_request())
                ),
                Response::bad_request()
            );

            Response::ok("Timetable successfully set".into())
        },
        "/api/set-breaks" => {
            or_return!(
                configs.set_breaks_from_csv(
                    csv_from_utf8_or_return!(body.as_slice(), Response::bad_request())
                ),
                Response::bad_request()
            );

            Response::ok("Timetable successfully set".into())
        },
        "/api/set-utc-offset" => {
            match str::from_utf8(body.as_slice()).ok().and_then(|s| str::parse::<i8>(s).ok()) {
                Some(n @ -12..12) => {
                    unsafe { configs.set_utc_offset_unchecked(n); }
                    Response::ok("UTC offset successfully set".into())
                },
                Some(_) => Response::unprocessable_request(),
                None => Response::bad_request()
            }
        },
        "/api/disable-songs" => {
            let mut success: u16 = 0;

            for name in first_line_from_utf8_csv_or_return!(body.as_slice(), Response::bad_request()) {
                /*let songs = database
                    .inner()
                    .iter()
                    .map(|song| if song.filename() == OsStr::new(name) {
                        let mut s = song.clone();
                        s.set_enabled(false);
                        success += 1;
                        s
                    } else {
                        song.clone()
                    })
                    .collect::<HashSet<_>>();

                *database.inner_mut() = songs;*/
                database
                    .inner_mut()
                    .iter_mut()
                    .for_each(
                        |song| if song.filename() == OsStr::new(name) {
                            song.set_enabled(false)
                        }
                    )
            }

            if success == 0 {
                Response::new(404, "Not Found", Vec::new(), "All requests failed.".as_bytes().to_vec()).unwrap()
            } else {
                Response::ok(format!("{} successfully disabled", success).as_bytes().to_vec())
            }
        },
        "/api/enable-songs" => {
            let mut success: u16 = 0;

            for name in first_line_from_utf8_csv_or_return!(body.as_slice(), Response::bad_request()) {
                database
                    .inner_mut()
                    .iter_mut()
                    .for_each(
                        |song| if song.filename() == OsStr::new(name) {
                            song.set_enabled(true)
                        }
                    )
            }

            if success == 0 {
                Response::new(404, "Not Found", Vec::new(), "All requests failed.".as_bytes().to_vec()).unwrap()
            } else {
                Response::ok(format!("{} successfully enabled", success).as_bytes().to_vec())
            }
        },
        "/api/play-songs" => {
            let mut success: u16 = 0;

            let mut songs = Vec::new();

            for name in first_line_from_utf8_csv_or_return!(body.as_slice(), Response::bad_request()) {
                songs.push(or_continue!(Song::new(Path::new(name))));
                success += 1;
            }

            std::thread::spawn(move || songs::play_playlist(&songs));

            if success == 0 {
                Response::new(404, "Not Found", Vec::new(), "All requests failed.".as_bytes().to_vec()).unwrap()
            } else {
                logln!("Manual play started at {}", time::Time::now(configs.utc_offset()));
                Response::ok(format!("{} successfully played", success).as_bytes().to_vec())
            }
        },
        "/api/delete-songs" => {
            let mut success: u16 = 0;
            let mut error: u16 = 0;

            for name in first_line_from_utf8_csv_or_return!(body.as_slice(), Response::bad_request()) {
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
                    Err(_) => { drop(file); let _ = std::fs::remove_file(file_path); Response::internal_server_error() }
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