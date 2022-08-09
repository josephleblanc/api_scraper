use api_scraper::{backup_master, merge, scrape};
use std::error::Error;
use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::time::SystemTime;

// This is meant to be a command line tool used to interact with the <forgot name>
// api. It can download one week's worth of market data at one minute resolution,
// turn it into a .csv, merge .csv files together and check for discontinuities in
// the resulting file.
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
        } else if command.starts_with("update master") {
            let master_dir = "/home/brasides/programming/data/BTC_historic_minute/master";
            let write_dir = master_dir;
            let mut master_file: Vec<String> = fs::read_dir(master_dir)?
                .filter(|f| f.as_ref().unwrap().metadata().unwrap().is_file())
                .map(|f| f.unwrap().file_name().into_string().unwrap())
                .collect();
            if master_file.len() != 1 {
                println!("There must be one file in the master folder. Ensure the only file in the master folder is the master .csv file to proceed.");
                continue;
            }
            let master_file_name = master_file.pop().expect("Problem loading master file");
            let master_file_path = format!("{}/{}", master_dir, master_file_name);

            println!("Creating backup for master file");
            let backup_filepath = backup_master(&master_file_name)?;
            println!("Master backup successfully created");
            println!("Merging backup with newly downloaded files");

            let newly_downloaded_dir =
                "/home/brasides/programming/data/BTC_historic_minute/weekly_data";
            let mut newly_downloaded_filenames: Vec<String> = fs::read_dir(newly_downloaded_dir)
                .expect("cannot read newly_downloaded_dir")
                .filter(|f| f.as_ref().unwrap().metadata().unwrap().is_file())
                .map(|f| f.unwrap().file_name().into_string().unwrap())
                .collect();
            if newly_downloaded_filenames.len() == 1 {
                let newly_downloaded_filepath = format!(
                    "{}/{}",
                    newly_downloaded_dir,
                    newly_downloaded_filenames.pop().unwrap()
                );
                println!("merging {} with master", newly_downloaded_filepath);
                let new_master =
                    merge(&backup_filepath, &newly_downloaded_filepath, write_dir).await?;
                println!("new master created at {}", new_master);
                println!("removing newly dowloaded file");
                fs::remove_file(&newly_downloaded_filepath)
                    .expect("Could not remove newly dowloaded file");
                println!("removing old master");
                fs::remove_file(&master_file_path).expect("Could not remove old master file");
                println!("master successfully updated");
            } else {
                println!("Error: There is more than one newly downloaded file in weekly_data");
                continue;
            }
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
                        continue;
                    }
                }
                println!("You must supply 2 file names to use merge");
                continue;
            } else if command.starts_with("merge master all") {
                let master_dir = "/home/brasides/programming/data/BTC_historic_minute/master";
                let write_dir = master_dir;

                let mut master_file: Vec<String> = fs::read_dir(master_dir)?
                    .filter(|f| f.as_ref().unwrap().metadata().unwrap().is_file())
                    .map(|f| f.unwrap().file_name().into_string().unwrap())
                    .collect();
                if master_file.len() != 1 {
                    println!("There must be one file in the master folder. Ensure the only file in the master folder is the master .csv file to proceed.");
                    continue;
                }
                let master_file_name = master_file.pop().expect("Problem loading master file");
                let master_file = format!("{}/{}", master_dir, master_file_name);
                let merged_dir =
                    "/home/brasides/programming/data/BTC_historic_minute/merged_storage";
                let merged_files: Vec<String> = fs::read_dir(merged_dir)?
                    .filter(|f| f.as_ref().unwrap().metadata().unwrap().is_file())
                    .map(|f| f.unwrap().file_name().into_string().unwrap())
                    .collect();
                let mut new_master: String = String::new();
                for file in merged_files {
                    let f = format!("{}/{}", merged_dir, file);
                    println!("merging {} with master", f);
                    new_master = merge(&master_file, &f, write_dir).await?;
                }
                println!("merging complete");
                let master_dir_files = fs::read_dir(master_dir)?
                    .filter(|f| f.as_ref().unwrap().metadata().unwrap().is_file())
                    .map(|f| f.unwrap().file_name().into_string().unwrap());
                if master_dir_files.count() != 1 {
                    println!("moving old master to backup");
                    let backup_dir = "/home/brasides/programming/data/BTC_historic_minute/backup";
                    let backup_master_filepath = format!("{}/{}", backup_dir, master_file_name);
                    let _backup_master_fd = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(&backup_master_filepath)
                        .expect("Could not create backup master file");
                    fs::copy(&new_master, &backup_master_filepath)?;
                    fs::remove_file(&master_file)?;
                }
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
