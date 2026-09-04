# Contre-revue adverse — SPEC-088 (plan)

Date : 2026-09-03, demande envoyée 14:07 CEST par la ligne de commande du serveur (identité `human`, sans réponse possible par message), verdict rendu par écrit à 14:16 CEST.
Agent : Jim, fournisseur Codex (`gpt-5.6-terra`), UUID `437b3175-76b1-4337-ae44-e7b1ce8bb809`.
Question posée : quatre questions précises sur la reconnaissance des refus de sandbox, la conformité ADR-003/027 du champ `auto_reassignment` et du fichier de droits, les faux résultats possibles de « Tester », et le desserrage d'une ligne locale par un serveur relié.
Verdict reçu : **BLOCKED**.

## Instruction des objections

| Objection | Vérifiée comment | Retenue | Raison et changement |
|---|---|---|---|
| P0 stdout `bwrap` assimilé à une preuve | `codex_app_server.rs:2198` : `outputDelta` est du texte sans provenance ; un `echo` produit les mêmes octets | oui | Le signalement exige la ligne reconnue ET la fin en échec de la même `commandExecution` (`item/completed`, à mesurer sur le serveur) ; il est rendu « signalement non attesté » avec le geste Tester ; `echo` + sortie 0 ⇒ aucun acte (témoin ajouté). Si Codex n'émet pas de fin structurée : aucun acte, jamais nommé sandbox. |
| P0 `command` = démarrage, pas réussite | `codex_app_server.rs:2159` : acte à `item/started` ; `item/completed` traité seulement pour `agentMessage` (`:2104`) | oui | Le wrapper journalise désormais la fin d'une commande (état, code de sortie, sortie bornée) ; « réussi » exige la fin réussie et le jeton. |
| P0 aucun lien tentative/message/commande/exécution | `data-model.md` ne portait ni `test_id` ni `message_id` | oui | Tentative corrélée : `test_id`, `message_id` rendu par `/v1/send`, commande canonique et son empreinte, identifiant d'item, issue terminale ; hors fenêtre ⇒ `unknown_expired`. |
| P1 `false` au fichier, `true` par défaut sur la trame | `data-model.md` et `research.md` R3 | oui | Champ `Option<bool>`, défaut `None` = inconnu ⇒ Maicie diffère (motif `droits`). Aucun défaut `true`. |
| P1 deux autorités, deux générations, écriture `partial` | `contracts/rights-v1.md` §apply | oui | `agent_posture` et `auto_reassignment` rejoignent `control_state` et `control_events` ; un seul `ControlStateSet` porte profil, posture, réassignation et plafond ; `server-rights.json` supprimé ; le profil est dérivé. |
| P1 `daemon.rs` et `bridget_client.rs` hors périmètre | `plan.md` structure | oui | Ajoutés : bras `ControlStateSet`/`Read` du daemon, `ControlStateWire`, `ControlSnapshot`, garde `admit_autonomous_effect(Reassignment)`. |
| P2 `isTrusted` ne fixe pas la provenance du geste local | `renderRefusal` acceptait `local_toggle` de n'importe quelle source | oui | `local_toggle` n'est honoré que si le rendu local le demande (`allowLocalToggle`), et tout refus venu d'un acte, du daemon ou d'un message est normalisé en `none` ; essai adverse ajouté. |

Désaccord résiduel : aucun. Une précision sur la migration : le défaut Prudent (FR-012) vaut pour une base neuve ; une base existante conserve son comportement observable (posture complète, réassignation active) et affiche le profil Équilibré ou Personnalisé, pour ne pas changer le comportement d'un serveur en production au redémarrage. À confirmer par le référent.

## Verdict brut de Jim

# Contre-revue adverse — PLAN SPEC-088 « Droits lisibles et vérifiables »

**Verdict : BLOCKED**

Les deux promesses qui fondent cette SPEC — attribuer un refus de sandbox sans inventer sa cause et mesurer un droit réel — ne sont pas démontrables avec les signaux retenus. Le plan confond des octets que l’agent peut produire avec une preuve émise par le runtime, puis confond le démarrage d’une commande avec son succès.

Référence examinée : a931a8acebc59463899b4e7e102c721d21413bcf, comme demandé. Fait de reproductibilité : le ref local main résout actuellement vers bf93c3fe53ee3ac9eeadfc405c37c1175fb19266 ; la présente revue porte exclusivement sur a931a8ac.

## Réponses aux quatre questions

### 1. Refus sandbox reconnu dans outputDelta

**Non : cette reconnaissance n’est ni fiable ni non contournable.**

Le code de référence qualifie outputDelta de « sortie stdout uniquement », sans provenance système, code de sortie ni événement terminal :
- /home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-transport/src/codex_app_server.rs:2198-2208 (a931a8ac)

Le contrat ne fait qu’interpréter une ligne commençant par bwrap :
- /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/contracts/rights-v1.md:17-22
- /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/plan.md:66-67

Un agent dont le shell est disponible peut imprimer exactement « bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted ». Les octets sont alors indiscernables d’un vrai diagnostic Bubblewrap. Exiger le début de ligne, réduire la regexp ou dédupliquer par exécution ne résout pas cette indistinguabilité. Le résultat serait un faux refus provider_sandbox attribué à Bridget, contraire à FR-005 :
- /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/spec.md:117-119
- /home/moi/bridget-referent/.worktrees/088-droits/docs/decisions/028-refus-explicites-et-page-droits.md:11-13

### 2. auto_reassignment, server-rights.json, ADR-003 et ADR-027

**La direction est compatible avec ADR-003 uniquement si Maicie ne lit jamais le fichier ; le plan viole ADR-027 tel qu’il est écrit.**

Le passage par une trame daemon → Maicie préserve la frontière compagnon d’ADR-003 : Maicie doit utiliser les interfaces publiques Bridget, sans lire ses fichiers ni importer ses modules :
- /home/moi/bridget-referent/.worktrees/088-droits/docs/decisions/003-maicie-compagnon-orchestration.md:20-38

ADR-027 impose toutefois un état de contrôle durable dans le daemon, une lecture à chaque relève et une attitude conservatrice lorsque l’état est inconnu :
- /home/moi/bridget-referent/.worktrees/088-droits/docs/decisions/027-etat-de-controle-et-voie-humaine-attestee.md:11-18,30-34

Le plan ajoute une seconde autorité mutable, avec une génération séparée, puis veut la joindre à la trame sans génération commune ni transaction commune. Le défaut de sûreté est explicite : le fichier est initialisé à false, alors que le champ absent de la trame est défini à true :
- /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/data-model.md:6-11,23-25
- /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/research.md:28-32

Une absence, migration ou erreur de lecture peut donc devenir une autorisation de réassigner alors que le profil Prudent la diffère. C’est un fail-open.

Le plan oublie aussi les véritables points d’intégration. La trame est aujourd’hui créée depuis SQLite dans referent_control.rs puis renvoyée telle quelle par daemon.rs :
- /home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/referent_control.rs:199-218 (a931a8ac)
- /home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/daemon.rs:8696-8716 (a931a8ac)

Pourtant daemon.rs n’est pas dans le périmètre annoncé :
- /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/plan.md:46-59

Et Maicie désérialise sa propre ControlStateWire, puis construit un ControlSnapshot sans ce champ :
- /home/moi/bridget-referent/.worktrees/088-droits/plugins/maicie/src/bridget_client.rs:1207-1220 (a931a8ac)
- /home/moi/bridget-referent/.worktrees/088-droits/plugins/maicie/src/control.rs:14-45 (a931a8ac)
- /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/plan.md:74-75

Enfin, apply écrit le fichier puis exécute un ControlStateSet séparé, avec une réponse partial si la seconde étape échoue :
- /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/contracts/rights-v1.md:7-10

C’est un assemblage non atomique des droits, du plafond et de la réassignation, alors que cette absence d’atomicité est précisément l’alternative rejetée par ADR-027.

### 3. Tester : message à jeton et journal sous trente secondes

**Oui : la conception peut produire de faux réussis, de faux refus et de faux négatifs.**

Le journal crée l’acte command dès item/started, à partir du texte de commande annoncé, donc avant toute exécution complète :
- /home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-transport/src/codex_app_server.rs:2146-2175 (a931a8ac)
- /home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-transport/src/codex_app_server.rs:1407-1437 (a931a8ac)

Le code ne traite item/completed que pour agentMessage ; il ne collecte pas de terminaison commandExecution :
- /home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-transport/src/codex_app_server.rs:2102-2144 (a931a8ac)

Le test prévu déclare pourtant « acte command avec jeton ⇒ passed » :
- /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/plan.md:81-83

Cela suffit pour un faux réussi : la commande peut démarrer puis échouer dans Bubblewrap ; l’acte command arrive avant le refus. L’agent peut aussi émettre une autre commande qui contient le jeton, ou une commande qui échoue après son démarrage.

La corrélation disponible n’est pas portée par le contrat. /v1/send retourne bien un message_id durable :
- /home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/ui.rs:1481-1490,7135-7173 (a931a8ac)

Mais RightsTestRecord et l’acte refusal ne contiennent ni test_id, ni message_id, ni execution_id, ni empreinte de la commande attendue :
- /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/data-model.md:13-34

« Un acte refusal dans la fenêtre » peut donc attribuer le refus d’un autre tour du même agent :
- /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/research.md:40-46

Enfin, « agent libre » suivi d’un envoi n’est pas une réservation atomique. L’envoi retourne in_flight et peut attendre dans la file :
- /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/plan.md:81-82
- /home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/ui.rs:7135-7173 (a931a8ac)

Un tour concurrent peut démarrer entre contrôle et envoi, ou le test peut réussir après trente secondes. Le résultat doit alors être inconnu/expiré, jamais assimilé à un refus ou à une mesure négative.

### 4. Ligne locale et serveur relié

**Pas par écriture directe dans le modèle décrit, mais la preuve de non-desserrage est insuffisante.**

La base est correcte : GET rights exclut les lignes locales et le navigateur les compose ; la préférence actuelle est écrite dans le localStorage du navigateur :
- /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/contracts/rights-v1.md:3-5
- /home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/assets/ui/app.js:7125-7160 (a931a8ac)

Un serveur relié ne peut donc pas écrire cette clé directement. Mais le plan protège le geste local avec isTrusted seulement :
- /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/plan.md:68

isTrusted prouve un clic navigateur, pas la provenance de la carte qui expose ce clic. Si renderRefusal accepte un Refusal distant qui porte gesture.local_toggle, un serveur peut fabriquer une carte « Autoriser les liens » et un clic humain vrai desserrera la préférence. Ce n’est pas un contournement silencieux, mais c’est un confused deputy contraire à la frontière annoncée :
- /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/spec.md:126-131

Le plan doit réserver local_toggle au renderer local : cible fermée, littérale, construite uniquement à partir de readContentSecurityPreferences. Toute donnée provenant du serveur, du journal, d’un agent ou d’un serveur relié doit rester affichable, mais ne doit jamais porter une mutation localStorage. Le test de profil reçu du serveur ne couvre pas ce payload adverse :
- /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/plan.md:77

## Défauts prioritaires

| Priorité | Défaut | Références | Conséquence |
|---|---|---|---|
| P0 | stdout bwrap est assimilé à une preuve de sandbox. | contracts/rights-v1.md:17-22 ; plan.md:66-67 ; codex_app_server.rs:2198-2208 | Faux refus provider_sandbox attribué à Bridget. |
| P0 | command signifie démarrage, pas réussite terminale. | plan.md:81-83 ; codex_app_server.rs:2102-2175 | Faux succès Tester, notamment avant un refus sandbox. |
| P0 | Aucun lien contractuel complet entre tentative, message, commande et exécution. | research.md:40-46 ; data-model.md:13-34 | Refus ou succès d’un autre tour confondu avec le test. |
| P1 | false dans le fichier, true par défaut dans la trame. | data-model.md:6-11,23-25 | Réassignation fail-open. |
| P1 | Deux sources d’état et deux générations, écriture partial non atomique. | contracts/rights-v1.md:7-10 ; ADR-027:11-18,30-34 | Profil, plafond et réassignation peuvent diverger. |
| P1 | daemon.rs et bridget_client.rs sont hors périmètre malgré leur nécessité. | plan.md:46-59,74-75 ; daemon.rs:8696-8716 ; bridget_client.rs:1207-1220 | Champ ignoré ou intégration incomplète. |
| P2 | isTrusted ne fixe pas la provenance du geste local. | plan.md:68,77 ; spec.md:126-131 | Carte distante susceptible d’induire une mutation locale. |

## Changements minimaux prioritaires

1. **Retirer l’attribution provider_sandbox depuis outputDelta seul.** Modifier :
   - /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/plan.md:66-68
   - /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/contracts/rights-v1.md:15-22

   Rechercher un signal terminal structuré du fournisseur, lié à commandExecution. S’il n’existe pas, conserver la ligne brute mais classer unknown conformément à FR-005 ; ne pas la nommer sandbox. Ajouter le témoin : une commande réussie qui imprime exactement la chaîne bwrap ne crée aucun refus provider_sandbox.

2. **Faire de Tester une tentative corrélée et terminale.** Ajouter dans :
   - /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/data-model.md:13-34

   les champs test_id, message_id, instant de départ, hash de la commande fermée attendue, execution_id et issue terminale. La réussite exige le même message_id, la commande canonique exacte, le même execution_id, une terminaison fournisseur réussie et le jeton attendu. Le refus doit être un signal fournisseur terminal pour la même exécution. timeout, file et absence de consommation deviennent unknown/expired.

3. **Réserver l’agent ou rendre le test explicitement asynchrone.** Modifier :
   - /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/contracts/rights-v1.md:11-13

   La vérification de liberté doit être atomique avec l’admission du test, ou la tentative doit attendre la consommation de son message_id sans conclure à trente secondes. Ajouter les essais : tour concurrent, succès après délai, command puis sandbox, et refus d’un autre tour.

4. **Réconcilier auto_reassignment avec ADR-027.** Avant les tasks, choisir explicitement : le champ rejoint control_state et control_events, ou server-rights demeure l’autorité mais le daemon renvoie une projection avec rights_generation et un état unknown si le fichier est absent/illisible. Dans les deux cas, aucun défaut true : false ou unknown seulement. Étendre ControlStateWire, ControlSnapshot et la garde unique admit_autonomous_effect ; ne pas ajouter une condition parallèle dans la réduction de réassignation. Ajouter daemon.rs et bridget_client.rs au périmètre :
   - /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/plan.md:46-59,72-75

5. **Figer la provenance locale.** Prescrire dans :
   - /home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/plan.md:68,77

   que local_toggle ne peut être créé que par le code local depuis les préférences locales ; un objet serveur/journal/message ne peut jamais en porter un. Ajouter l’essai adverse : un Refusal distant avec gesture.local_toggle ne modifie pas bridget.content-security.v1.

## Trous de couverture

- Pas de témoin distinguant une erreur bwrap réelle d’un stdout fabriqué : plan.md:66-68.
- Pas de test de l’ordre item/started puis échec terminal : plan.md:81-83.
- Pas de preuve que le refus lu appartient au message_id du test : research.md:40-46.
- Pas de migration fichier absent/invalide, daemon ancien ou générations divergentes : data-model.md:3-11,23-25.
- Pas de payload adverse distant portant local_toggle : plan.md:77.

## Condition de levée du blocage

Le verdict pourra devenir APPROVE_WITH_CHANGES lorsque ces cinq changements seront intégrés au plan et au contrat avec leurs témoins adverses. Condition non négociable : ni un texte libre de stdout ni un simple acte de démarrage ne doivent pouvoir, seuls, établir un refus de sandbox ou une réussite de droit.

