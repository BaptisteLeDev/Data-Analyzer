use anyhow::Result;
use rusqlite::Connection;

pub fn create(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        DROP TABLE IF EXISTS prenoms;
        DROP TABLE IF EXISTS prenoms_nat;
        DROP TABLE IF EXISTS naissances;
        DROP TABLE IF EXISTS departements;
        DROP TABLE IF EXISTS prenoms_intl;
        DROP TABLE IF EXISTS prenoms_nat_phon;
        DROP TABLE IF EXISTS prenoms_intl_phon;

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

        CREATE TABLE prenoms_nat (
            sexe   INTEGER NOT NULL,
            prenom TEXT NOT NULL,
            annee  INTEGER NOT NULL,
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

        CREATE TABLE prenoms_intl (
            prenom TEXT NOT NULL,
            sex    INTEGER NOT NULL,  -- 1=M, 2=F
            annee  INTEGER NOT NULL,
            nombre INTEGER NOT NULL,
            source TEXT NOT NULL DEFAULT 'US'
        );

        -- Materialized phonetic codes (DoubleMetaphone, max len 6).
        -- Populated lazily by `server::phonetic::ensure_materialized` on
        -- first boot, OR by the migration script
        -- `prep/migrations/001_phonetic.sql` for existing DBs.
        CREATE TABLE prenoms_nat_phon (
            prenom   TEXT PRIMARY KEY,
            phon     TEXT NOT NULL,
            phon_alt TEXT NOT NULL
        );
        CREATE TABLE prenoms_intl_phon (
            prenom   TEXT PRIMARY KEY,
            phon     TEXT NOT NULL,
            phon_alt TEXT NOT NULL
        );
        ",
    )?;
    Ok(())
}

pub fn index(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE INDEX idx_prenoms_year_dept    ON prenoms(annee, dept);
        CREATE INDEX idx_prenoms_prenom       ON prenoms(prenom);
        CREATE INDEX idx_prenoms_nat_year     ON prenoms_nat(annee);
        CREATE INDEX idx_prenoms_nat_prenom   ON prenoms_nat(prenom);
        CREATE INDEX idx_nais_dept_mois       ON naissances(dept_nais, mois);
        CREATE INDEX idx_nais_age_mere        ON naissances(age_mere);
        CREATE INDEX idx_nais_age_pere        ON naissances(age_pere);
        CREATE INDEX idx_intl_prenom          ON prenoms_intl(prenom);
        CREATE INDEX idx_intl_annee           ON prenoms_intl(annee);
        CREATE INDEX idx_intl_source          ON prenoms_intl(source);
        CREATE INDEX idx_nat_phon_phon        ON prenoms_nat_phon(phon);
        CREATE INDEX idx_nat_phon_phon_alt    ON prenoms_nat_phon(phon_alt);
        CREATE INDEX idx_intl_phon_phon       ON prenoms_intl_phon(phon);
        CREATE INDEX idx_intl_phon_phon_alt   ON prenoms_intl_phon(phon_alt);
        ",
    )?;
    Ok(())
}
