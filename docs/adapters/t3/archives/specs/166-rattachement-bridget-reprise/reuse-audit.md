# Archives techniques de l’adaptateur T3

Document d’origine : [specs/166-rattachement-bridget-reprise/reuse-audit.md](../../../../../../specs/166-rattachement-bridget-reprise/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Lecture de la preuve courante | `McpProviderSession.readMcpProviderSession()` | `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts:1812` | L'autorité par fil existe déjà. |
| Préparation de la configuration | `threadRuntimeParams()` | `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts:1809` | Prépare déjà Bridget et T3 Code MCP. |
| Mémoire de la preuve montée | `bridgetCredentialsByThread` | `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts:1805` | Aucun registre parallèle n'est requis. |
| Reprise du fil natif | Requête `thread/resume` existante | `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts:6583` | Elle conserve le même fil et recharge sa configuration. |
| Publication des droits | `publishMcpProviderPermissions()` | `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts:6282` | L'attestation reste liée au run. |
| Test de protocole | Banc de rejeu Codex | `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.test.ts:1840` | Il contrôle l'ordre et la configuration des requêtes. |

<a id="extrait-2"></a>

## Extrait 2

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| SPEC-149 | Identité privée distincte par fil | La preuve reste indexée par `ThreadId`. |
| SPEC-163 | Session fournisseur partagée et reprise | Le processus partagé ne doit pas fusionner les identités. |
| SPEC-165 | Cycle de vie borné des liens T3 | Le défaut courant est séparé de la limite de fichiers déjà corrigée. |

<a id="extrait-3"></a>

## Extrait 3

| Requete | Portee | Resultat |
|---|---|---|
| `rg "prepareMcpSession|bridgetCredentialsByThread|startNativeTurn"` | Adaptateur et gestionnaire T3 | Montage, mémoire et contrôle du tour trouvés. |
| `rg "thread/resume|resumeThread"` | Adaptateur et tests T3 | Reprise native et traitement des archives trouvés. |
| `rg "McpProviderSession|credential stable|shared app-server"` | Tests T3 | Couvertures proches trouvées, sans rotation avant tour. |
| Inspection SQLite et journaux expurgés | Runtime T3/Bridget | Session partagée et échec limité au fil repris confirmés. |
