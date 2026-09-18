# Contrat104 — Recherche et relecture

## MCP bridget_ledger

action absent/recent : contrat actuel view/limit/requests_scope, sans nouvelles restrictions
rétroactives. Les paramètres de recherche n'y sont pas acceptés silencieusement.
action=search : champs définis data-model.md ; source défaut messages ; query obligatoire.
action=read : id et target obligatoires ; offset=0 par défaut ; digest selon offset.
view/requests_scope interdits dans search/read ; autres champs inconnus invalid_params.

```json
{"action":"search","query":"pagination erreur","peer":"11111111-1111-4111-8111-111111111111","limit":20}
```

```json
{"action":"search","source":"thread","thread_id":"22222222-2222-4222-8222-222222222222","query":"décision"}
```

```json
{"action":"read","id":"handoff-pagination-01","target":"11111111-1111-4111-8111-111111111111"}
```

Suite : même query/source/filtres, cursor reçu copié sans le fabriquer. La réponse peut être
hits=[] avec has_more=true. Arrêter seulement sur has_more=false, ou annoncer sa recherche
partielle s'il suffit d'un résultat. Ne pas boucler automatiquement jusqu'à épuisement.
Pour un fil, l'agent consulte bridget_thread action=history avec from_seq/to_seq explicites ;
ne pas utiliser read puis ack pour chercher une ancienne décision.

## CLI équivalent

bridget ledger --limit 20 (inchangé)
bridget ledger search --query "pagination erreur" [--peer UUID] [--author UUID]
  [--since UNIX] [--until UNIX] [--limit N] [--cursor HEX] [--json]
bridget ledger search --source thread --thread-id UUID --query "décision" [autres options]
bridget ledger read --id ID --target UUID [--offset N --digest HEX] [--json]

--source messages est le défaut ; peer interdit en source thread. Valeurs non reconnues
refusées ; --cursor exige de répéter query/filtres comme MCP. Ne pas lire un curseur depuis
un chemin, variable globale ou état implicite. Identité attestée actuelle uniquement.
Sortie humaine : extraits, références et « suite disponible » ou « fin de la partie conservée ».
CLI codes0 succès même zéro résultat,2 paramètres invalides,1 refus/erreur ; --json homogèneMCP.

## Trames proposées

WrapperToDaemon::LedgerSearch { request: LedgerSearchRequest }
WrapperToDaemon::LedgerRead { request: LedgerReadRequest }
DaemonToWrapper::LedgerSearchResult { outcome: LedgerSearchOutcomeV1 }
DaemonToWrapper::LedgerReadResult { outcome: LedgerReadOutcomeV1 }

Enums outcome avec succès typé ou refus {code,reason}, serde strict sur requêtes.
Requêtes ne contiennent ni agent_id appelant ni instance ni chemin de base.
Chemin client registered_connection réutilisé, preuve auxiliaire avant toute requête.
Daemon contrôle live_connection_identity au départ ET avant d'envoyer les corps.
Fils : contrôle membership dans la transaction de lecture, même source que102.
Pas d'accès humain sans identité membre, même si le ledger historique possède une route globale.

Erreurs : invalid_params, invalid_cursor, identity_unavailable, not_found_or_forbidden,
storage_unavailable, busy, source_too_large, source_metadata_too_large, content_changed, capability_unavailable,
daemon_protocol. Pas de corps, titre ou existence d'une source interdite dans les erreurs.
SQL échouée → storage_unavailable, jamais hits=[] ; erreurs de décodage de ligne propagées.
Daemon ancien : afficher incompatibilité ou erreur de protocole, ne pas réessayer par global.

## Pagination normative

Messages ordre DESC(ts,id,target), borne exclusive before ; fil ordre DESC(seq).
Ne jamais utiliser OFFSET, rowid ou seulement ts/id.
Si limite de résultats ou sortie atteinte AVANT traitement d'une ligne, ne pas l'acquitter
dans le curseur. scanned_count augmente seulement pour un candidat consommé.
Si limit atteint sur la dernière ligne existante : has_more=false, next_cursor=null ;
stop_reason=exhausted a priorité. Si un témoin supplémentaire existe : suite.
Corps>16Mio : consommer la clé et incrémenter skipped_oversized sans charger le corps ;
métadonnées>256octets : refuser source_metadata_too_large, aucune réponse partielle.
Ne pas tronquer une clé pour avancer. Les identifiants historiques autorisés sont des
chaînes, pas nécessairement des UUID ; seules les identités appelantes restent attestées.

Réponse totale≤61 440octets pour l'objet outcome compact avant enveloppeMCP.
Lecture corps : fragment≤16 384octets UTF-8, next_offset à frontière ; offset=body_bytes
retourne fragment vide et next_offset=null, offset>body_bytes ou milieuUTF-8 invalid_params.
Empreinte différente : content_changed sans fragment ; le client recommence offset0
seulement s'il veut lire la nouvelle version. Référence perdue : pas de recherche globale
automatique ni de choix d'un autre message ayant le même id.
