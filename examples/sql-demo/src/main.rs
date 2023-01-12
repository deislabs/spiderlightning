use anyhow::Result;

use rnglib::{Language, RNG};
use sql::*;
wit_bindgen_rust::import!("../../wit/sql.wit");
wit_error_rs::impl_error!(sql::SqlError);

fn main() -> Result<()> {
    let sql = Sql::open("my-db")?;

    // create table if it doesn't exist
    sql.exec(&sql::Statement::prepare(
        "CREATE TABLE IF NOT EXISTS users (id SERIAL PRIMARY KEY, name TEXT NOT NULL)",
        &[],
    ))?;

    // add new user
    let rng = RNG::new(&Language::Elven).unwrap();
    let name = rng.generate_name();
    sql.exec(&sql::Statement::prepare(
        "INSERT INTO users (name) VALUES (?)",
        &[&name],
    ))?;

    // get all users
    let all_users = sql.query(&sql::Statement::prepare(
        "SELECT name FROM users",
        &[],
    ))?;
    dbg!(all_users);

    // get one user
    let one_user = sql.query(&sql::Statement::prepare("SELECT name FROM users WHERE id = ?", &["2"]))?;
    dbg!(one_user);

    // try sql injection
    assert!(sql
        .query(&sql::Statement::prepare("SELECT name FROM users WHERE id = ?", &["2 OR 1=1"]))
        .is_err());

    Ok(())
}
