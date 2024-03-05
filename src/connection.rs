use std::io::Cursor;

use tokio::{io::BufWriter, net::TcpStream, select};

use tokio_util::bytes::{Buf, BytesMut};

use crate::{frame::Error, Frame};

// Send and recieve `Frame` values from a remte peer.
//
// To read frames, `Connection` uses an internal buffer, which is
// filled up until there are enough bytes to create a full frame.
//
// When sending frames, the frame is first encoded into the write
// buffer. The contents of the write buffer are then written to
// the socket.
#[derive(Debug)]
pub struct Connection {
    // The `TcpStream` is decorated with a `BufWriter`, which provides
    // write level buffering.
    stream: BufWriter<TcpStream>,

    // The buffer for reading frames.
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection {
            stream: BufWriter::new(stream),

            // Default 4KB read buffer, this is ok for our
            // use case.
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    // Tries to parse the frame, if the buffer does not contain
    // enough data , `Ok(None)` is returned. If there is an
    // invalid frame and Err is returned.
    pub fn parse_frame(&mut self) -> Result<Option<Frame>, Error> {
        // Cursor is used to track the current location in the buffer.
        let mut buf = Cursor::new(&self.buffer[..]);

        // Check if enough data has been buffered to  parse a single frame.
        // If enough data is not present we can skip allocating.
        match Frame::check(&mut buf) {
            Ok(_) => {
                // `check` function will advance the cursor until the end of the
                // frame. Since the cursor has position set to zero before
                // `Frame::check` was called, we get the length of the frame
                // by checking the cursor position.
                let len = buf.position() as usize;

                // We have enough data in the buffer to parse the frame.
                // lets' reset the position and call `Frame::parse`
                buf.set_position(0);

                // Parse the frame, if the encoded frame is invalid an
                // error is returned.
                let frame = Frame::parse(&mut buf)?;

                // Parsing the frame succeded, let discard the parsed data.
                // Calling advance will discard the data.
                self.buffer.advance(len);

                // Return parsed frame.
                Ok(Some(frame))
            }
            Err(Error::Incomplete) => Ok(None),

            Err(e) => Err(e.into()),
        }
    }
}
