use std::error::Error;
use std::io;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::time::{sleep, Duration};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

use im_codec::long_link::ParseHeaderError;

mod connection;
mod frame;

use connection::Connection;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:9999").await.unwrap();
    println!("accept...");
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            _ = process(stream).await;
        });
    }
}

async fn process(stream: TcpStream) -> Result<(), ParseHeaderError> {
    println!("accept: {:?}", stream);
    let mut connection = Connection::new(stream);

    loop {
        let maybe_frame = connection.read_frame().await?;

        let frame = match maybe_frame {
            Some(frame) => frame,
            None => return Ok(()),
        };

        println!("frame: {:?}", frame);
    }

    Ok(())
}
