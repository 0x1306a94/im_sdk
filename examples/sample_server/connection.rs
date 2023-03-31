use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, BufWriter};
use tokio::net::TcpStream;

use super::frame::Frame;

use im_codec::long_link::{MsgHeader, ParseHeaderError, DEFAULT_HEADER_LEN};

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4096),
        }
    }
}

impl Connection {
    pub async fn read_frame(&mut self) -> Result<Option<Frame>, ParseHeaderError> {
        loop {
            if let Some(frame) = self.pares_frame()? {
                return Ok(Some(frame));
            }

            if let Ok(0) = self.stream.read_buf(&mut self.buffer).await {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    // TODO:
                    return Err(ParseHeaderError::VersionMismatch);
                }
            }
        }
    }

    async fn write_frame(&mut self) {}
}

impl Connection {
    fn pares_frame(&mut self) -> Result<Option<Frame>, ParseHeaderError> {
        let bytes = &self.buffer[..];
        match MsgHeader::try_from(bytes) {
            Ok(header) => {
                self.buffer.advance(DEFAULT_HEADER_LEN);
                let body = &self.buffer[..(header.body_len as usize)];
                let bytes = bytes::Bytes::copy_from_slice(body);
                return Ok(Some(Frame::new(header.cmd_id, header.task_id, bytes)));
            }
            Err(ParseHeaderError::Underlength) => {
                return Ok(None);
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
}
