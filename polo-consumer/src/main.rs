use grayarea_lib::WebSocket;

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
    println!("WS: {}", std::str::from_utf8(msg).expect("text message"));
}
