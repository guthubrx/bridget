# Recherche 101

Date : 2026-09-16. Exploration locale en lecture seule ; aucun fournisseur arrêté.

## Décisions et preuves

- `mcp.rs:359` résout une identité avant chaque appel ; MCP réel `bridget_who`
  échoue `identity_not_found`. Ne pas supprimer le contrôle pour résoudre l'usage privé.
- `t3code.rs:1005` connaît fil → UUID → instance et `wrapper.rs:1636` conserve
  déjà la preuve 099. Il manque le lien vers le processus fournisseur.
- Recherche indépendante : un processus Codex T3 ouvre un rollout parent et
  un rollout de sous-agent ; le plus récent n'est pas une identité fiable.
- T3 local `apps/server/src/provider/Layers/CodexSessionRuntime.ts:1178` et
  `CodexAdapter.ts:2290` créent un runtime dédié par fil. SQLite
  `provider_session_runtime.resume_cursor_json.threadId` fournit l'ID Codex.
- T3 `ClaudeAdapter.ts:4693` passe resume/sessionId ; SDK installé les transforme
  en flags natifs. Pas de Claude actif pendant la recherche : recette réelle à faire.
- Décision : croisement exact d'identifiants natifs et filiation OS, lecture SQLite
  strictement limitée et read-only dans l'adaptateur. Alternatives rejetées :
  cwd/titre/mtime (ambigu), demander UUID au modèle (usurpation), modifier T3
  (périmètre externe), lire tous les transcripts (coût/confidentialité).
- Dépendance au schéma T3 assumée et testée par contrat ; forme inconnue = refus.
  Maintenance : petit module d'adaptateur, mêmes primitives de preuve que 099.
- `t3code_contract.rs:337` ignore actuellement les activités disponibles ;
  `t3code.rs:1794` écrit une fin sans `stop_reason`, écart avec
  `journal.rs:530`. La corrélation anti-boucle doit précéder le raccordement.
- T3 `packages/contracts/src/orchestration.ts:453` ferme les états de tour à
  running/interrupted/completed/error. Une valeur inconnue n'est pas finale.
- T3 `ProviderRuntimeIngestion.ts:373` produit approval.requested ; ligne838
  produit tool.completed. `ActivityPayloadProjection.ts:359` conserve des paths,
  mais leur présence seule ne prouve pas une écriture : exiger un type et succès.
- `observation.rs:47` accepte aujourd'hui tout filtre sans capacité source.
  Réutiliser cet état et le daemon, ne pas créer un second moteur d'événements.
- Après restart, mémoire perdue : une trace bornée des abonnements dans Store
  est nécessaire pour distinguer « interrompu » de « aucun abonnement ».
  Pas de garantie artificielle de livraison durable.

## Sources primaires consultées

Baseline utilisateur : security-compliance et testing-quality ; principes retenus,
pas de métriques anciennes utilisées comme chiffres actuels.

- https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html :
  refus par défaut et vérification de l'autorité ; aucun élargissement de droits.
- https://www.cisa.gov/sites/default/files/2023-06/principles_approaches_for_security-by-design-default_508c.pdf :
  responsabilité produit plutôt que configuration de sécurité répétée à l'utilisateur.
- https://martinfowler.com/articles/practical-test-pyramid.html : tests aux frontières
  de sérialisation et intégrations réelles, distincts des tests unitaires.
- https://testing.googleblog.com/2008/03/tott-understanding-your-coverage-data.html :
  couverture de lignes différente de preuve du comportement.

## Outillage indisponible et substituts

Scripts/modèles `.specify/scripts` et `.specify/templates` absents (vérifiés ls),
`mem` absent (command -v), Sequential Thinking absent du catalogue : protocole
SpecKit manuel et décisions écrites ici. Aucun runtime réinstallé.
Contre-revue autre fournisseur : canal MCP refuse actuellement l'identité ;
à retenter après correction, pas de CLI usurpant un autre agent.

## Vérification du contrat réel pendant implémentation

- T3 conserve provider_name=claudeAgent : normalisation avec agent_type_for existant.
- 35/35 tours du fil courant ont un unique message utilisateur dont createdAt
  égale requestedAt. La corrélation ne compare ni texte, ni rang, ni horloge daemon.
- Première métadonnée du rollout parent : 18 578 octets, sous la limite64Kio.
- /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration/ActivityPayloadProjection.ts
  ne conserve pas input.file_path/notebook_path. Quatre Write Claude réels ont
  perdu le chemin dans la projection. ClaudeAdapter synthétise aussi certains
  completed sans résultat propre d'outil. Donc aucune capacité FileWritten
  Claude via T3 ; les wrappers structurés restent indépendants et inchangés.
- Fenêtre T3 : 500 activités avant compression, 12 chemins par activité.
  Un tour réel compte604activités ; quatre opérationsCodex dépassent12chemins,
  maximum22. Limites exposées dans catalogue/doc ; perte de recouvrement,
  origine inconnue et seuil de chemins provoquent une notice de lacune de
  quantité inconnue, sans fausse déconnexion ni nombre de pertes inventé.
- Une métadonnée illisible invalide toute collecte d'identité. Ne pas effacer
  un processus du décompte : il peut rendre ambigu un autre processus lu correctement.
- Inventaire OS groupé mesuré sur macOS à38–69ms dans les recettes unitaires,
  borne2s. Cela n'est pas une mesure de la latence de notification réelle.
- CLIClaude : installation actuelle native Mach-O, commande par défaut claude,
  aucun binaryPath personnalisé. Le SDK autorise aussi node/bun + script.js,
  variante non reconnue par le filtre strict actuel et non déclarée compatible.
  Aucun Claude enfant de T3 actif pendant vérification ; brancheClaude validée
  sur fixtures, pas par lancement payant ni redémarrage d'un fournisseur.

## Preuves après adoption101 autorisée

Le16septembre à17:02CEST : CodexT3 reconnu par son MCP préexistant (abonnement
Horizon propriétaire exact), Claude natifT3 reconnu par CLI (MCP absent dans son
catalogue, repli annoncé par le témoin). Réception réelle dans filClaude bdget,
4s après observation de fin, une seule notification ponctuelle. La limitation
node/bun n'est pas levée par ce test. Aucun fournisseur existant redémarré.
Le refus lsof est sûr et journalisé précisément dans le pont ; le client voit
encore un refus d'identité générique. Limite ergonomique déclarée, pas un succès
fictif ni une raison d'affaiblir l'attestation.
