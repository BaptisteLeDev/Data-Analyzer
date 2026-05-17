use anyhow::Result;

mod schema;
mod departements;
mod prenoms;
mod prenoms_nat;
mod naissances;
mod intl;
mod intl_uk;

const DB_PATH: &str = "../data/analyzer.sqlite";
const DBF_PATH: &str = "../data/NAIS2006.dbf";
const NAT_DBF_PATH: &str = "../data/nat2021.dbf";
const DATA_DIR: &str = "../data";
const DEPTS_JSON: &str = "../web/src/data/departements.json";
const SSA_DIR: &str = "../data/ssa_names";
const UK_DIR: &str = "../data/uk_names";

fn main() -> Result<()> {
    println!("Data-Analyzer prep starting...");
    let _ = std::fs::remove_file(DB_PATH);
    let mut conn = rusqlite::Connection::open(DB_PATH)?;
    schema::create(&conn)?;
    departements::load(&mut conn, DEPTS_JSON)?;
    prenoms::load(&mut conn, DATA_DIR)?;
    prenoms_nat::load(&mut conn, NAT_DBF_PATH)?;
    naissances::load(&mut conn, DBF_PATH)?;
    intl::load(&mut conn, SSA_DIR)?;
    intl_uk::load(&mut conn, UK_DIR)?;
    schema::index(&conn)?;
    println!("Done. Database written to {DB_PATH}");
    Ok(())
}
