use std::io;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::time::{sleep, Duration};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

mod connection;

use connection::Connection;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:9999").await.unwrap();
    println!("accept...");
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(stream).await;
        });
    }
}

async fn process(stream: TcpStream) {
    // let mut connection = Connection::new(socket);

    // connection.run_read_write().await;

    let mut stream = stream;
    println!("{:?}", stream);

    let (mut read_half, mut write_half) = stream.split();

    loop {
        tokio::select! {
            value = read_half.readable() => {
                println!("readable: {:?}", value);
                match value {
                    Ok(_) => {
                        if !do_read(&mut read_half) {
                            println!("连接断开...");
                            return ;
                        }
                    },
                    Err(ref e) => match e.kind() {
                        io::ErrorKind::BrokenPipe => {
                            println!("连接断开...");
                            return;
                        },
                        _ => {}
                    },
                    Err(_) => {},
                }

            },
            value = write_half.writable() => {
                println!("writable: {:?}", value);

            }

        }
    }
}

fn do_read(read_half: &mut ReadHalf) -> bool {
    let mut buf = [0u8; 10];
    match read_half.try_read(&mut buf) {
        Ok(read_len) => {
            if read_len == 0 {
                return false;
            }
            println!("read: {}", read_len);
        }
        Err(ref e) => match e.kind() {
            io::ErrorKind::BrokenPipe => {
                println!("连接断开...");
                return false;
            }
            _ => {}
        },
        Err(_) => {}
    }

    true
}

fn do_write(write_half: &mut WriteHalf) -> bool {
    match write_half.try_write(b"hello wolrd") {
        Ok(write_len) => {
            println!("write: {:?}", write_len);
        }
        Err(ref e) => match e.kind() {
            io::ErrorKind::BrokenPipe => {
                println!("连接断开...");
                return false;
            }
            _ => {}
        },
        Err(e) => {}
    }

    true
}
