use bytes::Buf;
use std::fmt;
use std::io::Cursor;
use std::num::TryFromIntError;
use std::string::FromUtf8Error;
use tracing::{debug, error};

#[derive(Clone, Debug)]
pub enum Frame {
    Error { msg: String },
    Plate { plate: String, timestamp: u32 },
    WantHeartbeat { interval: u32 },
    IAmCamera { road: u16, mile: u16, limit: u16 },
    IAmDispatcher { roads: Vec<u16> },
}

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
                get_u32(src)?;
                Ok(())
            }
            // Heartbeat (just Server -> Client)
            // 0x41 => {
            //     Ok(())
            // }
            // IAmCamera: road: u16, mile: u16, limit: u16
            0x80 => {
                // road
                get_u16(src)?;
                // mile
                get_u16(src)?;
                // limit
                get_u16(src)?;
                Ok(())
            }
            // IAmDispatcher: numroads: u8, roads: [u16]
            0x81 => {
                // numroads
                let amount = get_u8(src)? * 2;
                // roads
                skip(src, amount as usize)?;
                Ok(())
            }
            actual => Err(format!("protocol error; invalid frame type byte `{}`", actual).into()),
        }
    }

    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Frame, Error> {
        match get_u8(src)? {
            // Error: msg: str
            0x10 => {
                let n = get_length(src)?;
                let msg = get_str(src, n)?.to_string();
                Ok(Frame::Error { msg })
            }
            // Plate: plate: str, timestamp: u32
            0x20 => {
                // Read length character of the plate string
                let n = get_length(src)?;
                // Skip the string to get to the timestamp
                let plate = get_str(src, n)?.to_string();
                // check if valid timestamp
                let timestamp = get_u32(src)?;
                Ok(Frame::Plate { plate, timestamp })
            }
            // Ticket (just Server -> Client)
            // 0x21 => {
            //     Ok(())
            // }
            // Want Heartbeat: interval: u32
            0x40 => {
                let interval = get_u32(src)?;
                Ok(Frame::WantHeartbeat { interval })
            }
            // Heartbeat (just Server -> Client)
            // 0x41 => {
            //     Ok(())
            // }
            // IAmCamera: road: u16, mile: u16, limit: u16
            0x80 => {
                // road
                let road = get_u16(src)?;
                // mile
                let mile = get_u16(src)?;
                // limit
                let limit = get_u16(src)?;
                Ok(Frame::IAmCamera { road, mile, limit })
            }
            // IAmDispatcher: numroads: u8, roads: [u16]
            0x81 => {
                // numroads
                let numroads = get_u8(src)?;
                // roads
                let roads = get_u16_vec(src, numroads as usize)?;

                Ok(Frame::IAmDispatcher { roads })
            }
            actual => Err(format!("protocol error; invalid frame type byte `{}`", actual).into()),
        }
    }
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

fn get_u16_vec<'a>(src: &mut Cursor<&'a [u8]>, len: usize) -> Result<Vec<u16>, Error> {
    if src.remaining() < len {
        return Err(Error::Incomplete);
    }

    let mut roads = Vec::new();

    for _ in 0..len {
        let road = src.get_u16();
        debug!(?road);
        roads.push(road);
    }

    Ok(roads)
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

    Ok(src.get_u8())
}

fn get_u16(src: &mut Cursor<&[u8]>) -> Result<u16, Error> {
    if !src.has_remaining() {
        error!("Incomplete frame");
        return Err(Error::Incomplete);
    }

    Ok(src.get_u16())
}

fn get_u32(src: &mut Cursor<&[u8]>) -> Result<u32, Error> {
    if !src.has_remaining() {
        error!("Incomplete frame");
        return Err(Error::Incomplete);
    }

    Ok(src.get_u32())
}

// Same as get_u8, but the current cursor points to the byte of the length of a message string.
fn get_length(src: &mut Cursor<&[u8]>) -> Result<usize, Error> {
    if !src.has_remaining() {
        error!("Incomplete frame");
        return Err(Error::Incomplete);
    }

    Ok(src.get_u8() as usize)
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
