use std::borrow::Borrow;
use std::sync::Arc;

use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use tokio::net::TcpStream;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;

use super::endpoint;
use crate::codec::long_link;
use crate::codec::long_link::CmdId;

use im_util::buffer::AutoBuffer;

#[derive(Debug)]
pub(crate) enum LongLinkRequest {
    Checkidentify((AutoBuffer, CmdId)),
    NoCheckidentify,
}

unsafe impl Send for LongLinkRequest {}

#[derive(Debug)]
pub(crate) enum LongLinkResponse {
    None,
    Connecting,
    Connected,
    Disconnected,
    ConnectFail,
    Checkidentify,
}

pub(crate) struct LongLink {
    pub(crate) req_tx: Sender<LongLinkRequest>,
    req_rx: Receiver<LongLinkRequest>,
    resp_tx: Sender<LongLinkResponse>,
    pub(crate) shutdown_tx: Sender<()>,
    shutdown_rx: Receiver<()>,
    codec: Arc<Mutex<Box<dyn long_link::Codec>>>,
    available: bool,

    identify_buffers: Vec<(AutoBuffer, CmdId)>,
}

impl LongLink {
    pub(crate) fn new(
        codec: Arc<Mutex<Box<dyn long_link::Codec>>>,
        resp_tx: Sender<LongLinkResponse>,
    ) -> Self {
        let (req_tx, req_rx) = channel(100);
        let (shutdown_tx, shutdown_rx) = channel(1);
        LongLink {
            req_tx: req_tx,
            req_rx: req_rx,
            resp_tx: resp_tx,
            shutdown_tx: shutdown_tx,
            shutdown_rx: shutdown_rx,
            codec: codec,
            available: false,
            identify_buffers: vec![],
        }
    }
}

impl LongLink {
    pub(crate) async fn run(&mut self, endpoint: endpoint::Endpoint) {
        let addr = format!("{}:{}", endpoint.get_host(), endpoint.get_port());
        println!("addr: {}", addr);

        _ = self.resp_tx.send(LongLinkResponse::Connecting).await;

        println!("Connecting...");
        let mut stream = match TcpStream::connect(addr).await {
            Ok(s) => s,
            Err(_) => {
                _ = self.resp_tx.send(LongLinkResponse::ConnectFail).await;
                return;
            }
        };

        _ = self.resp_tx.send(LongLinkResponse::Checkidentify).await;

        let (mut read_half, mut write_half) = stream.into_split();

        let mut recv_buffer = AutoBuffer::default();

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    _ = self.resp_tx.send(LongLinkResponse::Disconnected).await;
                    return;
                },
                req = self.req_rx.recv() => {
                    println!("req: {:?}", req);
                    match req {
                        Some(v) => match v {
                            LongLinkRequest::Checkidentify(buffer) => {
                                self.identify_buffers.push(buffer);
                            },
                            LongLinkRequest::NoCheckidentify => {
                                self.identify_buffers.clear();
                               _ = self.resp_tx.send(LongLinkResponse::Connected).await;
                            },
                        },
                        _ => {},
                    }
                },
                _ = read_half.readable() => {
                    if !self.do_read(&mut recv_buffer, &mut read_half).await {
                        _ = self.resp_tx.send(LongLinkResponse::Disconnected).await;
                        return;
                    }
                },
                _ = write_half.writable() => {
                    self.do_write(&mut write_half).await;
                },
            }
        }
    }

    async fn do_read(
        &mut self,
        recv_buffer: &mut AutoBuffer,
        read_half: &mut OwnedReadHalf,
    ) -> bool {
        let max_size: usize = 64 * 1024;
        recv_buffer.add_capacity(max_size);
        let mut buffer = BytesMut::with_capacity(max_size);
        match read_half.read_buf(&mut buffer).await {
            Ok(read_len) => {
                recv_buffer.write(&buffer[..read_len]);
                return self.try_decode(recv_buffer).await;
            }
            Err(e) => {
                eprintln!("error: {:?}", e);
                return false;
            }
        };

        true
    }

    async fn do_write(&mut self, write: &mut OwnedWriteHalf) {
        if !self.identify_buffers.is_empty() {
            self.do_write_identify(write).await;
            return;
        }
    }

    async fn do_write_identify(&mut self, write: &mut OwnedWriteHalf) {
        println!("do_write_identify...");

        let codec = self.codec.lock().await;

        loop {
            if !self.identify_buffers.is_empty() {
                let value = self.identify_buffers.remove(0);

                let mut out_buffer = AutoBuffer::default();
                let extend_buffer = AutoBuffer::default();
                codec.encode(
                    &value.1,
                    &long_link::IDENTIFY_CHECKER_TASK_ID,
                    &value.0,
                    &extend_buffer,
                    &mut out_buffer,
                );

                let mut bytes = out_buffer.as_slice(0);
                // let mut buffer = Vec::<u8>::with_capacity(bytes.len());
                // buffer.extend_from_slice(bytes);

                // use std::io::Cursor;

                // let mut buffer = Cursor::new(Vec::new());
                // buffer.write_all(bytes);

                match write.write_buf(&mut bytes).await {
                    Ok(write_len) => {
                        println!("write len: {}", write_len);
                    }
                    Err(_) => {}
                }
            }
        }
    }

    async fn try_decode(&mut self, recv_buffer: &mut AutoBuffer) -> bool {
        let mut body_buffer = AutoBuffer::default();
        let mut extend_buffer = AutoBuffer::default();

        let mut codec = self.codec.lock().await;
        let (status, cmd_id, task_id, package_len) =
            codec.decode(recv_buffer, &mut body_buffer, &mut extend_buffer);

        match status {
            long_link::DecodeStatus::Continue => {}
            long_link::DecodeStatus::Fail => {
                return false;
            }
            long_link::DecodeStatus::Ok => {
                println!("try_decode: {:?} {:?} {}", cmd_id, task_id, package_len);
            }
        }

        true
    }
}
