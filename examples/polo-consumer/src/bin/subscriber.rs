use grayarea::{WebSocket};

fn main() {
    let pair = std::env::args().nth(0).unwrap();
    let subscription = format!(
        "{{ \"command\": \"subscribe\", \"channel\": \"{}\" }}",
        pair
    );
    WebSocket::send_message(subscription.as_bytes());
}
