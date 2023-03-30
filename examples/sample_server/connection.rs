use tokio::net::TcpStream;

#[derive(Debug)]
pub(crate) struct Connection {
    pub(crate) stream: TcpStream,
}

impl Connection {
    pub(crate) fn new(stream: TcpStream) -> Self {
        Connection { stream: stream }
    }
}
