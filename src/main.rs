mod cmds;

use cmds::seed;
use sqlx::PgPool;
use std::io::stdin;
use logger::init::init_logger;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    init_logger("dev")?;

    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&db_url).await?;

    info!(%db_url, "DB Url Established and Pool created");
    
    loop {
        println!("> Welcome to [T]he [H]obby [P]roject CLI tool!\n> Please select an operation\n");

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
            // select functions here
            _ => println!("Placeholder"),
        };

    }

    Ok(())

}
