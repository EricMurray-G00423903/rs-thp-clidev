use std::string;

use sqlx::PgPool;
use tracing::{error, info};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use fake::faker::internet::en::SafeEmail;
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

pub async fn create_tables(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Creating tables for The Hobby Project");

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

pub async fn seed_businesses(pool: &PgPool) -> Result<(), sqlx::Error> {

    Ok(())
}

pub async fn seed_activities(pool: &PgPool) -> Result<(), sqlx::Error> {

    Ok(())
}

pub async fn seed_bookings(pool: &PgPool) -> Result<(), sqlx::Error> {

    Ok(())
}

pub async fn flush_tables(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM users;
                    DELETE FROM businesses;
                    DELETE FROM activities;
                    DELETE FROM bookings;")
                    .execute(pool)
                    .await?;
    Ok(())
}