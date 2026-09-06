# Réalisation 091 — communication et observation

## Corrections intégrées

- `spawn --posture development` : attribution humaine par ordre, écriture Codex
  limitée au cwd, pas de réseau ni d'extension automatique ; définition et posture
  figées au rejeu. La posture globale n'est pas modifiée. Le CLI refuse une
  attribution sans entrée et sortie TTY ; un agent ne simule pas ce terminal.
- Les tours gérés Codex portent l'enveloppe complète from/id/reply. La découverte
  des outils MCP précède le recours au shell. La réponse finale gérée est relayée
  par le wrapper, sans double envoi MCP ; en interactif elle reste explicite.
- MCP `bridget_cancel(id, reason?)` réutilise l'autorité du client/daemon existant.
  Aucun outil de spawn/stop/relaunch n'est ajouté. Une annulation de demande
  n'arrête pas le processus destinataire.
- Reprise/fork Codex : cwd, sandbox et politique d'approbation effectifs sont
  réappliqués, également en mode géré ; configuration absente = refus.
- Attach rend les commandes et autorisations natives, neutralise les contrôles,
  garde les extensions inconnues explicites. La saisie et les bornes du journal
  restent celles de la voie existante.
- Statut : ListAgents sur connexion séparée (750 ms, 256 Kio), toutes les deux
  secondes. Métadonnées expirées après six secondes, sans renouvellement fictif
  dans une file retardée. Une panne de cette consultation ne coupe pas le journal.
  Type/client, modèle, effort et état sont attestés ; fournisseur non déduit.
- Skill, aide et README FR/EN distinguent les droits d'écriture, le droit de parler
  depuis attach, et les chemins CLI/MCP.

## Preuves ciblées

- Attach : 57/57 avant le dernier oracle réseau ; les quatre tests `spec091`
  passent ensuite, dont annuaire réel sur socket privée : présence, absence,
  ancienne réponse refusée, dépassement de taille, pair muet borné.
- Relecture indépendante : connexion séparée, vieillissement avant mise en file,
  lisibilité 80 colonnes et absence de double réponse vérifiés.
- Lancement : 42 validations ciblées du lot. Le sandbox Codex natif écrit dans
  cwd mais refuse l'extérieur et une connexion TCP, même face à une configuration
  utilisateur permissive. Le refus non-TTY est exercé par un vrai CLI.
- Enveloppes/reprise : 55 tests app-server et 40 tests MCP du lot. Annulation
  MCP → daemon réel : émetteur, intrus, usurpation de champ, autre instance et
  rejeu du reçu vérifiés dans `mcp_cancel_091_test.rs`.
- Skill : `quick_validate.py skills/bridget` réussi ; les deux liens globaux
  restent dirigés vers le dépôt actif, sans nouvelle installation de skill.

## Environnement de validation consolidée

La première passe utilise les temporaires macOS par défaut : 158 échecs, dominés
par SUN_LEN. Un répertoire court ramène le résultat à 710 réussites, 13 échecs,
8 ignorés pour la bibliothèque daemon. Ces échecs ne sont pas masqués.

Diagnostic : les harnais anciens ne passent pas tous par `initialize_process`,
qui applique umask 077 en production. Certains créaient leurs sockets/états avec
umask 022 ; des fixtures dépendaient aussi d'un HOME global modifié en parallèle.
Huit corrections de harnais, présentes comme défauts à e50c4928, raccourcissent les
racines CLI et créent les namespaces privés dédiés avant leur usage (aucun changement
de règle de sécurité en production).

Commandes consolidées (répertoire de travail de la session 091 ; `/tmp/b91.DYNm`
créé par `mktemp -d /tmp/b91.XXXX`, donc privé et sans donnée de production) :

```sh
umask 077
PATH=/Users/moi/.cargo/bin:$PATH TMPDIR=/tmp/b91.DYNm CARGO_TARGET_DIR=/Users/moi/Nextcloud/10.Scripts/64.bridget/target cargo test --workspace --features test-support --no-fail-fast -- --test-threads=1
PATH=/Users/moi/.cargo/bin:$PATH TMPDIR=/tmp/b91.DYNm CARGO_TARGET_DIR=/Users/moi/Nextcloud/10.Scripts/64.bridget/target cargo test -p bridget-daemon --features test-support --lib -- --test-threads=1
```

La première commande rend 1 217 réussites, les deux défauts restants d'isolation
CLI, et 52 ignorés. Après leur correction, la seconde rejoue **toute** la
bibliothèque daemon : **723/723**, 8 ignorés, 40,68 s. Les autres cibles du
workspace étaient toutes vertes et leur code n'a pas changé : **1 219 réussites
consolidées**, sans compter deux fois les répétitions. Ce bilan n'est pas présenté
comme un unique lancement de la commande workspace sans échec.

Logs bruts : `/tmp/bridget-091-suite-prive-20260906.log` et
`/tmp/bridget-091-lib-final-20260906.log`. Aucun test retiré, ignoré nouvellement
ou assertion affaiblie. Les recettes natives ignorées par défaut restent
distinctes de ce décompte.

Contrôles finaux réussis :

```sh
PATH=/Users/moi/.cargo/bin:$PATH cargo fmt --all --check
PATH=/Users/moi/.cargo/bin:$PATH CARGO_TARGET_DIR=/Users/moi/Nextcloud/10.Scripts/64.bridget/target cargo clippy --workspace --all-targets --features test-support -- -D warnings
PATH=/Users/moi/.cargo/bin:$PATH TMPDIR=/tmp/b91.DYNm CARGO_TARGET_DIR=/Users/moi/Nextcloud/10.Scripts/64.bridget/target cargo test -p bridget-daemon --lib spec_091_native_development -- --ignored
python3 /Users/moi/.codex/skills/.system/skill-creator/scripts/quick_validate.py skills/bridget
```

La recette sandbox supplémentaire réussit 1/1 (0,04 s) ; elle exécute le binaire
Codex installé, sans mission modèle ni usage de la production.

## Recette humaine restant à exécuter

### Complément observé le 2026-09-06 : approbation MCP

L'agent `a4d12c75-5994-4c02-9acc-2db07ba817af` lancé explicitement en
développement écrit et relit effectivement son fichier de preuve dans le cwd.
En revanche, ses appels natifs who/send sont refusés par Codex 0.153.4 :
`MCP tool call requires approval, but approval policy is never`.
Le retour lié reçu par l'émetteur est le relais automatique du wrapper ; il ne
constitue PAS une preuve d'appel MCP sortant réussi.

Cause : la projection `-c mcp_servers.bridget=…` ne déclarait aucune politique
d'outil. Codex `auto` exige une approbation lorsque les annotations ne permettent
pas de l'éviter ; `never` interdit alors cet appel avant d'atteindre Bridget.
Source primaire vérifiée à la version installée :
https://github.com/openai/codex/blob/rust-v0.153.4/codex-rs/core/src/mcp_tool_call.rs
(`requires_mcp_tool_approval_for_mode`, `request_mcp_tool_approval`).

Correction : seules les opérations `bridget_who`, `bridget_send`,
`bridget_ledger`, `bridget_cancel` reçoivent `approval_mode="approve"` dans la
configuration éphémère du serveur Bridget. Les autres outils de ce serveur
restent `prompt` ; aucun réglage des autres serveurs, du shell, des droits de
fichiers, de la posture globale ou de la configuration utilisateur n'est changé.
L'autorité identité/instance du daemon continue de contrôler chaque opération.

Preuves : oracle exact de projection ; lecture de cette même projection par le
vrai `codex app-server` installé (`config/read`, aucun appel modèle, HOME Codex
temporaire). La mutation `bridget_who.approval_mode="auto"` échoue effectivement
sur l'oracle. La re-recette de l'agent vivant reste nécessaire après installation
et reprise explicite de son processus pour charger cette nouvelle projection.

Le test sandbox natif n'est pas une preuve d'une nouvelle mission intellectuelle
en production. La recette de bout en bout exige le lancement depuis le terminal
humain d'un NOUVEL agent développement, puis mission écrite dans cwd, outil MCP
réel, et journal observé dans attach. Ne pas utiliser `relaunch` pour élargir les
droits de l'ancien agent découverte ; ne pas modifier la posture globale.

Les sessions MCP déjà ouvertes peuvent garder leur ancien catalogue jusqu'à
leur reprise. Ne pas annoncer `bridget_cancel` visible dans une session ancienne
avant d'avoir constaté son catalogue. Aucun redémarrage de production n'est requis
par les tests isolés.

## Vérification installée et compléments observés

Installation initiale de 99b952dd : `who` confirme le build daemon
`99b952dd8ac9`, le profil global reste `discovery`, génération 0. Le redémarrage
SIGTERM du service launchd conserve/reprend l'agent de recette existant.
L'ouverture réelle d'attach en terminal sur d61be10c-db0c-41a3-9f79-47e7af73734b
affiche Codex, gpt-5.6-terra, medium, connected et les commandes historiques.
Ctrl-C ferme uniquement cette vue. Le MCP installé expose `bridget_cancel`
via une négociation/listage stdio sous namespace temporaire isolé.

Deux constats de cette recette ont été corrigés avant livraison finale :

- Le journal natif porte aussi `reasoning { available:false }` : c'est une absence
  déclarée, pas un format inconnu. Le renderer affiche cette absence, ou le résumé
  déclaré s'il existe ; il ne projette jamais le champ brut `raw`. Fixture réelle
  ajoutée et absence/résumé/manquant testés.
- Le test historique `le_domaine_surcharge_prime_sur_le_derive` utilisait le vrai
  HOME et y créait `agent-domains` (0755 sous umask 022). Au redémarrage, la garde
  privée refusait légitimement ce répertoire vide. Type, absence de symlink,
  propriétaire 501 et vacuité vérifiés ; droits remis à 0700, aucune donnée
  supprimée. Le test injecte désormais son fichier privé dans le même lecteur
  métier, sans passer par le namespace réel ni muter l'environnement global.

Après ces compléments : attach **58/58**, wrapper **59/59**, exécutés en parallèle
à l'intérieur de chaque cible ; `cargo clippy --workspace --all-targets --features
test-support -- -D warnings`, fmt et diff-check réussis. Pas de répétition inutile
des recettes lourdes des autres composants non modifiés.

## Pied de page sous la saisie et fermeture de recette MCP

À la demande de l'humain, l'équipier de développement a déplacé le statut sous
l'invite et ajouté les oracles d'ordre, de retour du curseur, de première présence
et de sortie propre. Le relecteur a exécuté les validations hors de son sandbox :
**62/62 attach**, **60/60 wrapper** (et un test natif ignoré par défaut exécuté
explicitement : **1/1**). Fmt, clippy workspace/all-targets/test-support et
diff-check sont verts. La preuve d'écriture de l'équipier reste non versionnée.
Le statut de la recette MCP de l'agent vivant est consigné séparément : un
succès du parseur natif ne vaut pas encore un appel métier réussi.

La reprise réelle à 786e6372 révèle une seconde couture : les arguments -c
globaux injectés avant `app-server` disparaissent de la configuration effective
quand le profil possède aussi des -c après la sous-commande. Vérifié en relisant
les arguments du processus puis `config/read` : les quatre autorisations sont
absentes malgré leur présence dans argv. L'assemblage réunit désormais les
overrides après la sous-commande en préservant leurs octets et priorité, puis
ajoute la projection MCP. Cela conserve également modèle/effort et le bypass
explicitement demandé par un humain, sans en créer un. Le test natif consomme
maintenant les arguments du VRAI registre development passés au VRAI assembleur
wrapper ; les arguments anciens observés en production font échouer ce test.

Installation de b42aeb48 et reprise du SEUL équipier de recette (génération 443) :
who MCP natif réussit ; send MCP natif enregistre `RECETTE-091-MCP-OK` sous
`mcp-66627-6a9d386f-1`, livraison `3d6faeac-164d-470c-9519-260d43bd34b1`.
La demande de recette `mcp-30942-6a9d3867-7` est answered. Le message MCP distinct
reste en_vol au dernier relevé, le destinataire étant occupé : ne pas assimiler
son dépôt à un accusé final ni le réémettre sous une nouvelle clé. L'écriture
effective dans le cwd a été constatée avant reprise ; les droits sont conservés.

Validation finale du wrapper : **62/62**, test Codex natif inclus, en parallèle.
La recette TUI native 090 `--initial-resume` passe aussi sur le binaire installé :
même fil, historique et nom, --yolo demandé attesté (modèle HTTP synthétique,
aucune consommation de forfait). Log : `/tmp/bridget-091-reprise-native-final.log`.
Le daemon n'a pas été redémarré pour ces changements de CLI/wrapper ; seule la
nouvelle vue attach et l'équipier de recette chargent le nouveau binaire.

## Référence visuelle et sélection native à chaud — 2026-09-06

Le rendu reprend la référence demandée : saisie à fond gris 256 couleurs sur
toute la largeur, multiligne (Alt+Entrée ou Échap puis Entrée), statut coloré
SOUS la saisie. La fenêtre visible est bornée sans tronquer le tampon ; le curseur
revient après le texte et non après le padding. Les sorties sans TTY restent
inchangées, NO_COLOR/TERM=dumb retirent les SGR, pas les commandes de curseur.

Recette visuelle : vrai binaire attach connecté à l'agent vivant
a4d12c75-5994-4c02-9acc-2db07ba817af dans un pseudo-terminal 120×18 ; deux lignes
saisies, aucun message envoyé. Le flux ANSI réel a été interprété par pyte 0.8.2,
puis inspecté en image. Fond 303030 attesté sur les 120 cellules des deux lignes,
statut juste dessous, couleurs distinctes, coordonnées du curseur exactes.
Ce n'est pas une capture d'iTerm : son automatisation est indisponible dans cet
environnement. Capture de recette : `/tmp/b91-model.DzNs/attach-091-visuel.png`.

Le contrôle `/model <modèle> <effort>` ne devient jamais un Send. Il exige une
connexion Attach déjà abonnée à la cible, possède un reçu corrélé à la connexion
wrapper réelle, et refuse un reçu forgé par un autre wrapper. Le pilote interroge
au plus quatre pages model/list (100 modèles/page), valide les deux valeurs sans
substitution puis utilise thread/settings/update du Codex 0.153.4 installé.
La négociation experimentalApi expose cette méthode, sans changer les permissions.
L'échéance de dialogue fournisseur est 2 s, celle de réponse daemon 4 s ; toute
issue ambiguë reste inconnue. Un fournisseur muet ne laisse pas de waiters résiduels.
Le réglage est détenu par le fil natif, jamais dupliqué dans Bridget ni persisté
dans la définition de lancement. Pas de nouveau fil, pas de message caché.

La recette versionnée `runtime_selection_integration_test` utilise vrai daemon,
vrai wrapper et vrai Codex avec HOME privé et fournisseur HTTP synthétique :
sélection pendant le premier tour arrêté à une barrière HTTP, premier tour
inchangé, deuxième tour avec nouveau modèle ET effort, sentinelle du premier
tour présente dans le second, un seul rollout. Les modèles viennent du catalogue
installé (Astra puis Sol dans cette recette), pas d'un nom inventé par l'oracle.
Les deux refus modèle/effort ne modifient rien ; exactement deux requêtes HTTP.
La recette ne consomme aucun abonnement et ne contacte pas le fournisseur réel.

Validations (PATH=/Users/moi/.cargo/bin, TMPDIR privé court /tmp/b91.1cRe,
umask 077, target partagé du dépôt actif) :

- `cargo test --workspace` : 1 203 succès, zéro échec, 47 ignorés, avant les deux
  derniers oracles de fermeture/expiration. Les cibles finales touchées ont été
  rejouées après eux, sans relancer les bancs lourds étrangers à leur diff.
- `cargo test -p bridget-transport` final : 263 unitaires verts, puis intégrations
  et doctests verts ; le reçu outcome_unknown refuse aussi les champs inconnus.
- `cargo test -p bridget-daemon --lib attach::tests` : 67/67, parallèle, 0,88 s.
- `cargo test -p bridget-daemon --lib spec091_selection_autorisee` : 1/1.
- `cargo clippy --workspace --all-targets --features test-support -- -D warnings` : vert.
- `cargo test -p bridget-daemon --test runtime_selection_integration_test -- --ignored --nocapture` : 1/1, 2,03 s.
- `cargo fmt --all --check` et `git diff --check` : verts.

Installation et reprise de production restent à constater pour T10, pas déduites
des recettes privées. Le fichier de preuve d'écriture antérieur reste hors commit.
Les deux répertoires temporaires de compilation de l'équipier ont été déplacés,
sans suppression, sous `/tmp/b91-model.DzNs/agent-build-Gd83Re` et
`/tmp/b91-model.DzNs/agent-build-XdMxR8` pour conserver le worktree propre.
