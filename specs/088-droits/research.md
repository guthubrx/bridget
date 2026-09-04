# Research: Droits lisibles et vérifiables

Date : 2026-09-03. Sources : code de `main` `a931a8ac`, journal Codex `rollout-2026-09-03T05-46-20-…jsonl` sur le serveur, SPEC-080, 083, 085, 086, 087, ADR-011, ADR-027.

## R1. Quelle couche a produit chacun des deux refus observés ?

**Constat** : « Bloqué par vos réglages locaux » vient de `crates/bridget-daemon/assets/ui/app.js` (`renderContentReferences`, vers la ligne 7241) quand `normalizeContentSecurityPreferences` rend `externalLinks: false`. Les trois préférences sont dans `localStorage` sous `bridget.content-security.v1`, toutes à `false` par défaut (`defaultContentSecurityPreferences`, ligne 7126), ou pilotées par Bridget Desktop via `window.__BRIDGET_CONTENT_SECURITY__`. Le blocage du shell vient de la sandbox Bubblewrap de Codex lancé en `--sandbox read-only` : c'est la posture « découverte » du registre (`crates/bridget-daemon/src/registry.rs`, `project_discovery_definition`, ligne 1133), pas un réglage Bridget visible. L'erreur `bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted` arrive dans la sortie de commande, que le wrapper observe déjà (`crates/bridget-transport/src/codex_app_server.rs`, `item/commandExecution/outputDelta`, ligne 2198) sans l'interpréter.

**Décision** : ne créer aucune couche. Rendre lisibles les six existantes.

## R2. Où vivent les lignes de la page Droits ?

**Décision** : chaque ligne pilote un mécanisme existant, sans copie :

| Ligne | Mécanisme | Lieu | Modifiable depuis le serveur ? |
|---|---|---|---|
| Liens externes, Images distantes, Aperçus de fichiers | préférences de sécurité du contenu (`app.js:7058`) | navigateur, ou Desktop | non, par conception |
| Artefacts HTML | isolation SPEC-083, toujours active | navigateur | non ; ligne en lecture seule |
| Internet, Fichiers du projet, Shell | posture d'agent : `complete` (lancement avec contournement de sandbox, `wrapper.rs:1907`) ou `discovery` (`--sandbox read-only`, `registry.rs:1142`) | serveur, colonne `agent_posture` de `control_state` (087) | oui, principal humain |
| Modifier Bridget | réglage expert `dogfooding.bridget` (SPEC-086, `control_settings.rs:183`) | serveur | oui, avec confirmation existante |
| Pause, Plafond | `control_state` (SPEC-087, `referent_control.rs`) | serveur | oui, routes `/v1/control/state` |
| Réassignation automatique | garde unique Maicie `admit_autonomous_effect` (087), nouvel effet `Reassignment` | serveur, colonne `auto_reassignment` de `control_state`, lue par Maicie via `ControlStateFrame` | oui |

**Rationale** : les trois lignes d'agent sont aujourd'hui une seule posture ; les séparer exigerait des postures intermédiaires que Bridget ne sait pas lancer (FR-016). La page montre donc les trois lignes avec la même valeur réelle, et l'expert voit qu'elles sont liées.

**Révision après contre-revue (Jim, 14:16)** : la première version plaçait posture et réassignation dans un fichier du relais, `server-rights.json`, avec sa propre génération et une écriture en deux temps (fichier puis `ControlStateSet`). C'est exactement l'assemblage non atomique que l'ADR-027 rejette. Décision révisée : les deux valeurs sont des colonnes de `control_state`, écrites par `ControlStateSet` (champs optionnels additifs `agent_posture`, `auto_reassignment`), journalisées dans `control_events` (`rights_set`), sous la même génération que la pause et le plafond. Le profil n'est pas stocké : il est dérivé des trois valeurs serveur par `profile_for` (`Custom` si aucune ligne de la matrice ne correspond). Le superviseur de flotte lit la posture dans le daemon au lancement ; le relais ne possède plus rien sauf les résultats de tests.

**Migration** : base neuve ⇒ `discovery`, réassignation `0` (Prudent, FR-012). Base existante ⇒ colonnes ajoutées avec `complete` et `1`, le comportement observable du serveur avant la mise en service ; la page affiche alors Équilibré ou Personnalisé. À confirmer par le référent.

**Alternatives écartées** : fichier du relais (non atomique, deux autorités) ; une table Maicie (Maicie ne possède pas les droits, elle les lit).

## R3. Comment Maicie lit-elle « réassignation automatique » ?

**Décision révisée** : `ControlStateFrame` gagne `auto_reassignment: Option<bool>` et `agent_posture: Option<AgentPostureFrame>`, défaut `None` (`serde(default)`). `None` signifie « daemon antérieur ou état inconnu » et Maicie le traite comme la pause : effet différé, motif `droits`. Jamais de défaut `true` (fail-open relevé par la contre-revue). Côté Maicie : `ControlStateWire` et `ControlSnapshot::Read` portent le champ ; la garde unique `admit_autonomous_effect` reçoit un effet `Reassignment` ; `reduire_reassignation` n'a pas de condition parallèle, c'est son appelant qui consulte la garde.

**Piège connu** (mémoire projet) : un champ additif sur le fil n'est pas additif à la source ; `cargo test --workspace --no-run` avant de conclure.

## R4. Comment le wrapper reconnaît-il un refus de sandbox ?

**Décision révisée** : une ligne de sortie n'est pas une preuve : un agent peut l'imprimer. Le wrapper Codex journalise désormais la FIN de chaque `commandExecution` (`item/completed`, type `commandExecution` : état, code de sortie, sortie agrégée bornée), ce qu'il ne faisait pas. Un acte `refusal` n'est écrit que si, pour le MÊME item, une ligne de la liste fermée (`refusals.rs`) est apparue dans la sortie ET la commande s'est terminée en échec. Il est rendu comme « signalement non attesté » (`evidence: output_and_exit`) avec le geste « Tester » ; jamais comme une cause établie (FR-005). Témoin : `echo 'bwrap: … Operation not permitted'` avec sortie 0 ⇒ aucun acte. Si Codex n'émet pas de fin structurée sur ce serveur (à mesurer avant T006), aucun acte n'est écrit et la ligne reste dans la sortie brute, jamais nommée sandbox. Claude : pas de Bubblewrap, pas de détection dans ce lot.

## R5. Comment « Tester » constate-t-il un droit ?

**Décision révisée** : une tentative corrélée et terminale, jamais une fenêtre de trente secondes sur le journal. `POST /v1/control/rights/test` crée une tentative `{ test_id, line, agent_id, expected_command, expected_sha256, started_at }`, envoie le message par `post_ui_message` (identité humaine, intention `TriggerTurn`) et enregistre le `message_id` rendu ; la réponse est immédiate, issue `pending`. À chaque `GET /v1/control/rights`, le relais relit le journal de l'agent pour ce `message_id` seulement : un acte `command` dont le texte est exactement `expected_command`, terminé (`state: completed`, `exit_code: 0`) avec le jeton dans la sortie ⇒ `passed` ; le même item terminé en échec avec un acte `refusal` du même `message_id` ⇒ `refused_provider_sandbox` ; un `curl` terminé en échec avec un code réseau ⇒ `unreachable` ; aucune terminaison après 120 s ⇒ `unknown_expired`. Un agent occupé au moment du dépôt ⇒ refus `busy` ; si le message reste en file (`in_flight`), la tentative reste `pending` puis expire. Un tour concurrent ne peut pas être confondu : la corrélation est par `message_id` et commande exacte.

Commandes fermées (FR-021) : Shell `printf 'BRIDGET-TEST-%s
' <jeton>` ; Fichiers `head -c 0 <fichier connu du projet> && printf 'BRIDGET-TEST-%s
' <jeton>` ; Internet `curl -sS -o /dev/null -w '%{http_code} BRIDGET-TEST-<jeton>
' https://example.com/` ; Modifier Bridget : si `dogfooding.bridget` est `disabled`, `refused_bridget` sans solliciter l'agent ; sinon même lecture qu'à la ligne Fichiers sur le checkout système. Limite assumée (ADR-011) : un agent qui ment en imprimant le jeton sans exécuter la commande n'est pas dans le modèle ; la mesure vise l'agent contraint, pas l'agent hostile.

## R6. Profils

Matrice retenue (à confirmer par le référent, hypothèse de la spec) :

| Ligne | Prudent | Équilibré | Confiant |
|---|---|---|---|
| Liens externes | fermé | au clic | au clic |
| Images distantes | fermé | fermé | au clic |
| Aperçus de fichiers | fermé | au clic | au clic |
| Artefacts HTML | isolés (fixe) | isolés | isolés |
| Internet, Fichiers, Shell (posture) | découverte | complète | complète |
| Modifier Bridget | fermé | fermé | fermé (expert seulement) |
| Pause | inchangée | inchangée | inchangée |
| Plafond | 5 | 5 | 20 |
| Réassignation automatique | différée | active | active |

Les lignes locales d'un profil sont appliquées par le navigateur qui a choisi le profil ; un autre navigateur affiche « Personnalisé » tant que ses lignes locales diffèrent (FR-013).

## R7. Forme unique d'un refus

**Décision** : un objet `{ layer, prevented, gesture, raw, at, attributed_to, evidence }` avec `layer` fermé : `browser_content`, `artifact_sandbox`, `provider_sandbox`, `server_runtime`, `bridget_system`, `referent_control`, `bridget_policy`, `unknown`. Rendu par une seule fonction `renderRefusal` dans `app.js`, réutilisée par la carte de lien, la carte d'acte `refusal`, et les refus déjà affichés (pause, plafond, principal humain, capacité). `gesture` est soit une action locale (activer une préférence), soit un lien vers une ligne de la page Droits, soit absent. **Provenance (contre-revue P2)** : un geste local ne peut être créé que par le rendu local à partir des préférences locales ; tout refus venu d'un acte du journal, du daemon ou d'un message est normalisé sans geste local avant rendu, et `renderRefusal` n'honore `local_toggle` que sur demande explicite du rendu local. Un serveur relié ne peut donc pas fabriquer une carte « Autoriser les liens ».

## Baseline research (Article IX)

`~/.speckit/research/06-security-compliance.md` et `03-cognitive-load-productivity.md` : réduire la charge cognitive par des refus explicables et un lieu unique ; pas de validation live nécessaire, aucun framework ni fournisseur choisi.
