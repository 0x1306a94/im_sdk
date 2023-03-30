use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;

use crate::codec;

use super::callback;
use super::endpoint;
use super::identify_mode;
use super::identify_step;
use super::long_link;
use super::net_source;
use super::short_link;

use im_util::buffer::AutoBuffer;

pub struct Transport {
    source: Arc<Mutex<net_source::NetSource>>,
    callback: Arc<Mutex<Box<dyn callback::Callback>>>,

    long_link_codec: Arc<Mutex<Box<dyn codec::long_link::Codec>>>,
    short_link_codec: Arc<Mutex<Box<dyn codec::short_link::Codec>>>,

    long_link_establish: bool,
    long_link_req_tx: Option<Sender<long_link::LongLinkRequest>>,
    long_link_resp_rx: Option<Receiver<long_link::LongLinkResponse>>,
    long_link_shutdown_tx: Option<Sender<()>>,
}

impl Transport {
    pub fn new(
        long_link_codec: Box<dyn codec::long_link::Codec>,
        short_link_codec: Box<dyn codec::short_link::Codec>,
        callback: Box<dyn callback::Callback>,
    ) -> Self {
        let source = Arc::new(Mutex::new(net_source::NetSource::new()));
        let long_link_codec = Arc::new(Mutex::new(long_link_codec));
        let short_link_codec = Arc::new(Mutex::new(short_link_codec));

        Transport {
            source: source,
            callback: Arc::new(Mutex::new(callback)),
            long_link_codec: long_link_codec,
            short_link_codec: short_link_codec,

            long_link_establish: false,

            long_link_req_tx: None,
            long_link_shutdown_tx: None,
            long_link_resp_rx: None,
        }
    }
}

impl Transport {
    pub async fn makesure_long_link_connected(&mut self) {
        if self.long_link_establish {
            return;
        }

        let (resp_tx, resp_rx) = channel::<long_link::LongLinkResponse>(100);

        let mut long_link =
            long_link::LongLink::new(Arc::clone(&self.long_link_codec), resp_tx.clone());
        self.long_link_req_tx = Some(long_link.req_tx.clone());
        self.long_link_resp_rx = Some(resp_rx);
        self.long_link_shutdown_tx = Some(long_link.shutdown_tx.clone());

        let endpoint = self.source.lock().await.get_long_link_endpoint().unwrap();
        tokio::spawn(async move {
            println!("start....");
            long_link.run(endpoint).await;
        });
    }

    pub async fn recv_response(&mut self) {
        let long_link_resp_rx = &mut self.long_link_resp_rx;
        if let Some(resp_rx) = long_link_resp_rx {
            loop {
                tokio::select! {
                    value = resp_rx.recv() => {
                        println!("resp: {:?}", value);
                        match value.unwrap_or(long_link::LongLinkResponse::None) {
                            long_link::LongLinkResponse::Connecting => {
                                println!("连接中...");
                            },
                            long_link::LongLinkResponse::Connected => {
                                println!("连接成功...");
                            },
                            long_link::LongLinkResponse::ConnectFail => {
                                println!("连接失败...");
                                return ;
                            },
                            long_link::LongLinkResponse::Checkidentify => {

                                let mut callback = self.callback.lock().await;
                                let mut identify_buffer = AutoBuffer::default();
                                let (mode, cmd_id) = callback.get_long_link_identify_check_buffer(&mut identify_buffer);
                                match mode {
                                    identify_mode::Mode::Now => {
                                        if let Some(ref tx) = self.long_link_req_tx {
                                            _ = tx.send(long_link::LongLinkRequest::Checkidentify((identify_buffer, cmd_id))).await;
                                        }
                                    },
                                    _ => {},
                                }
                            },
                            _ => {},
                        }
                    }
                }
            }
        }
    }

    pub async fn long_link_is_connected(&self) -> bool {
        self.long_link_establish
    }

    pub async fn set_long_link_endpoint(&self, endpoint: endpoint::Endpoint) {
        println!("endpoint: {:?}", endpoint);

        self.source.lock().await.set_long_link_endpoint(endpoint);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::transport::endpoint;
    use crate::transport::*;
    use codec::long_link::CmdId;
    use im_util::buffer::AutoBuffer;
    use tokio::runtime::Builder as RuntimeBuilder;

    struct MockCallback {}

    impl callback::Callback for MockCallback {
        fn get_long_link_identify_check_buffer(
            &mut self,
            identify_buffer: &mut AutoBuffer,
        ) -> (identify_mode::Mode, CmdId) {
            (identify_mode::Mode::Never, CmdId(0))
        }

        fn verify_long_link_identify(
            &mut self,
            response_buffer: &AutoBuffer,
        ) -> identify_step::Step {
            identify_step::Step::Ok
        }
    }
    // #[test]
    // fn case_connect() {
    //     let endpoint = endpoint::Endpoint::new("127.0.0.1".into(), 9999);

    //     let long_link_codec = Box::new(codec::long_link::DefaultLongLinkCodec::new(
    //         codec::long_link::DEFAULT_VERSION,
    //     ));

    //     let short_link_codec = Box::new(codec::short_link::DefaultShortLinkCodec::new(
    //         codec::short_link::DEFAULT_VERSION,
    //     ));

    //     let cb = Box::new(MockCallback {});
    //     let mut transport = Transport::new(long_link_codec, short_link_codec, cb);

    //     let rt = RuntimeBuilder::new_multi_thread()
    //         .enable_all()
    //         .thread_name("rnet-thread")
    //         .build()
    //         .unwrap();

    //     rt.block_on(async move {
    //         transport.set_long_link_endpoint(endpoint).await;
    //         transport.makesure_long_link_connected().await;

    //         _ = tokio::spawn(async move {
    //             transport.recv_response().await;
    //         })
    //         .await;
    //     });
    // }
}
