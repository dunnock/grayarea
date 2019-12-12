use grayarea_lib::WebSocket;

fn main() {
    let message = b"my message";
    WebSocket::send_message(message);
}
