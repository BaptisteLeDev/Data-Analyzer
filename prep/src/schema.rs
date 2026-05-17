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
            mois      INTEGER NOT NULL,
            dept_dom  TEXT,
            dept_nais TEXT,
            sexe      INTEGER NOT NULL,
            count     INTEGER NOT NULL DEFAULT 1
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
        ",
    )?;
    Ok(())
}
