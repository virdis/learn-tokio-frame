// TODO: Add more details around the protocol
// Keep the design simple.

// Protocol Design
// Lets create a new protocol client / server calculator
//
// Protocol Details:
// To encode the Addition operation the following bytes
// are sent.
// `+` followed by "{num1}:{num2}\r\n"
// num1 and num2 are numbers represented by `u64`.
// The end of the payload is represented by
// `\r\n`
//
// Similarly to encode the Subtraction operation the following
// bytes are sent.
// `-` followed by "{num1}:{num2}\r\n"
// num1 and num2 are numbers represented by `u64`.
// The end of the payload is represented by
// `\r\n`
//
// Similarly to encode the Multiplication operation the following
// bytes are sent.
// `*` followed by "{num1}:{num2}\r\n"
// num1 and num2 are numbers represented by `u64`.
// The end of the payload is represented by
// `\r\n`
//
use std::{io::Cursor, u64};

use atoi::atoi;
use tokio_util::bytes::Buf;

// A frame for our own protocol.
#[derive(Clone, Debug)]
pub enum Frame {
    Addition(u64, u64),
    Subtraction(u64, u64),
    Multiplication(u64, u64),
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
            b'+' => {
                get_line(src)?;
                Ok(())
            }
            b'-' => {
                get_line(src)?;
                Ok(())
            }
            b'*' => {
                get_line(src)?;
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
            b'+' => {
                let first_opereand = get_first_operand(src)?;
                let second_operand = get_second_operand(src)?;
                Ok(Frame::Addition(first_opereand, second_operand))
            }
            b'-' => {
                let first_opereand = get_first_operand(src)?;
                let second_operand = get_second_operand(src)?;
                Ok(Frame::Subtraction(first_opereand, second_operand))
            }
            b'*' => {
                let first_opereand = get_first_operand(src)?;
                let second_operand = get_second_operand(src)?;
                Ok(Frame::Multiplication(first_opereand, second_operand))
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

// Find line terminating character = `<` `>`
fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    let start = src.position() as usize;

    // last byte position
    let end = src.get_ref().len() - 1;

    // Search for the termination pattern
    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            // update the position after `\n`
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

#[test]
fn test_parse() {
    let buf = &b"+123:456\r\n"[..];

    let mut cursor = Cursor::new(buf);
    let frame = Frame::parse(&mut cursor);
    assert!(frame.is_ok());
}

#[test]
fn test_parse_fail() {
    let buf = &b"+123456\r\n"[..];

    let mut cursor = Cursor::new(buf);
    let frame = Frame::parse(&mut cursor);
    assert!(frame.is_err());
}
