# Journal d'implémentation 133

Statut : Completed — 12/12.
Date : 2026-10-04.
Worktree : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/133-relais-sous-agents`.

## T001 — Baseline ciblée

État privé : `BRIDGET_HOME=/tmp/bg133-baseline-home`,
`BRIDGET_SOCKET=/tmp/bg133-baseline-home/b.sock`. Aucun daemon installé, état
T3 réel ou fournisseur payant n'a été utilisé.

Résultats avant modification du code :

- compilation `bridget-core` ciblée : PASS ;
- compilation des tests `bridget-daemon --lib` : PASS ;
- `mcp_identity::tests::` : 10 PASS ;
- `t3code_identity::spec101_identity::` : 19 PASS ;
- `message::tests::` : 12 PASS ;
- `communication::tests::` : 4 PASS ;
- enveloppe SPEC-131 T3 : 1 PASS.

Limite : le filtre `prompt_for` de `bridget-transport` n'a sélectionné aucun
test existant. Les nouveaux tests SPEC-133 devront donc nommer explicitement ce
comportement.

### Self-review Article XIX/XX

- Pourquoi cette solution est nécessaire : figer les garanties avant changement.
- Solution la plus simple : tests existants ciblés, sans nouveau harnais.
- Hypothèses : les suites filtrées représentent les frontières modifiées.
- Vérifications : compilations et 46 tests ciblés verts.
- Non vérifié : suite workspace complète, réservée à T012.
- Code supprimé ou évité : aucun script de baseline ajouté.
- Complexité ajoutée : documentation seulement.

## Analyse SpecKit

Analyse lecture seule : 18 exigences et critères, 12 tâches, couverture 100 %.
Aucune ambiguïté, duplication, tâche orpheline ou violation constitutionnelle.

## T002–T003 — Provenance et rejeu

Les tests ont d'abord échoué à la compilation : `DelegatedOrigin` et le champ
`delegated_origin` n'existaient pas. Après ajout, les deux tests SPEC-133 passent.
Le sous-objet refuse les champs inconnus. Un message historique se réencode sans
provenance. La provenance ne modifie pas les bytes de rejeu du parent.

### Self-review Article XIX/XX

- Pourquoi cette solution est nécessaire : rendre l'origine enfant visible sans créer d'identité.
- Solution la plus simple : un sous-objet optionnel dans l'enveloppe existante.
- Hypothèses : fournisseur et référence sont validés avant construction.
- Vérifications : test RED observé, puis 2 tests ciblés verts.
- Non vérifié : rendu fournisseur et validation de la preuve, tâches suivantes.
- Code supprimé ou évité : aucun nouveau canon, identifiant ou service.
- Complexité ajoutée : deux chaînes optionnelles, schéma fermé.

## T004–T007 — Relais MCP et provenance visible

Le pont T3 publie une preuve privée séparée pour chaque fournisseur interne
rattaché sans ambiguïté à un parent vivant. La résolution MCP rend l'identité du
parent avec une provenance enfant fermée. `bridget_who` et `bridget_send` sont
les deux seules actions admises. Le routage, l'idempotence et les réponses
restent dans la portée du parent. Les rendus ACP, Codex et T3 affichent la
mention « via sous-agent » sans identifiant natif complet.

Les tests couvrent le rattachement nominal, la réponse vers le parent, le rendu
des trois transports et la compatibilité des anciens messages.

## T008–T009 — Autorité minimale

Le répartiteur MCP refuse tout outil autre que `bridget_who` et `bridget_send`
avant l'appel du handler. La ligne de commande applique une garde centrale avant
les commandes sensibles. Un contexte enfant ne peut donc pas obtenir les droits
du parent par une autre façade.

La matrice teste l'inventaire MCP complet. Le garde-fou CLI inventorie les
commandes sensibles afin qu'une future commande ne puisse pas éviter le refus.

## T010–T011 — Preuves fermées et incident réel T3

Les preuves enfant sont privées, bornées, non symboliques, liées au PID et à sa
naissance, puis retirées seulement par leur propriétaire. Une preuve enfant
présente mais invalide arrête la résolution. Elle ne déclenche jamais un repli
vers un marqueur principal plus haut.

L'observation de production a aussi trouvé la cause du blocage global T3. Le
processus `lsof` pouvait produire plus de données que le tube de sortie. Le pont
attendait sa fin avant de vider ce tube. `lsof` restait alors bloqué en écriture,
le délai expirait et le pont publiait un inventaire indisponible pour tous les
fils T3. La correction vide maintenant la sortie dans un lecteur dédié pendant
l'exécution, conserve la borne de taille et termine proprement le processus en
cas d'erreur.

Preuves ciblées :

- 11 tests `spec133_` du daemon : PASS ;
- 2 tests `spec133_` du transport : PASS ;
- 1 test `spec133_` du cœur : PASS ;
- inventaire `lsof` avec 192 fichiers de session : PASS en moins de trois secondes ;
- marqueur invalide, lien symbolique, dépassement, PID recyclé et propriété : PASS ;
- matrice MCP et garde-fou CLI : PASS.

### Self-review Article XIX/XX

- Nécessité : l'incident réel empêchait toute identité T3, puis le sous-agent
  restait sans autorité bornée même avec une identité parent valide.
- Solution la plus simple : réutiliser la filiation et les marqueurs existants,
  ajouter une preuve enfant séparée, puis vider le tube `lsof` en parallèle.
- Hypothèses : le compte local et le pont T3 restent dans le modèle coopératif
  défini par la spécification.
- Vérifications : tests RED/GREEN ciblés, reproduction du volume `lsof` et
  comparaison du premier échec workspace avec `main`.
- Non vérifié à ce stade : redémarrage des services installés et essai réel du
  fil « Véhicules », réservés à la livraison.
- Code évité : aucune identité enfant durable, table, dépendance ou daemon ajouté.
- Complexité ajoutée : un marqueur éphémère, un contexte MCP fermé et un lecteur
  borné pour le tube de l'inventaire.

## T012 — Validation finale et convergence

- `cargo fmt --all -- --check` : PASS ;
- `cargo clippy --offline --locked --workspace --all-targets -- -D warnings` : PASS ;
- `cargo build --offline --locked --release` : PASS ;
- daemon en série : 1 000 PASS, 10 ignorés ;
- transport final : 293 PASS, 1 ignoré ;
- cœur : 41 PASS ;
- tests SPEC-133 : 14 PASS ;
- `git diff --check` : PASS.

La première passe workspace a détecté deux régressions dans l'en-tête historique
Codex. Elles ont été corrigées. Le transport complet est vert après correction.

Deux tests historiques de contrôle CLI restent sensibles à l'identité réelle du
processus qui lance le banc. Ils attendent une connexion CLI nue, mais utilisent
l'identité T3 réelle lorsqu'elle est publiée. Le même échec
`auxiliary_credential_required` est reproduit sur `main` sans la SPEC-133. Cette
limite du harnais n'est donc pas attribuée au changement. Elle ne touche ni le
chemin MCP délégué ni le pont T3 de production.

La convergence finale couvre FR-13301 à FR-13313 et SC-13301 à SC-13307. La
contre-revue locale finale est `adversarial-review-final.md`. Aucun défaut ouvert
de sévérité haute ou critique n'est retenu.
