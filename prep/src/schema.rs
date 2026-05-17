use anyhow::Result;
use rusqlite::Connection;

pub fn create(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        DROP TABLE IF EXISTS prenoms;
        DROP TABLE IF EXISTS naissances;
        DROP TABLE IF EXISTS departements;

        CREATE TABLE departements (
            code TEXT PRIMARY KEY,
            nom  TEXT NOT NULL
        );

        CREATE TABLE prenoms (
            sexe   INTEGER NOT NULL,
            prenom TEXT NOT NULL,
            annee  INTEGER NOT NULL,
            dept   TEXT NOT NULL,
            nombre INTEGER NOT NULL
        );

        CREATE TABLE naissances (
            id        INTEGER PRIMARY KEY,
            mois      INTEGER NOT NULL,
            dept_dom  TEXT,
            dept_nais TEXT,
            sexe      INTEGER NOT NULL,
            age_mere  INTEGER,
            age_pere  INTEGER,
            situ_mere TEXT,
            situ_pere TEXT,
            nat_mere  INTEGER,
            nat_pere  INTEGER,
            ln_mere   TEXT,
            ln_pere   TEXT,
            accouchr  TEXT,
            nbenfpre  INTEGER,
            dmarnais  TEXT,
            tudom     TEXT
        );
        ",
    )?;
    Ok(())
}

pub fn index(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE INDEX idx_prenoms_year_dept ON prenoms(annee, dept);
        CREATE INDEX idx_prenoms_prenom    ON prenoms(prenom);
        CREATE INDEX idx_nais_dept_mois    ON naissances(dept_nais, mois);
        CREATE INDEX idx_nais_age_mere     ON naissances(age_mere);
        CREATE INDEX idx_nais_age_pere     ON naissances(age_pere);
        ",
    )?;
    Ok(())
}
