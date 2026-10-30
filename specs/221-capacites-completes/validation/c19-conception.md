# C19 — proposition indépendante à éprouver
30 octobre 2026. ClaudeSonnetSYS Task70c5a3fd-2b81-403d-9481-ccf052d438e6, lecture seule ; résultat intégral lu/ACK et clôture demandée. Proposition brute privée : /Users/moi/.cache/bridget221/c19-design-proposal.md. Aucune implémentation ou qualification d'accès fournisseur déduite.

Constats retenus : Shared peut demander un rafraîchissement de HEAD et refuse un descendant actif ; créer un worktreeGit n'est pas le résultat demandé. Les parties de sélection du profil compact peuvent être réutilisées, pas son audit d'index comme attestation absolue d'un dépôt enfant modifiable.
Option proposée, non adoptée : dépôt indépendant avec objets partagés et HEAD détaché. Elle exige une ancre contre le GC et modifie la racine par cette ref ; ce n'est donc pas un simple export sans métadonnées. L'accès aux objets par SHA dépasse la seule arborescence A ; ses droits doivent être réellement testés, pas supposés. Un .git contrôlé par l'enfant ne doit jamais servir seul de preuve d'intégrité.
Avant contrat : prototype privé, contrôle des liens, bornes exactes, GC, accès Git nécessaires et droits réels. Nouvelle stratégie et capacité imposent de vérifier les reçus à deny_unknown_fields et le retour vers un ancien lecteur même pour des tâches terminées conservées ; absence de tâche active ne suffit pas.
T174 puis T030/T031/T032 restent ouvertes. Aucun fallbackWorktree ni service/proxy ajouté ; interfaces complètes et programme global toujours à réaliser.

## Prototype privé après la reprise du programme

Source de travail inchangée : 37a11ed39ca701af7a5c4afc8e4537458cafd0b4.
Script : /home/moi/.cache/bridget221/c19-prototype/prototype.py.
Résultat : /home/moi/.cache/bridget221/c19-prototype/prototype-result.json.
Le script et le résultat restent hors Git. Un programme de prototype a exécuté
29 assertions sur des dépôts temporaires possédés. Ce ne sont pas 29 tests
de production et ce n'est pas une qualification du fournisseur.

Résultat observé : export de 200895 octets, 200000 octets explicitement omis.
Deux exports de la même base ont le même manifeste et les mêmes octets.
La source reste sur B, avec une modification non commitée, un fichier non suivi
et un worktree Git descendant déjà inscrit. HEAD, index, références, types,
modes et octets de la source sont comparés avant/après. Aucun processus ou
garde de mission réel n'est simulé par cette seule inscription.

Les lectures utilisent ls-tree puis un seul cat-file --batch. Les entrées
binaires gardent leurs octets ; export-ignore et export-subst ne changent
aucun fichier. La sélection reproduit les règles de compact207 dans ce
prototype seulement ; elle ne réutilise pas encore son code Rust.
Les fichiers de code volumineux restent sélectionnés, le gros binaire est
omis et nommé. Les liens internes et internes pendants restent des liens.
Les liens sortants, chaînes sortantes, cycles, collisions de casse exactes,
chemins traversants, .git et noms non UTF-8 sont refusés.

Source absente, objet absent, objet non commit, accès réellement refusé
par les droits du répertoire d'objets, budgets d'entrées/octet et destination
existante sont éprouvés. Leur distinction diagnostique n'est pas encore
implémentée : le prototype regroupe certaines erreurs Git sous un refus
générique. Aucun fallback worktree ni publication de vue sur lien sortant.
Une modification de fichier est détectée par l'inventaire privé. Les fichiers
sélectionnés restent lisibles sans accès à la source. Le Git CLI ne fonctionne
pas depuis la vue : cette limite est testée et doit être traitée dans le
contrat de lecture, pas cachée par un dépôt Git fabriqué.

Limites bloquant la clôture C19 : politique fournisseur réellement readOnly,
corrélation et reprise durable après coupure, contrôle du lecteur minimal
avant retour ancien, lecture native des objets omis, budgets à
l'échelle du projet, portabilité macOS et concurrence hostile du système
de fichiers. chmod0444 ne prouve pas une interdiction d'écriture sous le même UID.
Le manifeste doit rester hors de la vue de l'enfant. Les 29 assertions ne
font cocher ni T174 ni T030/T031/T032. T177 reste également ouverte.

Contre-revue de contrat demandée par Bridget, clé
bridget221-c19-contract-review-20261030-01, tâche
1078ab03-86c7-4905-b51b-bcda44cb31e8, enfant
d2318a93-0ea9-409b-8d63-4287016a6a31, Claude Sonnet.
L'admission a rendu starting sans erreur ; le résultat a ensuite été lu comme indiqué ci-dessous.


## Traitement de la contre-revue de contrat

Résultat intégral de 5927 octets lu, ACK effectué et clôture native demandée.
Verdict APPROVE_WITH_CHANGES. Méthode limitée : spec et conception lues,
symboles localisés ; ni research D1 ni types protocole/record relus, aucun
test lancé. Ce verdict ne qualifie ni un diff d'implémentation ni une activation.
Le compte rendu intégral reste privé :
/home/moi/.cache/bridget221/c19-contract-review.md.

| Observation | Vérification et décision |
| --- | --- |
| Carte d'omissions à rendre visible au modèle | Retenue. Résumé borné dans le contexte et carte lisible ; copie publique distincte de l'ancre privée, collision de nom refusée. |
| Ancienne racine conservée comme autorité | Risque conditionnel retenu : remplacement explicite du mapping, pas ajout de racine. La preuve négative d'accès à tout autre chemin dépasse les politiques readOnly existantes ; aucune isolation générale de lecture n'est promise. Lecture native_permissions.rs : la projection traduit writableRoots et additional_directories, pas une liste globale des lectures système. |
| recordNone sans issue de reprise | Retenue : chemin déterministe, adoption seulement avec preuve privée complète et audit ; sinon conservation et refus guidé, jamais suppression/recréation. |
| Concurrence Shared à préciser | Retenue : pas de shared_refresh ni garde Shared pour la vue ; lecture par SHA, aucune garde étrangère retirée. Le test avec vraie commande active reste requis. |
| Rendre les omissions définitivement inaccessibles | Non retenue : cela laisserait un manque de lecture. Prévoir une lecture native du seul blob du commit, sans écrasement. L'absence future d'objet reste explicitement refusée ; aucune référence GC cachée. |
| Ancien lecteur et retour | Retenue comme gate avant activation : formats conservés, même terminaux, vérifiés avant Prepare. L'absence de mission active n'est pas une preuve de compatibilité. |

Le contrat correspondant est dans le fichier commun de cette spécification,
section C19. Les points retenus sont des exigences avant intégration, pas des
défauts de production déclarés corrigés. Les cinq assertions ajoutées après
le premier appel de revue (29 au lieu de 24) n'ont pas été relues par cet agent.
