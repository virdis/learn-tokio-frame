use crate::frame::{self, Frame};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

use std::io::{self, Cursor, ErrorKind};
use tokio_util::bytes::{Buf, BytesMut};

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
    pub fn parse_frame(&mut self) -> crate::Result<Option<Frame>> {
        use frame::Error::Incomplete;
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
            Err(Incomplete) => Ok(None),

            Err(e) => Err(e.into()),
        }
    }

    pub async fn read_frame(&mut self) -> crate::Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            // There is not enough data to read a frame. Attempt to
            // read more data from the socket.
            //
            // `0` returned means end of the stream.
            if 0 == self
                .stream
                .read_buf(&mut self.buffer)
                .await
                .map_or(Err("failed to read from socket".to_string()), |v| Ok(v))?
            {
                // The remote closed the connection. For this to be a clean shutdown
                // no data should be in the buffer. If there is data, that means
                // the peer closed the socket while sending the frame.
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    // TODO: cleanup and refactor the internal of each match arm
    pub async fn write_frame(&mut self, frame: &Frame) -> Result<(), crate::Error> {
        match frame {
            Frame::Addition(x, y) => {
                self.stream.write_u8(b'+').await.map_or(
                    Err::<(), crate::Error>("(+) failed to write byte".into()),
                    |v| Ok(v),
                )?;
                let data = format!("{}:{}\r\n", x, y);
                self.stream.write_all(data.as_bytes()).await.map_or(
                    Err::<(), crate::Error>("(+) failed to write all bytes".into()),
                    |v| Ok(v),
                )?;
            }
            Frame::Subtraction(x, y) => {
                self.stream.write_u8(b'-').await.map_or(
                    Err::<(), crate::Error>("(-) failed to write byte".into()),
                    |v| Ok(v),
                )?;
                let data = format!("{}:{}\r\n", x, y);
                self.stream.write_all(data.as_bytes()).await.map_or(
                    Err::<(), crate::Error>("(-) failed to write all bytes".into()),
                    |v| Ok(v),
                )?;
            }
            Frame::Multiplication(x, y) => {
                self.stream.write_u8(b'*').await.map_or(
                    Err::<(), crate::Error>("(*) failed to write byte".into()),
                    |v| Ok(v),
                )?;
                let data = format!("{}:{}\r\n", x, y);
                self.stream.write_all(data.as_bytes()).await.map_or(
                    Err::<(), crate::Error>("(*) failed to write all bytes".into()),
                    |v| Ok(v),
                )?;
            }
            Frame::OpResult(r) => {
                self.stream.write_u8(b'=').await.map_or(
                    Err::<(), crate::Error>("(=) failed to write all bytes".into()),
                    |v| Ok(v),
                )?;
                self.stream.write_u64(*r).await.map_or(
                    Err::<(), crate::Error>("(=) failed to write all bytes".into()),
                    |v| Ok(v),
                )?;
            }
        }
        // write the encoded frame to socket
        self.stream.flush().await.map_or(
            Err(Box::new(io::Error::new(ErrorKind::Other, "oh no!"))),
            |v| Ok(v),
        )
    }
}
