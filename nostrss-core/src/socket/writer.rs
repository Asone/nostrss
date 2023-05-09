use std::os::unix;
use tokio::{
    io::AsyncReadExt,
    net::{UnixListener, UnixStream},
};

pub struct SocketWriter {}

impl SocketWriter {
    pub async fn send(data: String) {
        let socket_path = "nostrss-socket";
        let mut unix_writer = UnixStream::connect(socket_path).await;

        let unix_listener = match UnixListener::bind(socket_path) {
            Ok(listener) => listener,
            Err(e) => {
                log::error!("{}", e);
                panic!("Socket listener could not be ignited");
            }
        };

        match unix_writer {
            Ok(writer) => {
                _ = writer.try_write(b"Toto");
            }
            Err(e) => {}
        }
    }
}
