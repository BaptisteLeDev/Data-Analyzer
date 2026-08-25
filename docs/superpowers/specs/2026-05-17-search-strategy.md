# Stratégie de recherche : SQLite suffit (et quand bouger)

`/api/intl-match` mélange trois algos (phonétique DoubleMetaphone matérialisé,
Levenshtein ≤ 2, règles d'anglicisation) sur ~50k INSEE × ~100k US. Le tout
tient dans un binaire + un fichier SQLite.

## Pourquoi FTS5 SQLite suffit

- **Volume** : ~5 M lignes total (prenoms 4.7 M, intl 1.9 M, naissances 800k).
  FTS5 supporte 10–50 M tokens sans s'essouffler.
- **Mono-utilisateur** : pas de QPS, pas de SLA. Une requête à 200 ms n'a pas
  d'impact business.
- **Single binary** : FTS5 est compilé dans `libsqlite3` (déjà bundlé via
  `rusqlite/bundled`). Zéro infra additionnelle.
- **Besoins actuels** : substring + filtres exacts couverts par index B-tree.
  Le fuzzy futur est déjà géré par `strsim` côté Rust.

Si on ajoute une recherche tolérante globale : `CREATE VIRTUAL TABLE
prenoms_fts USING fts5(prenom, tokenize='unicode61 remove_diacritics 2')` +
trigger de sync. Sub-jour.

## Quand basculer

- **Tantivy** (Rust natif, in-process, vrai concurrent de FTS5) — si on veut
  BM25 paramétrable, fuzzy intégré (sans implémentation Rust ad-hoc), ou
  facets. Migration sans casser le « single binary ». Seuil : recherche
  multi-algo > 500 ms en p95.
- **Meilisearch** — daemon séparé (interdit ici), mais excellent pour du
  multi-tenant grand public > 100 QPS.
- **ElasticSearch** — overkill franc (JVM, cluster, ops). Hors sujet.

## Recommandation

Rester sur **SQLite + index matérialisés + algos Rust**. Si la liste s'étoffe
(synonymes, scoring custom), passer sur **Tantivy embarqué**. Ne pas envisager
Meilisearch ou ES tant que le produit reste mono-utilisateur.
