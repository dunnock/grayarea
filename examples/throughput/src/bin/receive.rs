use grayarea::{set_message_handler, MessageHandler, Result};
use std::time::{Instant};

struct Processor(usize, Instant);

impl MessageHandler for Processor {
    fn on_message(&mut self, message: &[u8]) ->  Result<()>{
        self.0 += 1;
        if message[0] == b'F' {
            println!("Processed {} messages in {} ms", self.0, self.1.elapsed().as_millis());
            panic!("halt receiver");
        };
        if self.0 % 100_000 == 0 {
            println!("Processed {} messages", self.0);
        }
        Ok(())
    }
}

fn main() {
    let started = Instant::now();
    set_message_handler(Box::new(Processor(0, started)));
}
