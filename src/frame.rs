// TODO: Add more details around the protocol
// Keep the design simple.

// Protocol Design
// Lets create a new protocol with
// client / server calculator
//

use std::{io::Cursor, u64};

use atoi::atoi;
use tokio_util::bytes::Buf;

// A frame for our own protocol.
#[derive(Clone, Debug)]
pub enum Frame {
    Integer(u64),
    StrData(String),
    Array(Vec<Frame>),
}

#[derive(Debug)]
pub enum Error {
    // Not enough data is available
    Incomplete,

    // Non supported encoding.
    ErrMessage(String),
}

impl Frame {
    pub fn array() -> Frame {
        Frame::Array(vec![])
    }

    pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        match get_u8(src)? {
            b'%' => {
                // free form bytes
                get_line(src)?;
                Ok(())
            }
            b':' => {
                let _ = get_number(src)?;
                Ok(())
            }
            default => Err(Error::ErrMessage(format!(
                "protocol error, invalid type byte {}",
                default
            ))),
        }
    }

    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Frame, Error> {
        match get_u8(src)? {
            b':' => {
                let num = get_number(src)?;
                Ok(Frame::Integer(num))
            }
            b'%' => {
                let line = get_line(src)?.to_vec();
                let string = String::from_utf8(line)
                    .map_err(|_| Error::ErrMessage("Failed to parse the string".to_string()))?;
                Ok(Frame::StrData(string))
            }
            _ => !unimplemented!(),
        }
    }
}

fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }
    Ok(src.get_u8())
}

fn get_first_operand(src: &mut Cursor<&[u8]>) -> Result<u64, Error> {
    let start = src.position() as usize;

    // last byte position
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b':' {
            // set the position to `:`
            src.set_position((i + 1) as u64);
            let fbytes = &src.get_ref()[start..i];
            return atoi::<u64>(fbytes).map_or(
                Err(Error::ErrMessage(
                    "Protocol error, invalid frame".to_string(),
                )),
                |v| Ok(v),
            );
        }
    }
    return Err(Error::ErrMessage(
        "Protocol error, invalid frame".to_string(),
    ));
}

fn get_second_operand(src: &mut Cursor<&[u8]>) -> Result<u64, Error> {
    let start = src.position() as usize;

    // last byte position
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            // set the position to `\n`
            src.set_position((i + 2) as u64);
            let fbytes = &src.get_ref()[start..i];
            return atoi::<u64>(fbytes).map_or(
                Err(Error::ErrMessage(
                    "Protocol error, invalid frame".to_string(),
                )),
                |v| Ok(v),
            );
        }
    }
    return Err(Error::ErrMessage(
        "Protocol error, invalid frame".to_string(),
    ));
}

// Get Number
fn get_number(src: &mut Cursor<&[u8]>) -> Result<u64, Error> {
    let line = get_line(src)?;

    atoi::<u64>(line).map_or(
        Err(Error::ErrMessage(
            "Protocol error, invalid frame".to_string(),
        )),
        |v| Ok(v),
    )
}

// Find line terminating character = `<` `>`
fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    let start = src.position() as usize;

    // last byte position
    let end = src.get_ref().len() - 1;

    // Search for the termination pattern
    for i in start..end {
        if src.get_ref()[i] == b'<' && src.get_ref()[i + 1] == b'>' {
            // update the position after `>`
            src.set_position((i + 2) as u64);

            return Ok(&src.get_ref()[start..i]);
        }
    }
    Err(Error::Incomplete)
}

#[test]
fn test_get_operands() {
    let buf = &b"123:456\r\n"[..];
    let mut cursor = Cursor::new(buf);
    let first = get_first_operand(&mut cursor);

    assert_eq!(123, first.unwrap());
    let second = get_second_operand(&mut cursor);

    assert_eq!(456, second.unwrap());
}
