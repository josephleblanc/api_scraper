use reqwest::header;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    // println!("API_KEY is {}", &env::var("API_KEY").unwrap());

    let mut h = header::HeaderMap::new();
    h.insert(
        "Accept",
        header::HeaderValue::from_static("application/json"),
    );

    let client = reqwest::Client::builder().default_headers(h).build()?;

    let seed = "https://min-api.cryptocompare.com/data/v2/histominute";

    let fsym = "BTC";
    let tsym = "USD";
    let aggregate = "1";
    let limit = "100";
    #[allow(non_snake_case)]
    let toT = "100";
    let api_key = &env::var("API_KEY").unwrap();

    let request_url = format!(
        "{seed}?fsym={fsym}&tsym={tsym}&aggregate={aggregate}&limit={limit}&toT={toT}&api_key={api_key}",
        seed = seed,
        fsym = fsym,
        tsym = tsym,
        aggregate = aggregate,
        limit = limit,
        toT = toT,
        api_key = api_key
    );
    println!("request_url = {}", request_url);

    let response = client.get(request_url).send().await?.text().await?;
    println!("response:\n{}", response);

    Ok(())
}
