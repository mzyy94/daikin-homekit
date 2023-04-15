use daikin_homekit::daikin::Daikin;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let timeout = std::time::Duration::new(3, 0);
    let found = Daikin::discovery(timeout).await;
    println!("{:?}", found);
    Ok(())
}
