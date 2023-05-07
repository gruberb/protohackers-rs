use bytes::Buf;
use std::fmt;
use std::io::Cursor;
use std::num::TryFromIntError;
use std::string::FromUtf8Error;
use tracing::{debug, error, info};

#[derive(Clone, Debug)]
pub enum Frame {}

#[derive(Debug)]
pub enum Error {
    Incomplete,
    Other(crate::Error),
}

impl Frame {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        match get_u8(src)? {
            // Error: msg: str
            0x10 => {
                let n = get_length(src)?;
                skip(src, n as usize)
            }
            // Plate: plate: str, timestamp: u32
            0x20 => {
                // Read length character of the plate string
                let n = get_length(src)?;
                // Skip the string to get to the timestamp
                skip(src, n)?;
                // check if valid timestamp
                get_u32(src)?;
                Ok(())
            }
            // Ticket (just Server -> Client)
            // 0x21 => {
            //     Ok(())
            // }
            // Want Heartbeat: interval: u32
            0x40 => {
                unimplemented!()
            }
            // Heartbeat (just Server -> Client)
            // 0x41 => {
            //     Ok(())
            // }
            // IAmCamera: road: u16, mile: u16, limit: u16
            0x80 => {
                unimplemented!()
            }
            // IAmDispatcher: numroads: u8, numroads: [u16]
            0x81 => {
                unimplemented!()
            }
            actual => Err(format!("protocol error; invalid frame type byte `{}`", actual).into()),
        }
    }

    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Frame, Error> {
        unimplemented!()
    }
}

fn peek_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }

    Ok(src.chunk()[0])
}

fn get_str<'a>(src: &mut Cursor<&'a [u8]>, len: usize) -> Result<&'a str, Error> {
    if src.remaining() < len {
        return Err(Error::Incomplete);
    }

    let position = src.position() as usize;
    let slice = &src.get_ref()[position..position + len];

    let message =
        std::str::from_utf8(slice).map_err(|_| "protocol error; invalid frame format".into());

    src.advance(len);

    message
}

fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<(), Error> {
    if src.remaining() < n {
        return Err(Error::Incomplete);
    }

    src.advance(n);
    Ok(())
}

fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        error!("Incomplete frame");
        return Err(Error::Incomplete);
    }

    info!("get_u8: current cursor position: {:?}", src.position());

    Ok(src.get_u8())
}

fn get_u32(src: &mut Cursor<&[u8]>) -> Result<u32, Error> {
    if !src.has_remaining() {
        error!("Incomplete frame");
        return Err(Error::Incomplete);
    }

    info!("get_u32: current cursor position: {:?}", src.position());

    Ok(src.get_u32())
}

// Same as get_u8, but the current cursor points to the byte of the length of a message string.
fn get_length(src: &mut Cursor<&[u8]>) -> Result<usize, Error> {
    if !src.has_remaining() {
        error!("Incomplete frame");
        return Err(Error::Incomplete);
    }

    info!("get_length: current cursor position: {:?}", src.position());

    Ok(src.get_u8() as usize)
}

fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    unimplemented!()
}

impl From<String> for Error {
    fn from(src: String) -> Error {
        Error::Other(src.into())
    }
}

impl From<&str> for Error {
    fn from(src: &str) -> Error {
        src.to_string().into()
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_src: FromUtf8Error) -> Error {
        "protocol error; invalid frame format".into()
    }
}

impl From<TryFromIntError> for Error {
    fn from(_src: TryFromIntError) -> Error {
        "protocol error; invalid frame format".into()
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Incomplete => "stream ended early".fmt(fmt),
            Error::Other(err) => err.fmt(fmt),
        }
    }
}
