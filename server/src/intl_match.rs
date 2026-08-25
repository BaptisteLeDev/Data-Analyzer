//! Multi-algo matching: given a US-side anglo name, find French INSEE
//! candidates that are plausibly the same person under a different spelling.
//!
//! Three independent algos run in parallel and their results are merged
//! per French candidate name. A row that's flagged by multiple algos is
//! scored higher — see `Match::score()`.
//!
//! The three algos:
//!   1. `phonetic`     — DoubleMetaphone primary/alt code match
//!   2. `lev2`         — Levenshtein ≤ 2 (pre-filtered by length & first letter)
//!   3. `anglicisation` — rule-based spelling variants (PH↔F, K↔C, Y↔I, …)

use crate::phonetic::Phon;
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct Match {
    pub prenom: String,
    /// Sub-set of {"phonetic", "lev2", "anglicisation"}
    pub matched_by: Vec<&'static str>,
    /// Min Levenshtein distance observed (for lev2). None when not matched by lev2.
    pub lev_distance: Option<u32>,
    /// Original intl name(s) that produced this match (max 3 kept).
    pub from_intl: Vec<String>,
    /// Anglicisation rule labels that hit (e.g. "PH→F", "Y→I").
    pub anglo_rules: Vec<String>,
    /// Total INSEE occurrences (sum over years) — fetched post-merge.
    pub insee_total: i64,
    /// Number of distinct INSEE years the name appears in.
    pub insee_years: i64,
    /// Already filtered (one_l, n_min..n_max), so this is the final score.
    pub score: f64,
}

impl Match {
    /// Confidence score. Designed so:
    ///   - 3 algos hit  → ≥ 1.0
    ///   - 2 algos hit  → 0.6 .. 0.9
    ///   - 1 algo hit   → 0.3 .. 0.5
    /// Plus a small Lev-distance bonus (closer = higher).
    fn compute_score(&self) -> f64 {
        let base: f64 = match self.matched_by.len() {
            3 => 1.0,
            2 => 0.7,
            1 => 0.4,
            _ => 0.0,
        };
        let lev_bonus = match self.lev_distance {
            Some(0) => 0.30,
            Some(1) => 0.20,
            Some(2) => 0.10,
            _ => 0.0,
        };
        let multi_intl_bonus = (self.from_intl.len().min(3) as f64 - 1.0).max(0.0) * 0.05;
        (base + lev_bonus + multi_intl_bonus).min(1.5)
    }
}

#[derive(Debug, Clone)]
pub struct MatchParams {
    pub letter: String,
    pub sex: i32,
    pub search: String,
    pub exclude: String,
    pub era_start: i32,
    pub era_end: i32,
    /// Inclusive bounds for total INSEE occurrences.
    pub n_min: i64,
    pub n_max: i64,
    /// Require exactly one 'L' (case-insensitive) in the candidate.
    pub one_l: bool,
    /// Max Levenshtein distance for lev2 algo.
    pub lev_max: u32,
    /// Max number of intl seed names to expand on (controls cost).
    pub intl_seed_limit: usize,
    /// Cap on the merged result set returned to caller.
    pub limit: usize,
    /// Optional: which algos to run. Defaults to all three.
    pub use_phonetic: bool,
    pub use_lev2: bool,
    pub use_anglicisation: bool,
}

/// Run the three matchers, merge, score, filter, sort, paginate.
pub fn run(conn: &Connection, p: &MatchParams) -> anyhow::Result<Vec<Match>> {
    // ---------- 1. Pick intl seeds (US names absent from INSEE) ----------
    //
    // We fetch the candidate US-side names first, then for each we ask the
    // three algos for INSEE matches.
    let intl_seeds = fetch_intl_seeds(conn, p)?;

    // INSEE-known set (any year) — we EXCLUDE these from the final answer
    // (the whole point: French names that *don't* show up in INSEE for the
    // anglo spelling, but DO show up under a French variant).
    //
    // But wait — the user wants names that ARE in INSEE (the rare ones).
    // The mission is: anglo name → find its French INSEE variant. So:
    //   - intl seeds = names present in US, *absent* from INSEE entirely
    //     (these are anglo spellings the French dropped)
    //   - candidates = French INSEE names that map back to a seed
    let french_universe: HashSet<String> = {
        let mut s = conn.prepare(
            "SELECT DISTINCT prenom FROM prenoms_nat
              WHERE prenom NOT IN ('_PRENOMS_RARES', 'XXXX') AND length(prenom) > 1",
        )?;
        let out: HashSet<String> = s
            .query_map([], |r| r.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .collect();
        out
    };

    let mut matches: HashMap<String, Match> = HashMap::new();

    // ---------- 2. Algo: phonetic ----------
    if p.use_phonetic {
        run_phonetic(conn, &intl_seeds, &french_universe, &mut matches)?;
    }

    // ---------- 3. Algo: lev2 ----------
    if p.use_lev2 {
        run_lev2(conn, &intl_seeds, &french_universe, p.lev_max, &mut matches)?;
    }

    // ---------- 4. Algo: anglicisation rules ----------
    if p.use_anglicisation {
        run_anglicisation(&intl_seeds, &french_universe, &mut matches);
    }

    // ---------- 5. Post-filter (one_l, letter, sex via INSEE join, n_min..n_max) ----------
    if matches.is_empty() {
        return Ok(vec![]);
    }

    // Collect INSEE totals for all candidate names in one batch
    let names: Vec<String> = matches.keys().cloned().collect();
    let totals = fetch_insee_totals(conn, &names, p.sex)?;

    let mut out: Vec<Match> = Vec::new();
    let letter_upper = p.letter.to_uppercase();
    let search_upper = p.search.to_uppercase();
    let exclude_upper = p.exclude.to_uppercase();

    for (name, mut m) in matches {
        // hard filter: letter
        if !letter_upper.is_empty() && !name.contains(&letter_upper) {
            continue;
        }
        // hard filter: search substring
        if !search_upper.is_empty() && !name.contains(&search_upper) {
            continue;
        }
        // hard filter: exclude substring
        if !exclude_upper.is_empty() && name.contains(&exclude_upper) {
            continue;
        }
        // hard filter: exactly one 'L'
        if p.one_l {
            let l_count = name.chars().filter(|c| c.eq_ignore_ascii_case(&'L')).count();
            if l_count != 1 {
                continue;
            }
        }
        // hard filter: INSEE occurrence band (and presence)
        let (insee_total, insee_years) = match totals.get(&name) {
            Some(v) => *v,
            None => continue, // not in INSEE → drop
        };
        if insee_total < p.n_min || insee_total > p.n_max {
            continue;
        }

        m.insee_total = insee_total;
        m.insee_years = insee_years;
        m.prenom = name;
        // dedup + stable order for matched_by
        m.matched_by.sort();
        m.matched_by.dedup();
        m.from_intl.sort();
        m.from_intl.dedup();
        m.from_intl.truncate(3);
        m.anglo_rules.sort();
        m.anglo_rules.dedup();
        m.score = m.compute_score();
        out.push(m);
    }

    // sort: score desc, then total asc (prefer rarer), then name
    out.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.insee_total.cmp(&b.insee_total))
            .then(a.prenom.cmp(&b.prenom))
    });

    out.truncate(p.limit);
    Ok(out)
}

fn fetch_intl_seeds(conn: &Connection, p: &MatchParams) -> anyhow::Result<Vec<String>> {
    // US names absent from INSEE entirely, with at least one occurrence in
    // the era range. Bounded by `intl_seed_limit` (rarest first), and an
    // optional letter constraint inherited from the search.
    let letter_pattern = if p.letter.is_empty() {
        "%".to_string()
    } else {
        format!("%{}%", p.letter.to_uppercase())
    };
    let search_pattern = if p.search.is_empty() {
        "%".to_string()
    } else {
        format!("%{}%", p.search.to_uppercase())
    };

    let sql = "
        SELECT i.prenom
          FROM prenoms_intl i
         WHERE i.prenom LIKE ?1
           AND i.prenom LIKE ?2
           AND length(i.prenom) > 1
           AND NOT EXISTS (
                SELECT 1 FROM prenoms_nat n
                 WHERE n.prenom = i.prenom
           )
         GROUP BY i.prenom
        HAVING SUM(CASE WHEN i.annee BETWEEN ?3 AND ?4 THEN i.nombre ELSE 0 END) > 0
         ORDER BY SUM(i.nombre) ASC, i.prenom ASC
         LIMIT ?5
    ";
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt
        .query_map(
            rusqlite::params![letter_pattern, search_pattern, p.era_start, p.era_end, p.intl_seed_limit as i64],
            |r| r.get::<_, String>(0),
        )?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

fn run_phonetic(
    conn: &Connection,
    intl_seeds: &[String],
    french_universe: &HashSet<String>,
    out: &mut HashMap<String, Match>,
) -> anyhow::Result<()> {
    let phon = Phon::new();

    // pre-load nat_phon table into a code → [names] map. O(N) memory but
    // saves N seeds * lookup-cost. Names: ~50k. Cheap.
    let mut by_phon: HashMap<String, Vec<String>> = HashMap::new();
    {
        let mut s = conn.prepare("SELECT prenom, phon, phon_alt FROM prenoms_nat_phon")?;
        let it = s.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
        })?;
        for row in it.flatten() {
            let (name, p, a) = row;
            if !french_universe.contains(&name) {
                continue;
            }
            by_phon.entry(p.clone()).or_default().push(name.clone());
            if !a.is_empty() && a != p {
                by_phon.entry(a).or_default().push(name);
            }
        }
    }

    for seed in intl_seeds {
        let (p, a) = phon.encode(seed);
        let mut hits: Vec<String> = Vec::new();
        if let Some(v) = by_phon.get(&p) {
            hits.extend(v.iter().cloned());
        }
        if !a.is_empty() && a != p {
            if let Some(v) = by_phon.get(&a) {
                hits.extend(v.iter().cloned());
            }
        }
        hits.sort();
        hits.dedup();
        for name in hits {
            // never propose an exact-spelling collision (it would be INSEE-known
            // for the same string — but french_universe already excludes that
            // when the name only differs in case, etc. We DO want spelling
            // variants here even if they share a code.)
            let e = out.entry(name.clone()).or_insert_with(|| Match {
                prenom: name.clone(),
                matched_by: vec![],
                lev_distance: None,
                from_intl: vec![],
                anglo_rules: vec![],
                insee_total: 0,
                insee_years: 0,
                score: 0.0,
            });
            e.matched_by.push("phonetic");
            e.from_intl.push(seed.clone());
        }
    }
    Ok(())
}

fn run_lev2(
    conn: &Connection,
    intl_seeds: &[String],
    french_universe: &HashSet<String>,
    lev_max: u32,
    out: &mut HashMap<String, Match>,
) -> anyhow::Result<()> {
    // Group INSEE names by (first_char, length-bucket) for cheap pre-filter.
    let mut buckets: HashMap<(char, usize), Vec<String>> = HashMap::new();
    for name in french_universe {
        if name.is_empty() {
            continue;
        }
        let first = name.chars().next().unwrap().to_ascii_uppercase();
        buckets.entry((first, name.len())).or_default().push(name.clone());
    }
    let _ = conn; // unused here — kept in signature for symmetry / future EXPLAIN

    for seed in intl_seeds {
        if seed.is_empty() {
            continue;
        }
        let first = seed.chars().next().unwrap().to_ascii_uppercase();
        let slen = seed.len();
        let lo = slen.saturating_sub(lev_max as usize);
        let hi = slen + lev_max as usize;
        for blen in lo..=hi {
            if let Some(v) = buckets.get(&(first, blen)) {
                for cand in v {
                    let d = strsim::levenshtein(seed, cand) as u32;
                    if d <= lev_max && d > 0 {
                        let e = out.entry(cand.clone()).or_insert_with(|| Match {
                            prenom: cand.clone(),
                            matched_by: vec![],
                            lev_distance: None,
                            from_intl: vec![],
                            anglo_rules: vec![],
                            insee_total: 0,
                            insee_years: 0,
                            score: 0.0,
                        });
                        e.matched_by.push("lev2");
                        e.lev_distance = Some(match e.lev_distance {
                            Some(prev) => prev.min(d),
                            None => d,
                        });
                        e.from_intl.push(seed.clone());
                    }
                }
            }
        }
    }
    Ok(())
}

/// Bidirectional anglicisation/desanglicisation rules. Each rule produces
/// up to a few alternative spellings of `seed` to look up in the French set.
fn run_anglicisation(
    intl_seeds: &[String],
    french_universe: &HashSet<String>,
    out: &mut HashMap<String, Match>,
) {
    for seed in intl_seeds {
        let variants = anglicisation_variants(seed);
        for (variant, rules) in variants {
            if variant == *seed {
                continue;
            }
            if french_universe.contains(&variant) {
                let e = out.entry(variant.clone()).or_insert_with(|| Match {
                    prenom: variant.clone(),
                    matched_by: vec![],
                    lev_distance: None,
                    from_intl: vec![],
                    anglo_rules: vec![],
                    insee_total: 0,
                    insee_years: 0,
                    score: 0.0,
                });
                e.matched_by.push("anglicisation");
                e.from_intl.push(seed.clone());
                for r in rules {
                    e.anglo_rules.push(r);
                }
            }
        }
    }
}

/// Returns a list of (variant, rule_labels_that_produced_it). Rules are
/// applied in cross-product (up to a small cap to avoid combinatorial
/// explosion).
pub fn anglicisation_variants(input: &str) -> Vec<(String, Vec<String>)> {
    let upper = input.to_uppercase();
    let mut frontier: Vec<(String, Vec<String>)> = vec![(upper.clone(), vec![])];
    let mut seen: HashSet<String> = HashSet::new();
    seen.insert(upper);

    // We apply each rule family in a single pass over the current frontier.
    // Order doesn't really matter since we union everything in `seen`.
    let rule_families: &[fn(&str) -> Vec<(String, &'static str)>] = &[
        rule_ph_f,
        rule_k_c_start,
        rule_y_i,
        rule_ey_ie_end,
        rule_igh_y,
        rule_ou_u,
        rule_w_v,
        rule_double_consonant,
    ];

    let mut idx = 0;
    // up to 3 passes — enough for compound rules (e.g. KARLEY → CARLIE)
    for _pass in 0..3 {
        let snapshot = frontier[idx..].to_vec();
        idx = frontier.len();
        for (s, prior_rules) in snapshot {
            for rule_fn in rule_families {
                for (v, label) in rule_fn(&s) {
                    if seen.insert(v.clone()) {
                        let mut combined = prior_rules.clone();
                        combined.push(label.to_string());
                        frontier.push((v, combined));
                        if frontier.len() > 80 {
                            // hard cap — we never need more
                            return frontier;
                        }
                    }
                }
            }
        }
    }
    frontier
}

fn rule_ph_f(s: &str) -> Vec<(String, &'static str)> {
    let mut out = vec![];
    if s.contains("PH") {
        out.push((s.replace("PH", "F"), "PH→F"));
    }
    if s.contains('F') {
        out.push((s.replacen('F', "PH", 1), "F→PH"));
    }
    out
}
fn rule_k_c_start(s: &str) -> Vec<(String, &'static str)> {
    let mut out = vec![];
    if s.starts_with('K') {
        out.push((format!("C{}", &s[1..]), "K→C"));
    }
    if s.starts_with('C') {
        out.push((format!("K{}", &s[1..]), "C→K"));
    }
    out
}
fn rule_y_i(s: &str) -> Vec<(String, &'static str)> {
    let mut out = vec![];
    if s.contains('Y') {
        out.push((s.replace('Y', "I"), "Y→I"));
    }
    if s.contains('I') {
        out.push((s.replacen('I', "Y", 1), "I→Y"));
    }
    out
}
fn rule_ey_ie_end(s: &str) -> Vec<(String, &'static str)> {
    let mut out = vec![];
    if let Some(stem) = s.strip_suffix("EY") {
        out.push((format!("{stem}IE"), "-EY→-IE"));
        out.push((format!("{stem}I"), "-EY→-I"));
    }
    if let Some(stem) = s.strip_suffix("IE") {
        out.push((format!("{stem}EY"), "-IE→-EY"));
        out.push((format!("{stem}Y"), "-IE→-Y"));
    }
    if let Some(stem) = s.strip_suffix('Y') {
        out.push((format!("{stem}IE"), "-Y→-IE"));
    }
    out
}
fn rule_igh_y(s: &str) -> Vec<(String, &'static str)> {
    let mut out = vec![];
    if s.contains("IGH") {
        out.push((s.replace("IGH", "Y"), "IGH→Y"));
        out.push((s.replace("IGH", "I"), "IGH→I"));
    }
    out
}
fn rule_ou_u(s: &str) -> Vec<(String, &'static str)> {
    let mut out = vec![];
    if s.contains("OU") {
        out.push((s.replace("OU", "U"), "OU→U"));
    }
    if s.contains('U') && !s.contains("OU") {
        out.push((s.replacen('U', "OU", 1), "U→OU"));
    }
    out
}
fn rule_w_v(s: &str) -> Vec<(String, &'static str)> {
    let mut out = vec![];
    if s.starts_with('W') {
        out.push((format!("V{}", &s[1..]), "W→V"));
    }
    if s.starts_with('V') {
        out.push((format!("W{}", &s[1..]), "V→W"));
    }
    out
}
/// Collapse / expand a doubled consonant. Only does the first occurrence to
/// avoid blowing up the variant set.
fn rule_double_consonant(s: &str) -> Vec<(String, &'static str)> {
    let chars: Vec<char> = s.chars().collect();
    let mut out = vec![];
    // collapse first XX → X
    for k in 0..chars.len().saturating_sub(1) {
        if chars[k] == chars[k + 1]
            && chars[k].is_ascii_alphabetic()
            && !matches!(chars[k], 'A' | 'E' | 'I' | 'O' | 'U')
        {
            let mut v: String = chars[..k].iter().collect();
            v.extend(chars[k + 1..].iter());
            out.push((v, "XX→X"));
            break;
        }
    }
    // expand first consonant → XX
    for k in 0..chars.len() {
        if k > 0
            && chars[k].is_ascii_alphabetic()
            && !matches!(chars[k], 'A' | 'E' | 'I' | 'O' | 'U')
            && (k + 1 >= chars.len() || chars[k + 1] != chars[k])
        {
            let mut v: String = chars[..=k].iter().collect();
            v.push(chars[k]);
            v.extend(chars[k + 1..].iter());
            out.push((v, "X→XX"));
            break;
        }
    }
    out
}

/// Returns `(total_n_all_years, distinct_years_present)` per name, sex-filtered.
fn fetch_insee_totals(
    conn: &Connection,
    names: &[String],
    sex: i32,
) -> anyhow::Result<HashMap<String, (i64, i64)>> {
    let mut out: HashMap<String, (i64, i64)> = HashMap::with_capacity(names.len());
    // We could build a giant `WHERE prenom IN (...)`, but for ~few hundred
    // candidates the per-row prepared statement re-use is fast enough and
    // avoids hitting SQLite's parameter limits. Wrapped in a single
    // transaction for performance.
    let tx = conn.unchecked_transaction()?;
    let mut stmt = tx.prepare(
        "SELECT COALESCE(SUM(nombre), 0) AS total,
                COUNT(DISTINCT annee)     AS years
           FROM prenoms_nat
          WHERE prenom = ?1
            AND (?2 = 0 OR sexe = ?2)",
    )?;
    for n in names {
        let (total, years): (i64, i64) = stmt
            .query_row(rusqlite::params![n, sex], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
            })
            .unwrap_or((0, 0));
        if total > 0 {
            out.insert(n.clone(), (total, years));
        }
    }
    drop(stmt);
    tx.commit()?;
    Ok(out)
}
