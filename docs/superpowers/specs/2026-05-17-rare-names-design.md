# Design — Explorateur de prénoms rares (INSEE 2006)

**Date :** 2026-05-17
**Auteur :** Baptiste Dechamp
**Statut :** Brouillon — en attente de validation finale utilisateur
**Stack front retenue :** Svelte 5 + Vite + Bun (option B)

---

## 1. Objectif

Page web permettant à un utilisateur de trouver les **20 prénoms les plus rares** de France parmi les naissances de **2006** (année modifiable, plage 2000-2021), filtrés par :

- **Lettre obligatoire** dans le prénom — une seule lettre, recherche insensible à la casse, sous-chaîne (ex : `L` ⇒ tout prénom contenant `L` ou `l`). Default : `L`.
- **Département** de naissance (default : `76 — Seine-Maritime`).
- **Mois** de naissance (default : Mai = `05`) — *filtre démographique de contexte, voir §3*.
- **Année** (default : `2006`).

Look & feel inspiré des sites officiels de la République Française (INSEE, service-public.fr). Performances : algo en Rust, front léger.

## 2. Contrainte critique — disponibilité des données

Le fichier `etatcivil2006_nais2006_dbase.zip` fourni par l'utilisateur est le **fichier détail naissances** INSEE. D'après son dictionnaire de variables (`Contenu_du_fichier_naissances.pdf`), il contient :

```
SEXE, MNAIS, ANAIS, DEPDOM, DEPNAIS, AGEMERE, AGEPERE,
ACCOUCHR, AMAR, NBENFPRE, SITUATMR, SITUATPR, TUCOM, TUDOM, ...
```

**Aucune variable `PRENOM` ni `NOM`.** Ce fichier est anonymisé et purement démographique.

Les prénoms sont publiés par l'INSEE dans un **dataset séparé** : *Fichier des prénoms par département* (page : `insee.fr/fr/statistiques/2540004`). L'utilisateur dispose du fichier **allégé 2000-2021** (`dpt_2000_2021_csv.zip`) qui contient ~3,8 M lignes.

**Variables CSV** (séparateur `;`, en-tête présent) :

| Nom        | Type        | Long. | Modalités                                              |
| ---------- | ----------- | ----- | ------------------------------------------------------ |
| `SEXE`     | caractère   | 1     | `1` masculin, `2` féminin                              |
| `PREUSUEL` | caractère   | 25    | premier prénom en MAJUSCULES, sans accents             |
| `ANNAIS`   | caractère   | 4     | `1900`–`2021`, `XXXX` (année inconnue)                 |
| `DPT`      | caractère   | **3** | code département **3 caractères** (ex : `076`, `02A`, `02B`, `XX`) |
| `NOMBRE`   | numérique   | 8     | fréquence (≥ 3 ; sinon agrégé sous `_PRENOMS_RARES`)   |

**Granularité = année (pas de mois).** Censure INSEE : prénoms < 3 occurrences agrégés sous `_PRENOMS_RARES`.

⚠️ Le code département du CSV prénoms est sur **3 caractères** (ex : `076`) alors que le `.dbf` naissances utilise **2 caractères** (ex : `76`). Normalisation nécessaire à l'ETL : on stocke en table SQLite sur 2 caractères, padding à droite supprimé (`076` → `76`, `02A` → `2A`).

### Conséquence sur les filtres

| Filtre demandé    | Faisable                                                      |
| ----------------- | ------------------------------------------------------------- |
| Année 2006        | ✅                                                            |
| Département       | ✅ via `dpt` du fichier prénoms                               |
| Lettre dans nom   | ✅ via `LIKE '%L%'` SQL                                       |
| Top N rare        | ✅ tri ascendant par `nombre`                                 |
| **Mois**          | ⚠️ Pas dans le fichier prénoms. Affiché comme **contexte**.  |

Le sélecteur **Mois** est conservé dans l'UI mais étiqueté « filtre démographique » : il pilote un encart de contexte sous les résultats (« en mai 2006 en Seine-Maritime, *N* naissances sur *Total* annuel, soit *X*% »), calculé sur le `.dbf` naissances. Il ne ré-ordonne pas le classement des prénoms rares.

## 3. Architecture

```
┌──────────────────────────────────────────────────────────────┐
│  BROWSER                                                     │
│  Svelte + Vite (Bun runtime)                                 │
│  ├─ Header (logo tricolore + titre)                          │
│  ├─ Filters (année, dept, mois, lettre)                      │
│  ├─ Results (top 20 prénoms rares)                           │
│  └─ Context (distribution mensuelle du dept)                 │
└─────────────────────┬────────────────────────────────────────┘
                      │ HTTP JSON
┌─────────────────────▼────────────────────────────────────────┐
│  SERVER (Rust, axum)                                         │
│  GET  /api/departements        → liste officielle Etalab     │
│  GET  /api/rarest?dept=…&letter=…&limit=20                   │
│  GET  /api/birth-context?dept=…&month=…                      │
└─────────────────────┬────────────────────────────────────────┘
                      │
┌─────────────────────▼────────────────────────────────────────┐
│  SQLite (analyzer.sqlite)                                    │
│  Tables :                                                    │
│   prenoms  (sexe, prenom, annee, dept, nombre)               │
│   naissances (mois, dept_dom, dept_nais, sexe, … )           │
│   departements (code, nom)                                   │
└─────────────────────▲────────────────────────────────────────┘
                      │ one-shot
┌─────────────────────┴────────────────────────────────────────┐
│  PREP (Rust binaire `prep`)                                  │
│  - Lit nais2006.dbf via crate `dbase`                        │
│  - Lit dpt2006.csv (fichier prénoms INSEE)                   │
│  - Lit departements.json (Etalab)                            │
│  - Écrit SQLite avec index sur (annee, dept), (prenom)       │
└──────────────────────────────────────────────────────────────┘
```

### Composants

| Composant         | Rôle                                | Tech                       |
| ----------------- | ----------------------------------- | -------------------------- |
| `prep/`           | ETL one-shot dbf+csv → SQLite       | Rust, `dbase`, `rusqlite`, `csv` |
| `server/`         | API HTTP                            | Rust, `axum`, `rusqlite`, `serde` |
| `web/`            | UI                                  | Svelte 5, Vite, Bun        |
| `data/`           | Fichiers sources et SQLite généré   | gitignored                 |

### Pourquoi ce stack

- **Rust pour prep + server** : performance demandée par l'utilisateur, parsing dbf natif, SQLite zéro-config. Le binaire `server` démarre en <50ms, requêtes <5ms grâce aux index.
- **Svelte + Vite + Bun** : pas de virtual DOM (overkill pour 1 écran), build instantané, bundle <30 KB. Bun comme runtime/installer (plus rapide que npm sous Windows).
- **SQLite, pas Postgres** : monolithe local, zéro dépendance externe, parfait pour un dataset figé.

## 4. Algorithme d'élimination

```rust
fn top_rarest(conn, year, dept, letter, limit) -> Vec<NameResult> {
    let pattern = format!("%{}%", letter.to_uppercase());
    conn.prepare("
        SELECT prenom, sexe, SUM(nombre) AS n
        FROM prenoms
        WHERE annee = :year
          AND dept = :dept
          AND UPPER(prenom) LIKE :pattern
          AND prenom NOT IN ('_PRENOMS_RARES', 'XXXX')
          AND length(prenom) > 1
        GROUP BY prenom, sexe
        ORDER BY n ASC, prenom ASC
        LIMIT :limit
    ").query_map(...)
}
```

**Étapes d'élimination** :
1. Filtre année + département.
2. Lettre obligatoire (`LIKE '%X%'` insensible à la casse).
3. Exclusion `_PRENOMS_RARES` (agrégat de censure INSEE), `XXXX` (placeholder année inconnue ne devrait pas apparaître ici mais garde-fou), longueur ≤ 1.
4. Agrégation `(prenom, sexe)` : un même prénom peut apparaître séparément M/F.
5. Tri ascendant par occurrences puis alphabétique (tie-break déterministe).
6. Limite à N (20 par défaut).

**Ne pas inclure** : prénoms multiples agrégés, formes hypocoristiques (le fichier INSEE est déjà nettoyé en MAJUSCULES sans accents).

## 5. Endpoints API

### `GET /api/departements`

Renvoie la liste officielle issue de **Etalab/data.gouv.fr** (`departements-france.json`). Pré-chargé dans la table `departements` au build.

```json
[
  { "code": "01", "nom": "Ain" },
  { "code": "02", "nom": "Aisne" },
  ...
  { "code": "76", "nom": "Seine-Maritime" },
  ...
]
```

### `GET /api/rarest?year=2006&dept=76&letter=L&limit=20`

```json
{
  "dept": "76",
  "letter": "L",
  "results": [
    { "rank": 1, "prenom": "ALPHILDE",  "sexe": "F", "n": 3 },
    { "rank": 2, "prenom": "LOUVELYNE", "sexe": "F", "n": 3 },
    ...
  ]
}
```

### `GET /api/birth-context?year=2006&dept=76&month=05`

```json
{
  "dept": "76",
  "month": "05",
  "month_births": 1247,
  "year_births":  14802,
  "share_pct":    8.4
}
```

## 6. UI / UX

**Structure** : single-page, deux panneaux côte à côte.

```
┌──────────────────────────────────────────────────────────────┐
│  [▌] DATA-ANALYZER     prénoms rares · état civil INSEE 2006 │
├──────────────────────┬───────────────────────────────────────┤
│  FILTRES             │  TOP 20 — PRÉNOMS RARES (2006)        │
│                      │                                       │
│  Année      [2006 ▾] │  ┌──┬──────────────┬───┬───────────┐ │
│  Département[76  ▾]  │  │ 1│ ALPHILDE     │ F │  3 naiss. │ │
│  Mois       [Mai ▾]  │  │ 2│ LOUVELYNE    │ F │  3 naiss. │ │
│  Lettre obl.[ L  ]   │  │ 3│ ELYANEL      │ M │  3 naiss. │ │
│                      │  │ …│              │   │           │ │
│  [ ▶ Lancer ]        │  └──┴──────────────┴───┴───────────┘ │
│                      │                                       │
│                      │  ─── Contexte démographique ─────     │
│                      │  Mai 2006 / Seine-Maritime            │
│                      │  1 247 naissances (8,4% de l'année)   │
│                      │                                       │
└──────────────────────┴───────────────────────────────────────┘
```

**Style** :

| Élément       | Valeur                                                  |
| ------------- | ------------------------------------------------------- |
| Bleu primaire | `#000091` (bleu République)                             |
| Rouge accent  | `#E1000F`                                               |
| Fond          | `#FFFFFF`                                               |
| Fond panneau  | `#F6F6F6`                                               |
| Texte         | `#161616`                                               |
| Police        | Marianne (locale, fallback `Inter`, `system-ui`)        |
| Bordures      | 1px solid `#DDD`, rayons 0 (sobre, sans coins arrondis) |
| Spacing       | Base 8px                                                |

**Logo / favicon** : SVG monogramme `DA` sur bloc bleu, liseré rouge à droite. Style sobre, évoque la cocarde tricolore sans en abuser. Fichier `web/public/favicon.svg` + `favicon.ico` rasterisé.

**Accessibilité** : labels associés aux inputs, navigation clavier, contrastes AA min.

## 7. Données externes

| Source                        | Usage                       | Lien                                                       |
| ----------------------------- | --------------------------- | ---------------------------------------------------------- |
| Fichier prénoms INSEE allégé  | Algo rareté                 | `insee.fr/fr/statistiques/2540004` → `dpt_2000_2021_csv.zip` (fourni par l'utilisateur) |
| Fichier détail naissances 2006| Contexte mensuel            | `etatcivil2006_nais2006_dbase.zip` (fourni par l'utilisateur) |
| Départements officiels        | Dropdown                    | `github.com/etalab/decoupage-administratif`                |

Les deux fichiers INSEE sont fournis localement par l'utilisateur, à déposer dans `data/` avant lancement de `prep`. Non livrés dans le repo (taille, gitignored).

## 8. Découpage en travail

1. **Bootstrap** : init dossiers, `package.json`, `Cargo.toml` pour `prep/` et `server/`.
2. **Prep** : binaire Rust qui produit `analyzer.sqlite` depuis `nais2006.dbf` + `dpt2006.csv` + `departements.json`.
3. **Server** : axum avec les 3 endpoints, CORS dev, log structuré.
4. **Front** : Svelte avec les 4 composants (`Header`, `Filters`, `Results`, `Context`), fetch des endpoints.
5. **Logo** : SVG + favicon.
6. **README** : instructions « comment lancer ».

Chaque étape = un commit. Pas de tests automatisés (scope perso, dataset figé) ; vérification visuelle uniquement.

## 9. Hors scope

- Authentification, multi-utilisateur.
- Recherche fuzzy / suggestions de prénoms.
- Comparaison entre années / entre départements.
- Données outre-mer (codes 97+) — incluses si présentes dans le CSV mais pas mises en avant.
- Visualisations avancées (cartes choroplèthes, graphiques).
- Internationalisation (FR uniquement).

## 10. Risques

| Risque                                          | Mitigation                                   |
| ----------------------------------------------- | -------------------------------------------- |
| Encodage CSV INSEE (Latin-1 historiquement)     | Détection BOM, fallback Latin-1 → UTF-8       |
| Lignes `XXXX`/`XX` (année ou dept inconnus)     | Filtrées à l'ETL                              |
| Code dept 3 chars CSV vs 2 chars DBF            | Normalisation à 2 chars dans SQLite           |
| `.dbf` encodage Latin-1                         | `dbase` crate gère, on force decode latin-1  |
| Frustration utilisateur sur le filtre mois      | Tooltip explicite + texte clair sous l'input |
| Bun non installé sur Windows                    | Fallback npm/pnpm documenté                  |
