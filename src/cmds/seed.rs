use std::string;

use sqlx::PgPool;
use serde_json::{json, Value};
use tracing::{error, info};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use fake::faker::internet::en::SafeEmail;
use fake::faker::company::en::CompanyName;
use fake::faker::address::en::CityName;
use fake::Fake;
use argon2::{
    password_hash::{
        self, rand_core::OsRng, PasswordHash, PasswordHasher, SaltString
    },
    Argon2
};
struct User {
    id: Uuid,
    email: String,
    password_hash: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

struct Business {
    id: Uuid,
    name: String,
    description: String,
    owner_id: Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
}

struct Activity {
    id: Uuid,
    business_id: Uuid,
    title: String,
    location: String,
    capacity: u32,
    time: chrono::DateTime<chrono::Utc>,
    equipment: Value,
    created_at: chrono::DateTime<chrono::Utc>,
}

struct Booking {
    id: Uuid,
    user_id: Uuid,
    activity_id: Uuid,
    booked_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create_tables(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Create tables called");

    sqlx::query("CREATE TABLE IF NOT EXISTS users (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        email TEXT NOT NULL UNIQUE,
        password_hash TEXT NOT NULL,
        created_at TIMESTAMPTZ DEFAULT NOW()
    );
    ")
        .execute(pool)
        .await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS businesses (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        name TEXT NOT NULL,
        description TEXT,
        owner_id UUID REFERENCES users(id) ON DELETE CASCADE,
        created_at TIMESTAMPTZ DEFAULT NOW()
    );
    ")
            .execute(pool)
            .await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS activities (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        business_id UUID REFERENCES businesses(id) ON DELETE CASCADE,
        title TEXT NOT NULL,
        location TEXT NOT NULL,
        capacity INT NOT NULL,
        time TIMESTAMPTZ NOT NULL,
        equipment JSONB DEFAULT '[]',
        created_at TIMESTAMPTZ DEFAULT NOW()
    );    
    ")
            .execute(pool)
            .await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS bookings (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        user_id UUID REFERENCES users(id) ON DELETE CASCADE,
        activity_id UUID REFERENCES activities(id) ON DELETE CASCADE,
        booked_at TIMESTAMPTZ DEFAULT NOW()
    );  
    ")
            .execute(pool)
            .await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS logs (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        level TEXT NOT NULL,
        message TEXT NOT NULL,
        context JSONB DEFAULT '{}',
        created_at TIMESTAMPTZ DEFAULT NOW()
    );
    ")
            .execute(pool)
            .await?;    
    
    info!("Tables created/already exist!");

    Ok(())

}

pub async fn seed_users(pool: &PgPool, total_users: u32) -> Result<(), sqlx::Error> {

    info!("Seed users called");
    create_tables(pool).await?;

    for i in 0..total_users {
        info!("Creating user");

        let password = "thehobbyproject".as_bytes();
        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::default();
        
        let password_hash = match argon2.hash_password(password, &salt) {
            Ok(hash) => hash.to_string(),
            Err(e) => {
                error!("Error creating password hash");
                continue;
            }
        };
        
        let user = User {
            id: Uuid::new_v4(),
            email: format!("user{}-{}", i, SafeEmail().fake::<String>()),
            password_hash,
            created_at: chrono::Utc::now(),
        };

        info!("User created, updating user table");

        sqlx::query("INSERT INTO users (id, email, password_hash, created_at) VALUES ($1, $2, $3, $4)")
            .bind(user.id)
            .bind(user.email)
            .bind(user.password_hash)
            .bind(user.created_at)
            .execute(pool)
            .await?;

        info!("User completed, continuing...");

    }

    info!("Seeding Users completed");

    Ok(())
}

pub async fn seed_businesses(pool: &PgPool, total_businesses: u32) -> Result<(), sqlx::Error> {
    info!("Seed businesses called");
    create_tables(pool).await?;

    info!("Fetching User ID's to populate Seed's with owners");
    let user_ids: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM users LIMIT $1")
                                                    .bind(total_businesses as i64)
                                                    .fetch_all(pool)
                                                    .await?;

     if user_ids.len() < total_businesses as usize {
        error!("Not enough user id's to populate seed businesses");
        return Err(sqlx::Error::RowNotFound)
     }

    for i in 0..total_businesses {
        info!("Creating Business");

        let business: Business = Business { 
            id: Uuid::new_v4(), 
            name: format!("Business-{}-{}", i, CompanyName().fake::<String>()), 
            description: String::from("A rabble rousing business description"), 
            owner_id: user_ids[i as usize], 
            created_at: chrono::Utc::now(), 
        };
        info!("Business successfully created.. Inserting into DB");

        sqlx::query("INSERT INTO businesses (id, name, description, owner_id, created_at) VALUES ($1, $2, $3, $4, $5)")
            .bind(business.id)
            .bind(business.name)
            .bind(business.description)
            .bind(business.owner_id)
            .bind(business.created_at)
            .execute(pool)
            .await?;

        info!("Business inserted successfully, continuing...");

    }

    info!("Seeding businesses completed");

    Ok(())
}

pub async fn seed_activities(pool: &PgPool, total_activities: u32) -> Result<(), sqlx::Error> {
    info!("Seeding activities called");
    create_tables(pool).await?;

    info!("Pulling Business IDs");
    let business_ids: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM businesses LIMIT $1")
                                                    .bind(total_activities as i64)
                                                    .fetch_all(pool)
                                                    .await?;


    if business_ids.len() < total_activities as usize {
        error!("Not enough businesses/users to seed activities");
        return Err(sqlx::Error::RowNotFound)
    }

    info!("ID's pulled successfully");

    for i in 0..total_activities {
        info!("Creating activity");

        let activity: Activity = Activity { 
            id: Uuid::new_v4(), 
            business_id: business_ids[i as usize], 
            title: String::from("A fun all inclusive activity"), 
            location: CityName().fake::<String>(), 
            capacity: 10, 
            time: chrono::Utc::now() + chrono::Duration::days(rand::random::<u8>() as i64), 
            equipment: json!(["Water Bottle", "Mat", "Towel"]), 
            created_at: chrono::Utc::now(), 
        };

        info!("Activity created successfully, inserting into DB...");

        sqlx::query("INSERT INTO activities (id, business_id, title, location, capacity, time, equipment, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb, $8)")
            .bind(activity.id)
            .bind(activity.business_id)
            .bind(activity.title)
            .bind(activity.location)
            .bind(activity.capacity as i64)
            .bind(activity.time)
            .bind(activity.equipment.to_string())
            .bind(activity.created_at)
            .execute(pool)
            .await?;

        info!("Activity successfully inserted, continuing...");

    }

    info!("Seeding activities completed");

    Ok(())
}

pub async fn seed_bookings(pool: &PgPool, total_bookings: u32) -> Result<(), sqlx::Error> {
    info!("Seed bookings called");
    create_tables(pool).await?;

    info!("Pulling User IDs");
    let user_ids: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM users LIMIT $1")
                                                    .bind(total_bookings as i64)
                                                    .fetch_all(pool)
                                                    .await?;
    info!("Pulling Activity IDs");
    let activity_ids: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM activities LIMIT $1")
                                                    .bind(total_bookings as i64)
                                                    .fetch_all(pool)
                                                    .await?;
    
    if user_ids.len() < total_bookings as usize || activity_ids.len() < total_bookings as usize {
        error!("Not enough users / activities to seed bookings");
        return Err(sqlx::Error::RowNotFound)
    }

    info!("ID's pulled successfully");

    for i in 0..total_bookings {
        info!("Creating booking");

        let booking: Booking = Booking { 
            id: Uuid::new_v4(), 
            user_id: user_ids[i as usize], 
            activity_id: activity_ids[i as usize], 
            booked_at: chrono::Utc::now(),
        };

        info!("Booking successfully created, inserting into DB...");

        sqlx::query("INSERT INTO bookings (id, user_id, activity_id, booked_at) VALUES ($1, $2, $3, $4)")
            .bind(booking.id)
            .bind(booking.user_id)
            .bind(booking.activity_id)
            .bind(booking.booked_at)
            .execute(pool)
            .await?;

        info!("Booking successfully inserted, continuing...");

    }

    info!("Seeding Bookings Completed.");

    Ok(())
}

pub async fn seed_all(pool: &PgPool, total: u32) -> Result<(), sqlx::Error> {

    info!("Seed all helper function called");
    seed_users(pool, total).await?;
    seed_businesses(pool, total).await?;
    seed_activities(pool, total).await?;
    seed_bookings(pool, total).await?;
    info!("Completed all seeding");

    Ok(())
}

pub async fn flush_tables(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM users;")
                    .execute(pool)
                    .await?;

    sqlx::query("DELETE FROM businesses;")
                    .execute(pool)
                    .await?;

    sqlx::query("DELETE FROM activities;")
                    .execute(pool)
                    .await?;

    sqlx::query("DELETE FROM bookings;")
                    .execute(pool)
                    .await?;

    sqlx::query("DELETE FROM logs;")
                    .execute(pool)
                    .await?;
    Ok(())
}