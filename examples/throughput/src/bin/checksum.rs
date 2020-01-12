use grayarea::{set_message_handler, MessageHandler, Result};
use std::time::{Instant};

struct Processor(usize, usize, Instant);

impl MessageHandler for Processor {
    fn on_message(&mut self, message: &[u8]) ->  Result<()>{
        let chksum = message.iter().fold(0usize, |acc, i| acc + *i as usize);
        if self.0 == 0 {
            self.1 = chksum;
        } else if chksum != self.1 {
            panic!("checksum failure for message {}", self.0);
        };
        if self.0 % 100_000 == 0 {
            println!("Processed {} messages", self.0);
        };
        self.0 += 1;
        Ok(())
    }
}

fn main() {
    let started = Instant::now();
    set_message_handler(Box::new(Processor(0, 0, started)));
}
