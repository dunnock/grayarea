use grayarea_lib::WebSocket;

fn main() {
    let message = std::env::args().nth(0).unwrap();
    WebSocket::send_message(message.as_bytes());
}
