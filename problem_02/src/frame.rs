use bytes::Buf;
use std::fmt;
use std::io::Cursor;
use std::num::TryFromIntError;
use std::string::FromUtf8Error;
use tracing::{debug, error, info};

#[derive(Clone, Debug)]
pub enum Frame {
    Insert { timestamp: i32, price: i32 },
    Query { mintime: i32, maxtime: i32 },
}

#[derive(Debug)]
pub enum Error {
    Incomplete,
    Other(crate::Error),
}

impl Frame {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        info!("Check frame");
        match get_u8(src)? {
            b'I' => {
                get_line(src)?;
                Ok(())
            }
            b'Q' => {
                get_line(src)?;
                Ok(())
            }
            actual => Err(format!("protocol error; invalid frame type byte `{}`", actual).into()),
        }
    }

    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Frame, Error> {
        info!("Parsing frame");
        match get_u8(src)? {
            b'I' => {
                info!("Insert message");
                let line = get_line(src)?;
                debug!(?line);
                Ok(Frame::Insert {
                    timestamp: get_decimal(&line[1..=4])?,
                    price: get_decimal(&line[5..=8])?,
                })
            }
            b'Q' => {
                let line = get_line(src)?;

                Ok(Frame::Query {
                    mintime: get_decimal(&line[1..=4])?,
                    maxtime: get_decimal(&line[5..=8])?,
                })
            }
            _ => unimplemented!(),
        }
    }
}

fn get_decimal(src: &[u8]) -> Result<i32, Error> {
    debug!(?src);

    if let Ok(number) = <[u8; 4]>::try_from(src) {
        return Ok(i32::from_be_bytes(number));
    };

    Err("protocol error; invalid frame format".into())
}

fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        error!("Incomplete frame");
        return Err(Error::Incomplete);
    }

    Ok(src.get_u8())
}

fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    if src.get_ref().len() == 9 {
        src.set_position(9);
        return Ok(&src.get_ref()[..]);
    }

    Err(Error::Incomplete)
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
