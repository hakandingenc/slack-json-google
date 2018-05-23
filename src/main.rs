extern crate rson;
use rson::*;

fn main() {
    let my_string = "{\"text\":\"This is a line of text.\nAnd this is another one.\"}".to_string();
    let my_forwarder = Forwarder::new(
        my_string,
        "https://hooks.slack.com/services/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",
    );
    my_forwarder.send();
}
