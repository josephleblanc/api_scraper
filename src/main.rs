use api_scraper::{merge, scrape};
use std::error::Error;
use std::fs;
use std::io;
use std::time::SystemTime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut buffer = String::new();
    let stdin = io::stdin();

    loop {
        buffer.clear();
        stdin.read_line(&mut buffer)?;
        println!("buffer: {}", buffer);

        let command = buffer.trim().to_lowercase();

        if command == "scrape" {
            let datadir = "/home/brasides/programming/data/BTC_historic_minute/weekly_data";
            let file_prefix = "BTC_minute";
            let start_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs()
                .to_string();
            let file_name = format!("{}_{}", file_prefix, start_time);

            println!("Scraping from timestamp: {}", start_time);
            println!("Output directory: {}", datadir);
            println!("Output file name: {}", file_name);
            scrape(datadir, &file_name, &start_time).await?;
            break;
        } else if command.starts_with("merge") {
            let datadir = "/home/brasides/programming/data/BTC_historic_minute/weekly_data";
            let write_dir = "/home/brasides/programming/data/BTC_historic_minute/merged_storage";
            let words: Vec<&str> = buffer.split_ascii_whitespace().collect();
            if words.len() != 3 {
                if command == "merge" {
                    let datadir_files: Vec<String> = fs::read_dir(datadir)?
                        .filter(|f| f.as_ref().unwrap().metadata().unwrap().is_file())
                        .map(|f| f.unwrap().file_name().into_string().unwrap())
                        .collect();
                    if datadir_files.len() == 2 {
                        let file1 = format!("{}/{}", datadir, datadir_files[0]);
                        let file2 = format!("{}/{}", datadir, datadir_files[1]);
                        merge(&file1, &file2, write_dir).await?;
                        println!(
                            "merge complete:\nfiles {} and {} have been successfully merged.",
                            file1, file2
                        );
                    }
                }
                println!("You must supply 2 file names to use merge");
                continue;
            }
            let file1 = format!("{}/{}", datadir, words[1]);
            let file2 = format!("{}/{}", datadir, words[2]);
            merge(&file1, &file2, write_dir).await?;
            println!(
                "merge complete:\nfiles {} and {} have been successfully merged.",
                file1, file2
            );
        } else if command == "help" {
            println!(
                "Available commands:
            - scrape: uses an api to get the past week's trade data for BTC-dollar prices at
              a resolution of 1 minute.
            - exit: exit program."
            );
        } else if command == "exit" {
            break;
        } else {
            println!(
                "You have not input a valid command.
        Type 'help' to see valid commands, or type 'exit' to exit program"
            );
        }
    }
    Ok(())
}
