use std::env;

fn main() {
    dotenv::dotenv().ok();
    // println!("API_KEY is {}", &env::var("API_KEY").unwrap());

    let seed = "https://min-api.cryptocompare.com/data/v2/histominute";

    let fsym = "BTC";
    let tsym = "USD";
    let aggregate = "1";
    let limit = "100";
    let api_key = &env::var("API_KEY").unwrap();

    let request_url = format!(
        "{seed}?fsym={fsym}&tsym={tsym}&aggregate={aggregate}&limit={limit}&api_key={api_key}",
        seed = seed,
        fsym = fsym,
        tsym = tsym,
        aggregate = aggregate,
        limit = limit,
        api_key = api_key
    );
    println!("request_url = {}", request_url);
}
