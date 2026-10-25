# Archives techniques de l’adaptateur T3

Document d’origine : [specs/163-decision-admission/research.md](../../../../../../specs/163-decision-admission/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Objection | Preuve et décision retenue |
|---|---|
| O1 — permis réseau réel | ROOT a relu /Users/moi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/minreq-2.14.1/src/connection.rs:72, :111, :197–207 et :351–380. Sans with_timeout ; MINREQ_TIMEOUT présent, même vide/invalide, rend le seul service non activable et sans réseau. Garde démarrage/préappel, raison visible. DNS, connexion, TLS, écriture ou lecture peuvent retenir le travail indéfiniment ; deadline demandeur distincte de deux secondes, permis jusqu’au retour réel, aucun thread de remplacement. |
| O2 — entretien des reçus | Entretien existant propriétaire, premier passage après réconciliation pour tous les reçus d’époque précédente, lots/CAS/E/S hors mutex. Aucun nouvel HTTP de reprise. pending → probes figés puis choix unique/repli autorisé ou decision_interrupted ; decision_started → repli/refus ; chosen → admission idempotente + lien atomique. Erreurs techniques/SQL restent visibles, non résolues et entretenues. Tout refus après chosen est terminal auto sans étendre les refus natifs. |
| O3 — pression de rétention | UNION 4 096 inclut les refus auto ; seuls les terminaux sans Task sont purgeables sous pression, du plus ancien au plus récent, tombstone trente jours atomique. Rejeu exact jusqu’à purge puis request_expired. Natif et auto tentent la purge sûre avant refus ; états pending/chosen/ambigus/vivants protégés. Les cinq refus natifs restent hors compteur. |
| O4 — lecture des versions | v3 → v2 seulement unsupported_version structuré ; v2 → v1 conserve unsupported_version ou invalid_request structurés. Pas de mélange de pages ni de repli journal/cancel. |
| O5 — collision des entrées | Natif externe consulte reçu auto avant by_agent_request sous Immediate. L’autorité interne chosen inscrit les refus uniquement dans le reçu auto, sans native_refusal sous la même clé. owner stable et unicités historiques conservés. |
| O6 — réalisation sur Cartae | Rust/compilation/tests Linux sur Cartae, namespace privé et fixtures isolées ; Mac pour T3/tests OS/signature/installation. Recette décideur depuis maître Mac, même commit source pour les builds. Une compilation active dans la flotte ; défaut de connecteur hors163 mais bloquant pour lancement distant. |
