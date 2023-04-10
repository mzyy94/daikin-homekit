#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match std::env::args().nth(1) {
        Some(ip_addr) => get_status(ip_addr).await,
        None => Ok(()),
    }
}

async fn get_status(ip_addr: String) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .http1_title_case_headers()
        .build()?;
    let url = format!("http://{}/dsiot/multireq", ip_addr);
    let resp = client.post(url).send().await?;

    if resp.status() != reqwest::StatusCode::OK {
        println!("{}", resp.status());
        return Ok(());
    }

    let body = resp.text().await?;
    println!("{}", body);

    Ok(())
}
