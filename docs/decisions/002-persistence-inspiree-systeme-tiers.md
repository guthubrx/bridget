# Décision Architecturale : Observabilité des événements inspirée de un système tiers

## Contexte

Suite à l'analyse du système un système tiers, nous avons identifié des patterns d'observabilité (events JSONL) qui pourraient améliorer le debugging et la traçabilité de Bridget.

## Patterns observés chez un système tiers

### 1. Events JSONL pour traçabilité

```python
.runs/
└── [run_id]/
    ├── run.json           # État du run
    ├── events.jsonl       # Journal des événements
    └── result.json         # Résultat final
```

Chaque événement (création, réponse, annulation, escalation) est ajouté à `events.jsonl` pour une traçabilité complète post-mortem.

### 2. Pourquoi un système tiers utilise le filesystem

- un système tiers n'a PAS de database persistante
- Le fichier JSONL est SA source de vérité pour l'historique
- L'écriture atomique compensait l'absence de transactions

## Architecture actuelle de Bridget

Bridget utilise **SQLite** comme source de vérité :

```
Daemon + SQLite (Session 003 implémenté)
├── messages (historique routage)
├── tracked_requests (demandes suivies)
└── agents (présence)
```

**SQLite fournit déjà :**
- ✅ Écritures atomiques (transactions ACID + fsync)
- ✅ Persistence après redémarrage
- ✅ Pas de corruption possible
- ✅ Relecture de l'état complet

## Analyse critique

**L'écriture JSON atomique est REDONDANTE avec SQLite :**
- SQLite garantit déjà l'atomicité et la durabilité
- Dupliquer l'état (JSON + SQLite) = complexité inutile
- Synchroniser les deux sources = bugs potentiels

**Ce qui est réellement utile :**
- ➕ Observabilité (events) pour debugging
- 📝 Traçabilité des transitions d'état
- 🔍 Recherche post-mortem

## Proposition adaptée à l'architecture Bridget

### Option : Table events dans SQLite

```sql
CREATE TABLE request_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    request_id TEXT NOT NULL,
    event_type TEXT NOT NULL,  -- created, answered, cancelled, timed_out, escalated
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    details TEXT,               -- JSON optionnel
    FOREIGN KEY (request_id) REFERENCES tracked_requests(id)
);
```

**Avantages :**
- ✅ Une seule source de vérité (SQLite)
- ✅ Transactions garantissent la cohérence
- ✅ Requêtes SQL pour analyse
- ✅ Export JSONL possible si besoin
- ✅ Architecture simple, cohérente

### Option alternative : Export JSONL à la demande

```rust
fn export_request_events(conn: &Connection, request_id: &str) -> Result<Vec<Event>> {
    // Lecture depuis SQLite
    // Génération JSONL pour tools externes
}
```

## Décision

**REJETER** la duplication JSON + SQLite (incohérent architecturalement).

**ACCEPTER** l'observabilité via une table `request_events` dans SQLite, car :
- ➕ Garantit la cohérence (une seule source de vérité)
- ➕ Profite des transactions SQLite
- ➕ Permet l'export JSONL si nécessaire
- 🎯 Reste simple et maintenable

**Priorité** : P2 (amélioration de debug, pas critique pour le fonctionnel)

---

**Statut** : REJETÉ (version originale) - ACCEPTÉ (version corrigée)
**Date** : 2026-08-15
**Impact** : Ajout d'une table events dans SQLite, pas de changement d'architecture majeur
