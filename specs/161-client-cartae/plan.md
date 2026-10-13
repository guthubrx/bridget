# Plan SPEC-161 — Client Bridget à jour sur Cartae

**Statut :** Implemented pour le développement14/14 ; pipeline In Progress. Réemploi PASS et contre-revue de plan traitée. Converge P1 puis P2 terminés sur le développement et les procédures ; contre-revue code et audit en cours, gates réelsG8/G9 ouverts, sans livraison annoncée.  
**Date :** 2026-10-13  
**Spec :** specs/161-client-cartae/spec.md  
**Estimation :** 2–4 heures, tests ciblés, revue, livraison, push vérifié et nettoyage inclus. Dépend principalement du cache Cargo Linux et du temps de compilation. L'attente d'un approvisionnement non autorisé est hors ETA.

## Résumé

Étendre l'outil client-only existant pour borner les ressources et contrôler les octets transférés avant build. Exécuter cet outil 161 depuis le worktree, avec une source Git isolée et propre exactement au commit 160. Installer dans un préfixe neuf, vérifier le client par le tunnel existant, puis basculer la commande courante de façon atomique. Le maître et T3 restent actifs sans intervention.

## Contexte technique

- Langages : Bash et Python 3 pour les tests et preuves existants ; sources compilées Rust 1.92 inchangées.
- Dépendances : SSH, rsync, Git, Cargo, outils système SHA et nice déjà présents ; aucune nouvelle dépendance.
- Stockage : fichiers privés de livraison et preuves ; aucune table ni migration SQL.
- Plateforme : runner macOS, compilation Linux x86_64 sur Cartae.
- Tests : scripts/tests/federation_089_test.sh ; recette existante scripts/tests/federation_095_roundtrip.py et lectures de statut.
- Contraintes : --locked --offline, RUSTUP_AUTO_INSTALL=0, Cargo jobs=2 et nice=10, une compilation active. Ne pas changer la politique globale des outils.
- Source : e6812e3e7b2d051d630192857cc3f440180d51fa ; Build ID attendu e6812e3e7b2d. Le runner 161 et les documents ne doivent pas contaminer la source compilée.

## Contrôle constitutionnel avant et après design

- I/III/X/XVI : français, lot autorisé, worktree161 dédié, étapes SpecKit traçables. Aucun code Rust modifié.
- XVIII : vérification de manifest linéaire dans le nombre d'octets transférés ; une passe, sans hachages répétés inutiles.
- XIX : réemployer deploy-remote.sh et son harness089. Aucun nouveau service, framework, wrapper permanent ou nettoyeur.
- XX : mainteneur dispose des commandes, invariants et preuves. Les tests concernent les frontières modifiées. L'outil de livraison reste distinct du code livré. Le rollback ne dépend pas d'un contexte conversationnel.
- Proportion des contrôles : commandes frontend non applicables à ce lot Bash/Rust sans frontend. Tests globaux Rust non requis car les sources compilées sont identiques à la livraison160 ; pas de suppression des échecs historiques.
- Validation : pas de violation non justifiée. Le plan ne crée pas de nouvelle architecture de runtime ; la séparation client/maître est déjà définie par095/096. Décisions locales consignées ici et dans research.md.

## Décisions architecturales

1. **REUTILISER deploy-remote.sh.** Ajouter seulement les contrôles nécessaires au manifest et à la borne de compilation. Préserver les gardes de federate-ssh.sh.
2. **Séparer runner et source.** Le runner vient de161, la source compilée d'un checkout propre160. Réemployer le format SOURCE-MANIFEST.sha256 existant de package089 après les conclusions de reuse-audit.md, plutôt qu’inventer un format voisin. Le manifest des fichiers transférables correspond à la référence source ; le distant vérifie les empreintes avant toute invocation Cargo de build. Ajouter --expect-commit <40hex>, optionnelle pour compatibilité historique, obligatoire dans la recette161. Avant SSH, refuser HEAD différent, arbre sale ou index assume-unchanged/skip-worktree. Sans option, préserver le comportement legacy (dont suffixe dirty). Ne jamais forcer l'identité160 sur une source dirty. Le manifest et rsync utilisent la même liste filtrée ; transmettre l'empreinte attendue du manifest par SSH, vérifier l'ensemble exact des fichiers distants et l'absence de liens avant Cargo. Réemployer le format SHA standard ; exclure explicitement les modes Git de lien et les exclusions existantes, pas tous les fichiers hors crates. Noms relatifs ASCII avec espaces autorisés ; refuser notamment newline/backslash et chemins ambigus.
3. **Toolchain explicite.** Utiliser /home/moi/.rustup/toolchains/1.92.0-x86_64-unknown-linux-gnu/bin/cargo après vérification, avec son répertoire dans PATH pour rustc. Observer les configurations Cargo applicables sans afficher de secrets, fixer CARGO_TARGET_DIR au target du préfixe et --jobs 2 ; mesurer nice dans le processus lancé. Le cargo rustup générique dépend sinon d'une sélection distante implicite.
4. **Préfixe neuf.** /home/moi/.local/lib/bridget-communication/161-e6812e3e7b2d est la cible proposée si absente ; sinon refuser et relever l'occupant, pas l'écraser. Après échec, une reprise explicite peut employer le suffixe neuf -r2 (puis numéro suivant relevé), sans réutiliser le préfixe abandonné ; chaque candidat reste au registre cleanup. Ses parents autorisés et privés sont recontrôlés.
5. **Bascule limitée.** /home/moi/.local/bin/bridget pointe actuellement vers /home/moi/.local/lib/bridget-communication/096/bridget. Vérifier propriétaire et groupe du parent et relever lien/identité/empreinte, préparer un lien temporaire voisin puis os.replace atomique après relecture de la cible courante. Ne pas chmod le répertoire partagé /home/moi/.local/bin (observé775) ni suivre une cible inattendue.
6. **Rollback symétrique.** Conserver l'ancien binaire et la cible textuelle. Tester le changement aller/retour dans une fixture avant la production ; en production, contrôler la lisibilité/exécution de la cible précédente et documenter la commande de restauration. Si smoke post-bascule échoue, restaurer seulement si le lien courant est encore celui installé par161. Ne pas écraser une modification tierce.
7. **Recette sans modèle.** Status/annuaire par le binaire candidat puis, séparément, roundtrip synthétique Python existant. Celui-ci atteste le transport et le maître, inscrit deux agents de recette et envoie deux messages par l’API ; ces écritures normales autorisées sont tracées, sans édition directe de base. Ne pas couper le tunnel pour tester sa reprise : aucun changement de ce tunnel dans161. Un statut seul ne prouve pas le roundtrip.

## Fichiers modifiés

| Fichier | Modification prévue |
|---|---|
| scripts/deploy-remote.sh | Manifest pré-build, jobs2, nice10 ; gardes et comportement client-only conservés. |
| scripts/tests/federation_089_test.sh | Adapter les doubles et assertions, couvrir corruption du transfert et échec sans activation. |
| scripts/package-089-core.sh | Embarquer le contrat161 requis par la fixture de bascule dans le paquet existant. |
| scripts/tests/package_089_test.sh | Vérifier sa présence avec le test de paquet existant ; aucun helper nouveau. |
| docs/federation-services.md | Procédure courte de mise à jour cliente et identité distincte du maître. |
| specs/161-client-cartae/ | Artefacts, analyse, revue, convergence, audit et reçu de livraison. |
| .specify/feature.json et bloc SPECKIT de AGENTS.md | Sélection161 et état de progression documentaire. |

La bascule du lien est une opération de livraison tracée, pas un nouveau service permanent. Si une automatisation versionnée minimale s'avère indispensable pendant l'implémentation, la rechercher et la trancher dans reuse-audit.md avant création ; pas de helper anticipé.

## Structure du projet

Conserver scripts/deploy-remote.sh comme seule entrée de préparation distante. Conserver le test089 comme harness de cette entrée. Le préfixe distant contient source/, bin/bridget et preuves de manifest ; l'ancien préfixe096 reste le rollback. Aucun nouveau crate, endpoint ou fichier de configuration runtime.

## Étapes d'implémentation

1. Audit de réemploi frais et contre-revue du plan ; ne générer tasks.md qu'après gate PASS.
2. Figement du manifeste de propriété et des preuves initiales : Git, liens, toolchain, processus protégés et fédération. Préserver l'index et les modifications étrangers.
3. Adapter le script et ses tests ciblés. Les doublures exécutent réellement le shell distant. Vérifier que transfert corrompu interdit Cargo build, qu'un préfixe occupé reste intact et que pipefail rend un build échoué non réussi.
4. Converge, contre-revue code et audit applicable. Intégrer/pousser le runner161 après gates ; tracer son SHA. Les sources compilées restent160.
5. Préparer le checkout source160 propre et son manifest. Construire offline dans le préfixe neuf, à priorité et parallélisme bornés. Cache incomplet : arrêt avant activation ; documenter la dépendance exacte manquante, SC-001 non atteint et pipeline non livré. Aucun approvisionnement réseau/vendor/cross-build implicite ; rechercher ensuite une solution autorisée.
6. Vérifier empreinte, architecture, droits, Build ID et communication du binaire candidat via la fédération existante. Relever le lien courant une seconde fois, activer atomiquement, vérifier statut et roundtrip. Rollback conditionnel sur échec.
7. Intégrer/pousser les reçus finaux, vérifier ls-remote. Réemployer le nettoyage projet build.py pour les caches éligibles avec ses gardes, sans supposer qu’il supprime tout le préfixe source. Retirer stages et WT/branche propres après transfert vérifié des preuves ; garder la cible active, le rollback096 et caches utiles explicitement justifiés.

## Stratégie de vérification

Bash syntaxe puis harness089 complet une fois sur le script final ; répéter uniquement les cas touchés en cas de correction. Recette réseau095 existante et lectures de statut avant/après activation. Vérifier les modes, le hash, le Build ID, la cible du lien et les naissances/PID des services protégés. Aucun nouveau modèle ou tâche utilisateur. Un seul build Linux nécessaire à la livraison ; tout second build demande une raison explicite (erreur ou sources modifiées). Les preuves160 restent historiques, pas un GREEN global de161.

## Risques

- Cache incomplet → offline obligatoire, aucune bascule, manque exact remonté.
- Fausse identité source → checkout propre160 et manifest distant contrôlé avant compilation ; runner identifié séparément.
- Charge excessive → jobs2, nice10, aucune compilation parallèle additionnelle.
- Modification concurrente du lien → comparaison avant bascule et avant rollback ; arrêt conservateur.
- Tunnel indisponible → conserver l'ancien lien si candidat non validé, rollback si échec après activation.
- Nettoyage destructif → chemins exacts, propriété, preuves, aucun rm global ni suppression de DB.

## Protocole SpecKit disponible

sync-project.py exécuté dans le WT le13octobre, RC0. Sélection explicite161 écrite dans .specify/feature.json. Templates spec/plan et .specify/scripts/bash/setup-plan.sh absents du WT et du checkout principal ; recherche dans le runtime utilisateur sans équivalent. Tentative de setup-plan après sélection161 : RC127, « No such file or directory ». Fallback documentaire conforme aux sections des skills specify/plan ; aucun script de toolkit inventé. Bloc contexte AGENTS actualisé directement comme autorisé par speckit-plan.

## Approvisionnement borné du cache — décision ROOT du13octobre

Préflight ROOT :113paquetsregistry dans Cargo.lock160 ;28archives/sources absentes du cacheCartae, toutes présentes surMac et vérifiées SHA contreCargo.lock160, total4 009 872octets ; sept entrées d'index nécessitent les versions verrouillées. Inventaire : /Users/moi/.cache/bridget-delivery161-20261013/cache-inventory.json. Une absence portant sur une plateforme étrangère ne prouve pas à elle seule un échec de buildLinux.

Décision explicite dans le périmètre de livraison autorisé : copie additive contrôlée des archives déjà présentes surMac, sans téléchargement tiers, modification de Cargo.lock, installationRust, nouveau script permanent ni option. Vérifier lesSHA à destination. Pour l'index, sauvegarde privée avec empreinte, comparaison préalable avant remplacement, refus d'une entrée étrangère divergente ; aucune fusion aveugle. Journaliser les fichiers ajoutés, sauvegardés et conservés. Si ce transfert ne peut être réalisé proprement, arrêter avantbuild et garder le lien096. Tout autre approvisionnement reste hors de cette décision. Cette préparation permet ensuite le build locked/offline ; elle ne prouve pas son succès.
