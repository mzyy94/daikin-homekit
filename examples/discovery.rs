use daikin_homekit::daikin::Daikin;
use daikin_homekit::error::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let timeout = std::time::Duration::new(3, 0);
    let found = Daikin::discovery(timeout).await;
    dbg!(found);
    Ok(())
}
