use api_scraper::Data;
use csv::Writer;
use reqwest::header;
use serde_json::Value;
use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::time::{Duration, SystemTime};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    let datadir = "/home/brasides/programming/data/BTC_historic_minute";
    // Roundabout way of cleaning old file, since you cannot truncate on a file you open with
    // append.
    let _temp_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(format!("{}/{}", datadir, "test_data.csv"));
    // Open file for the csv writer, set to append.
    let file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(format!("{}/{}", datadir, "test_data.csv"))?;
    // Configure the csv writer, taking the file descriptor from above so it will append instead of
    // write over the current contents.
    let mut wtr = Writer::from_writer(file);

    let mut h = header::HeaderMap::new();
    h.insert(
        "Accept",
        header::HeaderValue::from_static("application/json"),
    );
    let client = reqwest::Client::builder().default_headers(h).build()?;
    let seed = "https://min-api.cryptocompare.com/data/v2/histominute";
    // Symbol of the cryptocurrency to be priced
    let fsym = "BTC";
    // Symbol to show the value of the crypto
    let tsym = "USD";
    // aggregate is the time interval variable, where 1 is 1 minute
    // min is 1, max is 30
    let aggregate = "1";
    // limit is number of data points to be queried for, from 1-2000
    let limit = "2000";
    // toT is how far back from the current time to query for
    #[allow(non_snake_case)]
    let mut toTs: String = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs()
        .to_string();
    // my api key
    let api_key = &env::var("API_KEY").unwrap();

    let mut data_response: Vec<Data>;
    for _ in 0..=5 {
        let request_url = format!(
        "{seed}?fsym={fsym}&tsym={tsym}&aggregate={aggregate}&limit={limit}&toTs={toTs}&api_key={api_key}",
        seed = seed,
        fsym = fsym,
        tsym = tsym,
        aggregate = aggregate,
        limit = limit,
        toTs = toTs,
        api_key = api_key
    );
        println!("request_url = {}", request_url);

        let response = client.get(request_url).send().await?.text().await?;
        let structured_response: Value = serde_json::from_str(&response).unwrap();
        data_response =
            serde_json::from_value(structured_response["Data"]["Data"].to_owned()).unwrap();
        for line in data_response.iter().rev() {
            println!("{:?}", line.time);
            wtr.serialize(line)?;
        }
        toTs = (data_response[0].time - 60).to_string();
    }
    //let response_data: Vec<Data> = serde_json::from_value(structured_response).unwrap();

    //for line in response_data {
    //    println!("{:?}", line);
    //}

    Ok(())
}
