use grayarea_lib::{WebSocket, MessageHandler, Result};
use poloniex::data::messages::BookUpdate;
use std::str::FromStr;

struct Processor(usize);

impl MessageHandler for Processor {
    fn on_message(&mut self, message: &[u8]) ->  Result<()>{
        self.0 += 1;
        let msg_str = std::str::from_utf8(message)?;
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
    WebSocket::set_message_handler(Box::new(Processor(0)));
}
