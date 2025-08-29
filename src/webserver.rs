use std::convert::Infallible;
use std::net::{TcpListener, ToSocketAddrs, TcpStream};
use std::io::{BufReader, BufRead, Write, Read};

use crate::{is_kind_of, or_continue};

#[derive(Debug)]
pub enum Error {
    CannotBind,
    InvalidRequest
}

type Headers = Vec<String>;
// type Body = Vec<u8>;
type Body = String;

pub enum Request {
    Get { uri: Uri, headers: Headers },
    Post { uri: Uri, headers: Headers, body: Body },
}

impl TryFrom<TcpStream> for Request {
    type Error = Error;
    fn try_from(stream: TcpStream) -> Result<Self, Error> {
        let mut error = false;
        let lines = BufReader::new(stream)
            .lines()
            .filter_map(|v| match v {
                Ok(x) => Some(x),
                Err(_) => {
                    error = true;
                    None
                }
            })
            .collect::<Vec<_>>();

        if error {
            return Err(Error::InvalidRequest);
        }

        let [method, uri, _version] = {
            let x: [&str; 3] = lines
                .get(0)
                .ok_or(Error::InvalidRequest)?
                .splitn(3, ' ')
                .collect::<Vec<&str>>()
                .try_into()
                .map_err(|_| Error::InvalidRequest)?;
            x
        };

        Ok(match method {
            "GET" => Request::Get {
                uri: uri.into(),
                headers: lines.get(1..).ok_or(Error::InvalidRequest)?.to_vec()
            },
            "POST" => Request::Post {
                uri: uri.into(),
                headers: lines.get(1..(lines.len() - 1)).ok_or(Error::InvalidRequest)?.to_vec(),
                body: lines.get(lines.len()-1).ok_or(Error::InvalidRequest)?.to_string()
            },
            _ => return Err(Error::InvalidRequest)
        })
    }
}

pub struct Uri(pub Box<str>);

impl<T: Into<Box<str>>> From<T> for Uri {
    fn from(s: T) -> Uri {
        Uri(s.into())
    }
}

pub struct Response {
    status_code: u8,
    headers: Headers,
    body: Body
}

pub fn listen<A: ToSocketAddrs, H: Fn(Request) -> Response>(addr: A, handler: H) -> Result<Infallible, Error> {
    let listener = TcpListener::bind(addr)/*.map_err(|_| Error::CannotBind)?*/.unwrap();

    for stream in listener.incoming() {
        let stream = or_continue!(stream.ok());

        handler(or_continue!(stream.try_into().ok()));
    }

    unreachable!()
}

pub fn handle_connection(request: Request) -> Response {
    todo!()
}