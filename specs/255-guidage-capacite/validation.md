# Validation 255 — livrée en 0.8.22

9 novembre 2026. Source exécutable :
0fc30045a675b21e53f14b5333bb522d2cd4cbba.
Source documentaire de revue : 8ed67c5e570ce9ba6f9c621bbccb6c869e382dcc.
Build actif : 255-capacity-guidance-r1.

## Incident et correction

Parix signale la demande parix-entretien-exports-mac-20261109-01 à1794226227 :
resource_saturated, machine monordinateur imposée, admission not_recorded,
task_id null. Les mesures jointes sont CPU61%, mémoire disponible46%,
charge normalisée100%, seuils90/10/100. Aucun rejeu métier n’est demandé.

Cause confirmée par le code : failure(resource_saturated) conseillait une
nouvelle clé en présumant un refus durable, avant que routing_failure_admission
n’ajoute son état réellement observé. Le reçu pouvait donc donner deux conseils
contradictoires. Le conseil global renvoie désormais à detail.admission.recovery.
Il ne présume plus l’état durable. Le texte exige une capacité fraîche sur la
même machine et conserve les contraintes.

La table d’admission reste inchangée : not_recorded conserve la même clé et
enveloppe après levée de la cause et fin des appels concurrents ; pending garde
son admission et sa clé ; admitted suit sa tâche ; refused durable reste clos,
puis une suite autorisée emploie une nouvelle clé ; unknown ne permet aucun
remplacement. Les mesures, seuils, règles d’admission, droits, reçus et tâches
ne sont pas modifiés par le correctif. Le cache d’observation déjà existant
continue son écriture et sa rétention habituelles.

## Contrôles et revue

Cinq contrôles distincts réussis sur Mac et Linux, sans additionner les deux
plateformes : le nouveau spec255 et les quatre spec189_capacity. Le nouveau
test projette des reçus absents, pending puis refused. Il vérifie le conseil
adapté, le reçu inchangé, zéro tâche créée et aucune remise au worker.
Les tests189 gardent les preuves fermées, les refus durables et leur rejeu.

Premier passage Mac bloqué par le chemin temporaire trop long pour une socket
Unix : le lanceur privé utilisait le TMPDIR système long. Après utilisation
de /private/tmp, le test passe. Aucun code produit changé pour ce problème de
banc. La copie Android a d’abord été demandée avant création du bundle ; aucun
fichier transféré à cette tentative. Le bundle terminé est ensuite vérifié
avant copie. Le transfert Linux est complet et les trois SHA sont comparés
avant installation ; aucun binaire partiel installé.

Builds Mac/Linux via scripts/build.py, cibles partagées et entretien automatique.
Build Android8022 réussi, certificat et lignée8021 vérifiés. Aucun changement
d’interface Android : pas de diaporama nouveau nécessaire.

Revue indépendante native 76e84329-1396-448e-bb9a-9e53b753ad2c, ClaudeHaikuSYS :
résultat lu et acquitté, aucunP1. P2 documentaire « lecture pure » corrigé en
« sans mutation des reçus ni tâches » ; le cache préexistait en254. P3 sur
les états non ciblés et le texte scoped disposés dans adversarial-review.md.
Aucune exécution indépendante de tests n’est inventée.
Analyze PASS, Converge CONVERGED. Aucun helper, champ, table, format ou service ajouté.

## Activation réelle

Sept composants chargés sur quatre hôtes : core/mobile/worker Cartae, workers
SYS/KS, worker/connecteur Mac. Images chargées vérifiées par empreinte.
Les outils de mise à jour sont relancés nativement après écriture atomique du
pointeur de retour sous verrou, sans activation concurrente.

- Mac SHA234de162b2813e90253bb342905ae19248ae1b6b00ba929cf663e9a9730b6ddf.
- Linux SHAce0c8ee3df8a5d6fb85be9d3d160cd3fdcc24da5215886cf9305e02a3745a581.
- Retour254R3, source695921e76ed3a4611a2a4b4dfbf0411ae2f6b6ae : compatibilités
  forward/return/inverse et SHA vérifiés. Ancien lecteur245 refusé.
- Versions des clients installés contrôlées sans authentification ni modèle.
- Coupure maître1,85s. Un wrapperMac, deuxCartae et unSYS aux passages ont leurs
  PID/birth conservés. WrapperSYS3016909/birth138868690 préservé.
- Onze processusT3 et leurs dates de démarrage, septpayloads protégés et registre
  Mac60acca93483698c25d5b1aca4b52af20cd3ce24889b6ecf21ade87e2367daebe inchangés.
- Session nativeV2 réussie ; parentParix busy avec projet attesté après passage.

T3 n’a pas été arrêté ni remplacé. Les anciennes façades MCP et les wrappers
de modèles encore vivants sont conservés. Le guidage est calculé par le serveur
actif ; aucune activation globale de ces anciens processus n’est annoncée.

## Publication, Git, diagnostic et nettoyage

Release publique v0.8.22 : six actifs téléchargés anonymement, SHA conformes.
APK8022 signé, 3967548octets,
SHA91215812ee220a02cf81578504e2e116f039c7dda77bf5817a3d9a30bcd75b3e.
Source intégrée et poussée ; main final propre et aligné sur les quatre hôtes.

Parix prévenu avant la coupure et informé du correctif actif. Reçu final
mcp-56788-6af1bc05-10, remise1028d1ab-0d3c-481b-95f5-b7b79081b4ea,
initialementin_flight. Aucune lecture métier affirmée. Aucun broadcast.
La demande d’entretien n’a pas été rejouée ; son dossier retiré doit aussi être
pris en compte avant toute suite. Aucune tâche de qualification créée.

Propre worktreeMac et branche locale/distante retirés après intégration,
contrôle des ignorés et absence d’usage. Treize copies de transfert et
téléchargements de vérification retirés : 307154776octets logiques.
WorktreesbuildSYS/AndroidCartae intégrés et propres conservés : cinq processus
SYS et quaranteCartae illisibles au contrôle des usages. Aucun processus étranger
arrêté pour le ménage. Vue native de revue, reçus, sauvegardes, preuves, actifR1,
retour254R3 et anciennes images utilisées ou sous observation incomplète conservés.
Derniers caches reconnus : Mac2,97Gio, SYS6,22Gio, Cartae4,46Gio, KS0Gio.

Preuves et recettes privées :
/Users/moi/.cache/bridget255-guidage-capacite/.
