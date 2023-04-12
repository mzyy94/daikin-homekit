use daikin_homekit::daikin::Daikin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let timeout = std::time::Duration::new(3, 0);
    let found = Daikin::discovery(timeout).await;
    dbg!(found);
    Ok(())
}
