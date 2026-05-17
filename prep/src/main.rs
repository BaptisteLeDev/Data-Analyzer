use anyhow::Result;

mod schema;
mod departements;
mod prenoms;
mod naissances;

const DB_PATH: &str = "../data/analyzer.sqlite";
const DBF_PATH: &str = "../data/nais2006.dbf";
const CSV_PATH: &str = "../data/dpt2006.csv";
const DEPTS_JSON: &str = "../web/src/data/departements.json";

fn main() -> Result<()> {
    println!("Data-Analyzer prep starting...");
    let mut conn = rusqlite::Connection::open(DB_PATH)?;
    schema::create(&conn)?;
    departements::load(&mut conn, DEPTS_JSON)?;
    prenoms::load(&mut conn, CSV_PATH)?;
    naissances::load(&mut conn, DBF_PATH)?;
    schema::index(&conn)?;
    println!("Done. Database written to {DB_PATH}");
    Ok(())
}
