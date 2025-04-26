mod cmds;

use cmds::seed;
use sqlx::PgPool;
use std::io::stdin;
use logger::init::init_logger_stdout;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    init_logger_stdout("dev")?;

    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&db_url).await?;

    info!(%db_url, "DB Url Established and Pool created");
    
    loop {
        println!("> Welcome to [T]he [H]obby [P]roject CLI tool!\n
        > Please select an operation\n
        > [1] Seed Users\n
        > [2] Seed Businesses\n
        > [3] Seed Activities\n
        > [4] Seed Bookings\n
        > [5] Seed All\n
        > [6] Flush DB\n
        > [0] Exit\n");

        let mut selection = String::new();

        stdin().read_line(&mut selection).expect("Failed to read line");

        info!(%selection, "User selected an option");

        let selection: u16 = match selection.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                error!("Error parsing selection");
                continue;
            }
        };

        match selection {
            1 => {
                println!("How many users to seed?");
                let total = get_number_from_user();
                seed::seed_users(&pool, total).await?;
            }
            2 => {
                println!("How many businesses to seed?");
                let total = get_number_from_user();
                seed::seed_businesses(&pool, total).await?;
            }
            3 => {
                println!("How many activities to seed?");
                let total = get_number_from_user();
                seed::seed_activities(&pool, total).await?;
            }
            4 => {
                println!("How many bookings to seed?");
                let total = get_number_from_user();
                seed::seed_bookings(&pool, total).await?;
            }
            5 => {
                println!("How many of each to seed?");
                let total = get_number_from_user();
                seed::seed_all(&pool, total).await?;
            }
            6 => {
                println!("Flushing DB...");
                seed::flush_tables(&pool).await?;
            }
            0 => {
                println!("Exiting Program");
                std::process::exit(0);
            }
            _ => {
                println!("Invalid selection try again...");
            }
        };

    }

    //Ok(())

}

fn get_number_from_user() -> u32 {
    info!("Pulling number from user");
    let mut input = String::new();
    stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().parse().expect("Please enter a valid number")
}