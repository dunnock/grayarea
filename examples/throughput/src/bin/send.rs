use grayarea::{channel};
use std::time::{Instant};
use rand::{thread_rng, Rng};
use rand::distributions::Standard;

fn main() {
    let n: usize = std::env::args().nth(0).unwrap().parse().unwrap();
    let size: usize = std::env::args().nth(1).unwrap().parse().unwrap();
    let data: Vec<u8> = thread_rng()
        .sample_iter(&Standard)
        .take(size)
        .collect();
    let mut msg = channel::Message { topic: 0, data };
    msg.data[0] = b'S';
    let started = Instant::now();
    for _ in 0..n {
        channel::Channel::send_message(&msg);
    };
    msg.data[0] = b'F';
    channel::Channel::send_message(&msg);
    let ms = started.elapsed().as_millis();
    println!("Sent {} messages in {} ms", n+1, ms);
    println!("Message size {} speed {} MiB/s", size, size as u128 * n as u128 * 8_000 / ms / 1024 / 1024);
}
