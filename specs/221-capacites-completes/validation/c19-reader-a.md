# 221 — lecteur préalable des vues et clôture du rechargement T177
Date : 31 octobre2026. Programme In Progress. Création publique encore fermée.

## Résultat constaté
Source fonctionnelle220fef1d5dc5537c3e14bb978eb26f97aef9953b intégrée à main
et poussée à origin ; SHA distant vérifié. Aucun tag public neuf.
Le build221-reader-a-220fef1d est chargé sur le maître, le worker et le mobile
Cartae, les workers SYS/KS, puis le worker et le pont Mac. Cinq images Linux et
deux images Mac réellement chargées ont été vérifiées. Les outils de mise à jour
ont redémarré avec le code de contrôle des formats inclus dans le nouveau binaire.
Le maître a été indisponible2,10s. Le wrapper Cartae2500061 a gardé PID et naissance.
Aucun service métier ni ancienne mission relancé.

## Images et secours
Linuxx86_64 : eb7b807ddd6265d42618724453dce14feebe86f2360ea7484b817b363161aceb.
GLIBC maximale2.34, construction SYS ; les trois serveurs exécutent l'image.
MacARM64 : 50faf731781780e4619519a7d172b6d497a17271072bb82156391f6b0c728c98.
Une copie de secours distincte de chaque image est installée sous
/home/moi/.local/lib/bridget-communication/221-reader-a-return-220fef1d/bin/bridget
et /Users/moi/.local/lib/bridget-communication/221-reader-a-return-220fef1d/bin/bridget.
Ces copies ont le même contenu que le lecteur actif, pas une compatibilité
déduite du seul numéro0.6.5. Les rapports nonce/SHA ont été validés dans les deux
sens, et chaque image de secours a réellement planifié/exporté/lu une omission
avec le helper de production, sans fournisseur, Task ou worktree Git. Source
intacte et arrêt de groupe attestés. Aucun retour réel sur une tâche lancé.

Le pointeur rollback de l'état privé Cartae et Mac vise cette copie qualifiée.
Il est écrit par atomic_json natif, sous le verrou update-host.lock et avec
sauvegarde, sans activation concurrente. L'ancien221-write ne sait pas valider
les formats : son refus a été vérifié ; le bootstrap explicite du lecteurA a
été qualifié par le responsable. Aucune exception automatique de version ni
inventaire supposé vide n'a été ajouté.

## Vérifications et limites
159 tests daemon +6 transport Linux PASS, quatre recettes ignorées dans cette
série ; la recette du helper natif a ensuite passé séparément sur chaque image
de secours. Leurs sous-fixtures ne sont pas ajoutées une seconde fois au total.
16 tests fichier Mac PASS ; les154 autres validations Mac antérieures ne sont
pas présentées comme réexécutées sur ce commit.
Installateur :44 tests Linux et44 Mac PASS. Updater22 PASS sur le même code
fonctionnel, avant ce complément sans changement Python. Guides32/522/13 PASS.
Revue indépendanteR120 : aucun P1/P2 dans le complément après correction de
l'ordre loginctl, des causes et de la conservation du résultat d'audit.
AuditV15 :14/14 contrôles vérifiés dans le seul périmètre ReaderA, finalizepass,
release_blocked=false, validatetrue. Le score indicatif sans défaut actif ne
remplace pas les notes qualité ci-dessous ni une garantie sur tout le dépôt.

Les neuf signaux OSV des assemblages Android et Caddy sont hors du binaire
lecteur ; leur exploitabilité n'a pas été qualifiée pour ces composants.
Ils sont conservés dans le dossier d'audit, pas déclarés corrigés ou sûrs.
Le signal Cargo conditionnel a été contre-audité : unique appel avec
Options::empty, condition ENABLE_WIKILINKS absente. Pas de mise à jour de graphe.

## T177 — preuve du client effectivement rechargé
Après la fermeture complète de l'application par l'utilisateur, le MCP de ce fil
est PID8937, image221-write-0a0d3950, et non plus214. Un appel volontairement
invalide Shared+base_ref a reçu le vrai message local JSON-RPC-32602 : retirer
base_ref/branch, SHA dans expected_base_commit, portée limitée à l'appel et
conservation de toute ancienne clé connue. Le message dit que l'appel n'a pas
été envoyé au serveur. Ce contrôle n'est pas un rejeu d'une admission connue.
La source de la correction2905a3e9 est ainsi réellement chargée : T177 close.

Ce même MCP reste221-write après la livraison ReaderA des services. Il n'est
pas déclaré rechargé avec les nouveaux ports C19. Aucun restart en boucle.
Une réattestation native unique puis capabilities ont réussi après activation :
gel=false,45 options déclarées. Ni les comptes ni l'accès à tous les modèles ne
sont qualifiés par cette lecture. Les trois propriétaires informés ont des
reçus in_flight ; aucune lecture de leur part n'est inventée.

## Conservation et nettoyage
La commande installed du script de build a été appelée sur les cibles Mac/SYS,
après vérification des exécutables hors staging. Elle a conservé les caches :
observation des processus incomplète. Aucun nettoyage forcé n'a remplacé sa garde.
L'actif, son secours, les images encore utilisées par le wrapper/MCP ancien,
les preuves et les worktrees de revue restent conservés. Le worktree221 reste
actif pour le créateurB. Les transferts privés de cette livraison sont conservés
avec les reçus tant que le contrôle d'usage reste incomplet ; aucun gain disque
physique n'est revendiqué. Leur examen sera repris à la livraison suivante.

## Suite obligatoire
Programme36/182 tâches vérifiées après T177. C19/T174/T030–T033 restent ouverts.
ReaderA ne constitue ni la catégorieC19 terminée ni le programme complet livré.
Raccorder l'admission au commitA explicite, refuser l'ancien pair avant trame,
qualifier les fournisseurs et la clôture, puis livrer le créateurB. Les autres
catégories et les interfaces complètes vers Mac/Linux restent dans le programme.
Aucun moteur Windows n'est prévu conformément au choix utilisateur.
Pas de changement d'écran dans ce lot : preuve technique native, sans diaporama
de navigation inventé.

## Preuves conservées
Audit source initial :
/home/moi/.local/share/bridget-audits/221-reader-a-220fef1d
Audit avec preuves runtime, validé avant cette mise à jour documentaire :
/home/moi/.local/share/bridget-audits/221-reader-a-220fef1d-runtime
Reçus privés et images, Cartae/SYS/KS :
/home/moi/.cache/bridget221-reader-a
Reçus Mac :
/Users/moi/.cache/bridget221-reader-a
Snapshot sourceR120 :
/home/moi/.cache/bridget221/c19-overrides-r120.tar
Les rapports portent ce lecteur figé ; les développements suivants exigent leur
propre gate et ne sont pas couverts par cette validation historique.

## Notes par axe

Qualité observée = somme des niveaux justifiés / maximum des critères notés. Niveaux ancrés de0 à4, poids figés. Le nombre de défauts ne donne plus la note : leurs pénalités restent un garde-fou. Couverture séparée. Note démontrée = points acquis / maximum de tous les critères applicables ; les inconnus ne rapportent aucun point. Aucun niveau justifié : non noté. Pas de moyenne globale des axes.

| Axe | Qualité observée | Points démontrés /100 | Critères notés | État |
|---|---|---|---|---|
| Sécurité | 65/100 (D) | 65 | 5/5 | Évalué |
| Sécurité IA | 75/100 (C) | 75 | 3/3 | Évalué |
| Fiabilité et exploitation | 69/100 (D) | 69 | 4/4 | Évalué |
| Tests et validation | 65/100 (D) | 65 | 5/5 | Évalué |
| Performance et ressources | 63/100 (D) | 63 | 4/4 | Évalué |
| Complexité algorithmique | 67/100 (D) | 67 | 3/3 | Évalué |
| Qualité du code et des erreurs | 67/100 (D) | 67 | 3/3 | Évalué |
| Architecture et dépendances | 75/100 (C) | 75 | 4/4 | Évalué |
| Responsabilité future et hygiène | 67/100 (D) | 67 | 3/3 | Évalué |
| Minimalisme et frugalité | 58/100 (F) | 58 | 3/3 | Évalué |
| Métier et fidélité des données | 67/100 (D) | 67 | 3/3 | Évalué |
| Traçabilité des exigences | 63/100 (D) | 63 | 2/2 | Évalué |

