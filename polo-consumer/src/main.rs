use grayarea_lib::{WebSocket, MessageHandler};
use poloniex::data::messages::BookUpdate;
use std::str::FromStr;

struct Processor(usize);

impl MessageHandler for Processor {
    fn on_message(&mut self, message: &[u8]) -> std::io::Result<()> {
        self.0 += 1;
        let msg_str = std::str::from_utf8(message)
            .map_err(|err| std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Cannot convert websocket message: {}", err),
            ))?;
        let bu = BookUpdate::from_str(msg_str);
        println!("{:?}", bu);
        Ok(())
    }
}

fn main() {
    let pair = std::env::args().nth(0).unwrap();
    let subscription = format!(
        "{{ \"command\": \"subscribe\", \"channel\": \"{}\" }}",
        pair
    );
    WebSocket::send_message(subscription.as_bytes());
    let handler = Processor(0);
    WebSocket::set_message_handler(Box::new(handler));
}
