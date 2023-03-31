use im_codec;
use im_net::{endpoint, Transport};
use tokio;

mod callback;
use callback::TCB;

#[tokio::main]
async fn main() {
    let endpoint = endpoint::Endpoint::new("127.0.0.1".into(), 9999);

    let long_link_codec = Box::new(im_codec::long_link::DefaultCodec::new(
        im_codec::long_link::DEFAULT_VERSION,
    ));

    let short_link_codec = Box::new(im_codec::short_link::DefaultCodec::new(
        im_codec::short_link::DEFAULT_VERSION,
    ));

    let cb = Box::new(TCB {});
    let mut transport = Transport::new(long_link_codec, short_link_codec, cb);

    transport.set_long_link_endpoint(endpoint).await;

    transport.makesure_long_link_connected().await;

    transport.recv_response().await;
}
