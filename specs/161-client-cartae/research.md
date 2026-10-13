# Recherche SPEC-161

**Date :** 2026-10-13. Périmètre : script de déploiement, son harness089, documentation de fédération et preuves de préflight transmises par ROOT. Pas d'inférence modèle, mutation distante ou build exécuté par cette recherche.

## Références et méthode

Lecture directe de scripts/deploy-remote.sh:1–93, scripts/tests/federation_089_test.sh:1–170 et docs/federation-services.md. Constitution utilisateur et ponts projet lus. Baseline : /Users/moi/.speckit/research/09-devops-infrastructure.md. Les principes de livraison reproductible et de traçabilité de l'outil/source s'appuient sur [Google SRE, Release Engineering](https://sre.google/sre-book/release-engineering/). Les contrôles post-déploiement et le réemploi d'une procédure automatisée s'appuient sur [DORA, Deployment automation](https://dora.dev/capabilities/deployment-automation/). Ces deux sources primaires ont été consultées par ROOT dans ce tour ; elles ne prouvent pas le runtime Cartae.

## Décision 1 — Client seul

**Décision :** garder le maître Mac et la fédération existante.
**Preuve :** docs/federation-services.md distingue explicitement socket cliente distante et exécution spawn locale au maître. Le préflight ROOT observe une réponse du maître160 depuis le client096 sur Cartae.
**Alternative écartée :** installer un daemon Cartae indépendant ou migrer le maître, hors161.
**Charge future :** aucune seconde autorité ni migration de base à maintenir.

## Décision 2 — Réemployer l'outil existant

**Décision :** compléter deploy-remote.sh, ne pas créer un autre système de déploiement.
**Preuve :** lignes55–68 préflight/réservation ; lignes74–77 transfert des fichiers suivis ; lignes79–92 compilation et installation cliente. Les limites jobs/nice et le manifest pré-build ne sont pas présents aujourd'hui.
**Alternative écartée :** script d'installation historique daemon, conteneur ou nouvel orchestrateur.
**Charge future :** un seul contrat sourceable et un seul harness089. Aucun helper sans règle justifiée.

## Décision 3 — Séparer provenance source et runner

**Décision :** checkout source propre e6812e3e7b2d051d630192857cc3f440180d51fa, runner161 externe, manifest SHA des octets transférés vérifié avant compilation.
**Preuve :** le script calcule le Build ID depuis HEAD puis ajoute -dirty selon statut des fichiers suivis ; il transfère les fichiers suivis du répertoire source, y compris leurs changements. Exécuter le runner modifié depuis le même checkout compilé serait donc incorrect.
**Alternative écartée :** forcer BRIDGET_BUILD_ID160 sur le checkout161. Une identité déclarative seule n'atteste pas les octets.
**Charge future :** reçu avec source_commit, runner_commit/sha et documentation_commit distincts, testables sans contexte caché.

## Décision 4 — Toolchain et charge bornées

**Décision :** Rust1.92 explicite via son cargo de toolchain, PATH aligné sur rustc ; nice10 et deux jobs.
**Preuves ROOT fraîches ~08:07 :** outils cargo/rustc1.92 présents, /usr/bin/rsync, cc et nice présents, espace265G. Parent /home/moi/.local/lib/bridget-communication mode700. /home/moi/.local/bin mode775, à préserver. Cache complet non attesté.
**Alternative écartée :** cargo générique dépendant du défaut1.98 ou installation automatique de dépendances.
**Charge future :** aucune mutation du défaut rustup et panne offline clairement attribuable.

## Décision 5 — Bascule et retour arrière

**Décision :** nouveau préfixe privé, lien voisin puis rename atomique, comparaison de la cible initiale avant mutation ; rollback conditionnel vers096 conservé.
**Preuve :** client actuel /home/moi/.local/bin/bridget → /home/moi/.local/lib/bridget-communication/096/bridget. Le script de préparation ne touche pas ce lien ; la bascule est donc une étape de livraison distincte.
**Alternative écartée :** écraser l'ancien exécutable ou remplacer le lien avant recette du candidat.
**Charge future :** récupération explicable en une bascule, pas réinstallation globale.

## Décision 6 — Contrôles proportionnés

**Décision :** harness089, recette réseau095, observations de processus et hashes. Aucun FULL Rust car aucune source Rust n'est changée.
**Preuve :** le harness089 exerce le programme Bash distant via doubleSSH, dont pipefail et quoting ; la recette095 utilise deux clients synthétiques sans modèles.
**Limites :** succès des mocks ≠ accès SSH réel ; succès status ≠ échange bidirectionnel ; build Linux ≠ suite Rust entièrement verte. Ni secrets ni conversations réelles dans les preuves.

## Inconnues restant opérationnelles

Cache offline complet, temps réel de compilation, résultat des recettes et disponibilité du préfixe neuf sont à constater pendant l'implémentation. Les conduites d'échec sont spécifiées ; aucune décision architecturale suspendue. Les paramètres des serveurs de décision par machine et du Lineage T3 appartiennent à162/163 et ne bloquent pas161.

## Compléments de préflight ROOT et audit de réemploi

Lecture distante fraîche ROOT : status RC0, maître e6812e3e7b2d ; le vieux client096 émet un avertissement trompeur qui accuse le daemon. Ne pas suivre sa suggestion de relance. PID maître20937, pont21097, T3 60116/60228 inchangés au relevé. Python3.12.3 et /usr/bin/lsof disponibles sur Cartae. L'audit de réemploi séparé recherche la réutilisation de SOURCE-MANIFEST.sha256 de package089 et du nettoyage existant build.py. Ces constats guident la future implémentation ; ce document ne remplace pas reuse-audit.md ni son gate.

## Précisions après contre-revue

Le réemploi du manifest089 porte sur son format SHA, pas sur son énumération find :161 exige la liste Git filtrée identique au transfert et la comparaison exacte côté distant. L'option --expect-commit étend le déployeur existant, elle ne change pas son mode historique ; la recette161 exige explicitement la garde stricte. Le nettoyage build.py sait traiter un source sans .git (fallback repo), donc peut être exécuté sur Cartae avec Python/lsof présents ; aucune garantie de suppression de tout source/ n'est déduite. Le contrôle du lien suivi d'os.replace n'est pas un CAS face à un tiers non coopérant ; on refuse les divergences observées et impose un propriétaire unique de l'opération. Ces limites sont conservées dans le contrat.

## Approvisionnement borné du cache — décision ROOT du13octobre

Préflight ROOT :113paquetsregistry dans Cargo.lock160 ;28archives/sources absentes du cacheCartae, toutes présentes surMac et vérifiées SHA contreCargo.lock160, total4 009 872octets ; sept entrées d'index nécessitent les versions verrouillées. Inventaire : /Users/moi/.cache/bridget-delivery161-20261013/cache-inventory.json. Une absence portant sur une plateforme étrangère ne prouve pas à elle seule un échec de buildLinux.

Décision explicite dans le périmètre de livraison autorisé : copie additive contrôlée des archives déjà présentes surMac, sans téléchargement tiers, modification de Cargo.lock, installationRust, nouveau script permanent ni option. Vérifier lesSHA à destination. Pour l'index, sauvegarde privée avec empreinte, comparaison préalable avant remplacement, refus d'une entrée étrangère divergente ; aucune fusion aveugle. Journaliser les fichiers ajoutés, sauvegardés et conservés. Si ce transfert ne peut être réalisé proprement, arrêter avantbuild et garder le lien096. Tout autre approvisionnement reste hors de cette décision. Cette préparation permet ensuite le build locked/offline ; elle ne prouve pas son succès.
