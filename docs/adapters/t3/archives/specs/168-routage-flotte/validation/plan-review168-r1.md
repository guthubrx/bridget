# Archives techniques de l’adaptateur T3

Document d’origine : [specs/168-routage-flotte/validation/plan-review168-r1.md](../../../../../../../specs/168-routage-flotte/validation/plan-review168-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Objection ou audit | Correction ciblée du plan et des contrats | Preuve attendue en développement/livraison |
|---|---|---|
| --bare incompatible avec OAuth | --bare interdit ; --safe-mode attesté par les aides Mac2.1.300/Cartae2.1.251, avec restricted, tools vide, MCP strict vide, setting-sources vide et désactivation des personnalisations. Politiques administrées conservées. | Recette OAuth réelle avec JSON-schema print/stream-json, aucune copie de credential ni outil. |
| Données de décision insuffisantes ou réduites | Chaque niveau consulté reçoit tous les faitsFR021 et modèles/efforts réels ; encodage Laya compact sans perte, validateurs v2 séparés de163. Si tout ne tient pas, capacité indisponible explicite. | Service réel sur ensemble entier compatible (≤8conservateur) ; matrice12+via LLM réel/score ; aucun top8 caché. |
| Conservation stderr inutile/risquée | Aucun stderr conservé ni nouveau tampon ; stdout64Kio et sortie4Kio bornés avant parsing. Deadline dès spawn, permis jusqu’à arrêt réel. | Dépassement/timeout/retour tardif sans fuite, double processus ou journal brut. |
| Mode SQL non compatible | CHECK explicit/automatic/fallback inchangé ; sources détaillées en metadata v2. Mapping explicit/single→explicit, service/LLM→automatic, déterministe→fallback. | Stockage et projectionv4 cohérents ; compatibilité163/v3. |
| Migration et base neuve | DDL168 directement sur base neuve ; reconstruction transactionnelle d’une table163 connue, copieBLOB exacte et trois index. | Migration fichier, hasheslegacy, rollback, réouverture, bornes16/128Kio et FK0. |
| Plafond/capacités/entitlement |128options statiques globales avant filtres ; registre129refusé sans catalogue partiel. Efforts exposés restent déclaratifs. | Catalogue humain/JSON12+, refus129, aucun modèle masqué ni27appels fournisseur imposés. |
| Mesures CPU | Échantillon mémorisé, heartbeat/probe existants, première valeurnull, une collecte machine/lot. | Fraîcheur/époque et absence de nouveau daemon. |
| Rollback ancien binaire | Toute lignev2 interdit l’ancien binaire même168OFF. | Binaire compatible+OFF ou restauration cohérente sans perte. |
| Garantie secrets excessive | Pas d’injection automatique de secrets/config/auth ; brief utilisateur inerte et borné, sans scanner parfait promis. | Charge automatique allowlistée et réponse sans autorité d’exécution. |
| Effort par défaut | Champ présent et résoluble : niveau compatible ou null/liste vide pour défaut natif sans flag. Aucun medium inventé. | Modèle sans niveau conservé ; effort imposé incompatible refusé. |
| Workload dans brief fermé | AutoDelegateV2.workload=general|compilation, défautgeneral, hors HandoffV1 ; canon/snapshot/payload figés. | Compatibilité du brief fermé et influence compilation sans mot-clé. |
| Catalogue humain dépendant de T3 | routing_catalogue_v1 décidé : unix_peer_uid/peer_uids, mêmeUID, Role::Client, version négociée ; aucun human_authority_guard. | Lecture humaine et agent locale par le parcours natif Bridget, projection publique allowlistée et zéro mutation. |
| Shared sans base attendue | AutoDelegateV2.expected_base_commit, requis en tout espaceGit168 ; propagation snapshot/probev2/WorkerPrepare.base_commit, verify_base_commit. HorsGit Shared général seulement : absence permise. | SHA manquant/inexact et base_ref divergente refusés avant fournisseur ; pas de HEAD distant implicite. |
