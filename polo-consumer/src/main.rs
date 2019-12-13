use grayarea_lib::WebSocket;
use poloniex::data::messages::BookUpdate;
use std::str::FromStr;

fn main() {
    let pair = std::env::args().nth(0).unwrap();
    let subscription = format!(
        "{{ \"command\": \"subscribe\", \"channel\": \"{}\" }}",
        pair
    );
    WebSocket::send_message(subscription.as_bytes());
}

#[no_mangle]
fn on_message(ptr: *const u8, len: i32) {
    if ptr.is_null() {
        panic!("null pointer passed to on_message");
    }
    let msg = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
    let msg_str = unsafe { std::str::from_utf8_unchecked(msg) };
    let bu = BookUpdate::from_str(msg_str);
    println!("{:?}", bu);
}
