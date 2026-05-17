use anyhow::Result;

mod schema;
mod departements;
mod prenoms;
mod naissances;

const DB_PATH: &str = "../data/analyzer.sqlite";
const DBF_PATH: &str = "../data/NAIS2006.dbf";
const DATA_DIR: &str = "../data";
const DEPTS_JSON: &str = "../web/src/data/departements.json";

fn main() -> Result<()> {
    println!("Data-Analyzer prep starting...");
    let _ = std::fs::remove_file(DB_PATH);
    let mut conn = rusqlite::Connection::open(DB_PATH)?;
    schema::create(&conn)?;
    departements::load(&mut conn, DEPTS_JSON)?;
    prenoms::load(&mut conn, DATA_DIR)?;
    naissances::load(&mut conn, DBF_PATH)?;
    schema::index(&conn)?;
    println!("Done. Database written to {DB_PATH}");
    Ok(())
}
