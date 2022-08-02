use csv::Writer;
use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::error::Error;
use std::fs::OpenOptions;
// use serde_json::Result;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Data {
    pub time: u64,
    pub high: f32,
    pub low: f32,
    pub open: f32,
    pub volumefrom: f32,
    pub volumeto: f32,
    pub close: f32,
    pub conversionType: String,
    pub conversionSymbol: Option<String>,
}

pub async fn scrape(
    datadir: &str,
    file_name: &str,
    start_time: &str,
) -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    let full_file_name = format!("{}/{}", datadir, file_name);
    // Roundabout way of cleaning old file, since you cannot truncate on a file you open with
    // append.
    let _temp_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&full_file_name);
    // Open file for the csv writer, set to append.
    let file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(&full_file_name)?;
    // Configure the csv writer, taking the file descriptor from above so it will append instead of
    // write over the current contents.
    let mut wtr = Writer::from_writer(file);

    // Not actually sure what this does, kind of copy pasta to use reqwest.
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
    // This is initialized to the current time when the program starts running
    #[allow(non_snake_case)]
    let mut toTs: String = String::from(start_time);
    // my api key
    let api_key = &env::var("API_KEY").unwrap();

    // Main loop for the program. It will get the 2000 most recent data points,
    // then use the earliest time stamp among those to query for the next most
    // recent 2000 data points, going up to a total of 10000, the number of
    // minutes in a week (which is the current amount of historical data
    // available).
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
            wtr.serialize(line)?;
        }
        toTs = (data_response[0].time - 60).to_string();
    }

    Ok(())
}

pub async fn merge(file1: &str, file2: &str) -> Result<(), Box<dyn Error>> {
    Ok(())
}
