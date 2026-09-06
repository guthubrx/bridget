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

Le test sandbox natif n'est pas une preuve d'une nouvelle mission intellectuelle
en production. La recette de bout en bout exige le lancement depuis le terminal
humain d'un NOUVEL agent développement, puis mission écrite dans cwd, outil MCP
réel, et journal observé dans attach. Ne pas utiliser `relaunch` pour élargir les
droits de l'ancien agent découverte ; ne pas modifier la posture globale.

Les sessions MCP déjà ouvertes peuvent garder leur ancien catalogue jusqu'à
leur reprise. Ne pas annoncer `bridget_cancel` visible dans une session ancienne
avant d'avoir constaté son catalogue. Aucun redémarrage de production n'est requis
par les tests isolés.
