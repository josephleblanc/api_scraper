use chrono::NaiveDateTime;
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

// Read two files downloaded api data saved as a .csv, and remove duplicates.
// Also checks to see if there are any gaps in the minute-resolution of the data.
pub async fn merge(file1: &str, file2: &str, write_dir: &str) -> Result<String, Box<dyn Error>> {
    let mut rdr1 = csv::Reader::from_path(file1)?;
    let mut rdr2 = csv::Reader::from_path(file2)?;
    let mut data_vec: Vec<Data> = vec![];
    for row in rdr1.deserialize() {
        let data1: Data = row?;
        data_vec.push(data1);
    }
    for row in rdr2.deserialize() {
        let data2: Data = row?;
        data_vec.push(data2);
    }

    data_vec.sort_by(|a, b| a.time.cmp(&b.time));

    println!("before dedup, vec.len(): {}", data_vec.len());
    let doubles_before: u64 = count_doubles(&mut data_vec)?;
    println!("{} doubles found.", doubles_before);

    data_vec.dedup_by(|a, b| a.time.eq(&b.time));

    println!("after  dedup, vec.len(): {}", data_vec.len());
    let doubles_after: u64 = count_doubles(&mut data_vec)?;
    println!("{} doubles found.", doubles_after);
    println!("first: {}", data_vec[0].time);
    println!("last : {}", data_vec.last().unwrap().time);
    println!("Checking continuity...");
    let continuity_breaks = check_continuity(&mut data_vec)?;
    match continuity_breaks.is_empty() {
        true => {
            println!("\tNo breaks in continuity");
        }
        false => {
            println!("\t{} breaks in continuity.", continuity_breaks.len());
            //for entry in continuity_breaks {
            //    println!("\t{:?}", entry);
            //}
        }
    }

    let start_date = NaiveDateTime::from_timestamp(data_vec[0].time.try_into().unwrap(), 0);
    let end_date =
        NaiveDateTime::from_timestamp(data_vec.last().unwrap().time.try_into().unwrap(), 0);
    let file_name = format!(
        "{}/{}_to_{}.csv",
        write_dir,
        start_date.date(),
        end_date.date()
    );
    println!("Writing to file: {}", &file_name);
    let fd = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&file_name)
        .expect("Could not create file, likely because the file already exists.");
    let mut wtr = Writer::from_writer(fd);
    for row in data_vec {
        wtr.serialize(row)?;
    }

    Ok(file_name)
}

// count the number of doubles in an array of Data structs ordered by time.
pub fn count_doubles(data_vec: &mut [Data]) -> Result<u64, Box<dyn Error>> {
    let mut doubles: u64 = 0;
    let peekable = &mut data_vec.iter().peekable();
    for _ in 0..peekable.len() {
        if let Some(first) = peekable.next() {
            if let Some(second) = peekable.peek() {
                if first.time == second.time {
                    doubles += 1;
                }
            }
        }
    }
    Ok(doubles)
}

pub fn check_continuity(data_vec: &mut [Data]) -> Result<Vec<&Data>, Box<dyn Error>> {
    let mut continuity_breaks: Vec<&Data> = vec![];
    let peekable = &mut data_vec.iter().peekable();
    for _ in 0..peekable.len() {
        if let Some(first) = peekable.next() {
            if let Some(second) = peekable.peek() {
                if first.time + 60 != second.time {
                    continuity_breaks.push(first);
                }
            }
        }
    }
    Ok(continuity_breaks)
}
