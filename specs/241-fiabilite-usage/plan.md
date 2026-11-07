# Plan 241 — Fiabilité des parcours courants

Statut : In Progress. Base : 04ad641e. Aucun changement de produit tiers requis
par défaut ; T3 reste ouvert. Les primitives SpecKit sont lues depuis les
skills installées ; les scripts/templates projet sont absents. Exécution
manuelle de leur protocole, sans recréer un runtime SpecKit dans le dépôt.

## Contexte et constitution

Produit Rust multi-hôtes, transport JSONL, reçus SQLite, helpers Python de
préparation, clients CLI/MCP et Android. Réutiliser ces composants. Les preuves
de 240 restent valides pour le cache de refus tardifs tant que son code ne change
pas. Une lecture statique ne remplace pas une qualification du parcours.
Articles XIX/XX : pas de nouveau moteur, service, base, framework de reprise
ou système de métriques. Les états et preuves restent dans les magasins natifs.
L'autorisation de la session et le mandat ne sont pas élargis par un chemin ou
un texte de mission. Les contrôles pertinents sont exercés aux frontières réelles.

## Lot A — Raccordement et admission durable

1. Localiser le délai JSONL par phases : connexion, inscription auxiliaire,
   dispatch, réponse. Lire les traces réelles et mesurer des lectures sans
   effet métier. Si les traces ne suffisent pas, ajouter des durées fermées aux
   points existants et qualifier cette instrumentation, sans contenu de trame.
2. Examiner la maintenance synchrone exécutée avant chaque message dans
   daemon.rs : collect_closed_attach_views, execute_controls et close_and_join.
   Un client lent ne doit pas bloquer une inscription sans rapport. Aucun lien
   causal historique n'est affirmé sans reproduction ou capture du délai.
3. Réutiliser registered_connection_until pour la récupération des erreurs
   transitoires AVANT envoi. Une inscription est sans effet métier ; les refus
   d'identité explicites restent terminaux. Ne pas imbriquer des reprises qui
   multiplient leur budget. Les mutations gardent send_then_wait et la
   récupération 171/232 de la même clé après envoi.
4. Donner aux nouvelles admissions une autorité durable propre à la mission,
   issue de l'attestation vérifiée AVANT admission. Réutiliser permission_snapshot
   et les contrôles natifs de propriétaire, révocation, sources, mapping et
   politique enfant. Distinguer explicitement ce contrat des tâches historiques.
   La réattestation HTTP d'un tour terminé ne doit plus devenir la condition
   d'exécution d'une mission déjà autorisée. Une preuve ancienne ne permet
   jamais d'admettre une nouvelle mission ni d'augmenter ses droits.
5. Contrôler toute évolution de l'autorité de mission depuis ses sources natives
   persistées (contracts/admission.md) ; la durée de conversation et la durée
   de mission sont distinctes. Vérifier arrêt/révocation natifs, propriétaire
   étranger, redémarrage et retour compatible. Modifier les droits des futurs
   tours ne constitue pas une annulation rétroactive implicite de cette mission.

## Lot B — Traductions utiles, fichiers et plafonds

1. Défaut Mac désormais constaté : les refus de sonde à corrélation
   678a3455 sont permission_mapping_unavailable/site=projection. Le parent
   atteste additional_directories=[projet, pièces jointes]. Les placements
   Linux ont un additional_root explicite ; le placement Mac n'en a pas.
   project_policy_directories171 exige une traduction même sur la machine
   d'origine. Distinguer origine=cible authentifiée et transport distant.
   Pour la machine d'origine, préserver un répertoire supplémentaire déjà
   attesté, existant et non protégé ; ne pas créer de mapping ou droit global.
   Sur une autre machine, garder l'autorisation et la traduction explicites.
2. Réutiliser les parcours general/shared hors Git. Accepter les variantes
   déjà couvertes par les droits de fichiers attestés, avec source et cible
   établies. Une racine réellement nouvelle garde une décision ciblée ; pas
   de faux Git ni de nettoyage destructif pour satisfaire la préparation.
3. Inventorier les plafonds publics et ceux qui interrompent ces parcours :
   volumes de sélection, manifestes, cartes, canon, texte, candidats, attentes,
   rétention, concurrence et délais. Classer intégrité/protocole, ressource,
   pagination ou arbitraire. Pour chaque borne : effet réel, justification,
   traitement progressif existant et décision. Supprimer les plafonds de volume
   total sans justification si le contrôle de disque et la lecture en flux
   couvrent déjà le coût ; garder les bornes nécessaires sur les trames en mémoire.
4. Étendre les lecteurs et pages existants plutôt qu'agrandir une réponse jusqu'à
   épuisement mémoire. Une limite historique ne doit pas bloquer une lecture
   d'état par clé. Garder les identités et empreintes d'intégrité nécessaires.

## Lot C — Outils nécessaires et preuve du service rendu

1. Réutiliser CandidateProbeV2, WorkerReadView et les préflights de programmes.
   Les ressources et compilation_capacity ne prouvent pas la disponibilité de
   Docker : la tâche bb7d6237 le mentionnait, sans machine imposée, et a été
   confiée à SYS. Identifier les prérequis de commandes connus sans exécuter
   le code du dépôt ni inventer une dépendance à partir d'un nom de projet.
2. Porter les exigences identifiées par le protocole natif négocié si nécessaire.
   L'agent n'a pas à renseigner un nouvel ensemble de paramètres pour le cas
   courant. Une mention ambiguë reste inconnue, pas une permission ni un refus
   général. Réutiliser le budget et le cache de la seule sonde pour éviter un
   appel système identique par modèle. Qualifier présence et utilisabilité sans
   installer ni démarrer de service. Respecter les axes humains imposés.
3. Réutiliser les fixtures de vrais échanges et les magasins temporaires pour
   les scénarios complets : coupure, concurrence, fin de tour, révocation,
   dossier local supplémentaire, source hors Git et dépendance manquante.
4. Mesurer et publier les nombres d'essais directs, récupérés, refusés, délais
   et interventions ; aucune extrapolation de fiabilité hors échantillon.

## Réutilisation de l'existant

Client : communication/client.rs, delegation_mcp.rs, t3code_mcp.rs.
Autorité : native_permissions.rs, daemon/native_permission_change.rs,
daemon/origin_proof.rs, daemon/native_delegation.rs et native_start_recovery.rs.
Sondes : remote_worker_read.rs et types/capacités de bridget-transport.
Sources : registry.rs, scripts/prepare-project.py, workspace_git.rs et
workspace_commit_view.rs. Pas de nouvelle table ni de dépendance externe prévue.
Mesures : journaux et reçus existants, preuves privées hors Git.

## Vérification et livraison

Tests de régression utiles avant correction ; Cargo via scripts/build.py
uniquement. Réutiliser le cache, sans target par test. Contre-revue du plan
puis du code par un autre fournisseur, audit ciblé, convergence exigence/preuve.
Livrer un lot cohérent. Version retenue : 0.8.8, correction des parcours déjà
autorisés, sans nouveau paramètre public obligatoire. Les contrats internes de
lecture évoluent ; leurs planchers imposent un retour241 compatible.
Construire les images et le retour compatible nécessaires, préserver les
registres et superviseurs, qualifier les formats/pointeurs rollback et les
processus réellement chargés. Actualiser les façades par le parcours natif
possible sans fermeture de T3 ; sinon déclarer ce point bloqué par l'exclusion.
Git : commit après gates, intégration main, push vérifié sur le remote existant,
alignement des quatre hôtes et nettoyage selon les gardes déjà établies.

## Conditions de sortie

Chaque exigence possède code et preuve, ou un blocage précis. Les limites non
résolues restent ouvertes ; pas de statut Implemented si une tâche reste ouverte.
Ne pas réécrire les tâches historiques pour vérifier le nouveau contrat.
