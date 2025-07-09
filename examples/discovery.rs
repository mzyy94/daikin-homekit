use daikin_homekit::discovery::discovery;
use futures::{pin_mut, prelude::*};

#[tokio::main]
async fn main() {
    let timeout = std::time::Duration::new(3, 0);
    let stream = discovery(timeout).await;
    pin_mut!(stream);
    while let Some(item) = stream.next().await {
        match item {
            Ok(found) => {
                println!("Discovered Daikin device: {found:?}");
            }
            Err(e) => {
                if let Some(elapsed) = e.downcast_ref::<tokio::time::error::Elapsed>() {
                    println!("Discovery finished: {elapsed}");
                } else {
                    println!("Error during discovery: {e}");
                }
            }
        }
    }
}
