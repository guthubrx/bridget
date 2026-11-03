# Validation 231 — avant activation

## Diagnostic constaté

Fil : <t3-thread://v2/3b1ba3d1-3df7-4d49-a17f-45154959c3a9/2c432650-9f99-4d47-ae35-101673d74562>.
Réglages réels `enableAgentDeviceAccess=true`, `enableDeviceSupport=true`.
Appel natif unique du propriétaire : `permission_attestation_unavailable`,
« The provider permission fact is missing, stale or no longer owned by this run. »
Les options réelles ont le callback des outils. L’image installée et la source
contiennent la garde globale hooks absent ; le générateur ajoute son propre
SubagentStart pour les appareils. Le test échoue sur inputs undefined avant
correction. Aucune dépendance à un numéro de version du programme Claude.

## Correctif client

Source : 97c8a892cdc98c9d94cc0d1f7224fab11be41541, intégrée dans la branche locale
principale `local/main-20261102` et poussée sur le dépôt guthubrx/t3code.
Deux fichiers, dans /Volumes/8TB2/50-repos-archives/11.Repositories/t3code/apps/server/src/orchestration-v2/Adapters/ :
ClaudeAdapterV2.ts et ClaudeAdapterV2.test.ts. Le contrôle reconnaît seulement
la fonction interne, un événement, un matcher sans autre champ et un callback.
Il est revérifié même si le contexte de lancement n’est pas disponible.
Les autres crochets, les événements ajoutés et une mutation restent refusés.

## Contrôles et limites

- 25 tests de capture/publication acquis après correction, avant déplacement
  de la régression vers la suite Claude existante conforme au lint actuel.
- 305 tests de la suite Claude passent, dont la nouvelle régression.
- Format, lint, GraphQL et types des trois paquets concernés passent dans
  le checkout isolé. Aucun test général répété.
- `check:changed` n’est PAS entièrement vert : Knip signale six exports
  historiques non utilisés au checkout source ; les liens de dépendances du
  checkout isolé amplifient ces faux non-usages. Le budget outils historique
  attend 47538 octets et mesure 47860. Ces contrats ne changent pas dans ce lot.
- Le hook de commit passe format/lint, puis échoue sur ces six exports. Le
  commit a été effectué sans hook après contrôles ; ce refus n’est pas déclaré
  réussi. Les anciennes violations de lint de BridgetPermissions149.test.ts
  sont conservées ; ce fichier n’est pas modifié dans le commit livré.
- Revue indépendante refusée `cwd_outside_parent_project`, clé
  bridget231-revue-attestation-crochet-appareils-20261103-01, not_recorded,
  task=null. Aucun changement de parent ni repli de moteur. Auto-relecture
  déclarée ; le delta reste petit et n’ajoute aucune autorité.
- Application 0.1.7-local.20261103.attestation-r1 construite en 85,73 s.
  Signature ad hoc vérifiée. Démarrage du paquet sur une base fixture isolée,
  deux routes HTTP200, migration77, arrêt normal du processus détenu.
  Aucun accès en écriture à la base vivante. Moniteur natif inchangé réutilisé.

## Conservation

Aucun Start, Cancel, Fence ou rejeu de 77ba483e, de la délégation T3 session194
ou de la tâche7024. Les preuves privées sont conservées dans
/Users/moi/.cache/bridget231-claude-attestation/.
Le changement utilisateur préexistant de AGENTS.md du dépôt client est conservé.
Les archives locales de qualification sont conservées et ignorées seulement
par le fichier privé Git info/exclude. Aucun effacement de ces archives.

Activation et lecture réelle : encore à vérifier à ce point de passage.
Correction technique sans changement visuel : pas de diaporama nécessaire.
