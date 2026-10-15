# Modèle de données SPEC168

Statut : conception. Réemploi du DelegationStore et du registre existants, sans nouvelle base d’authentification.

## 1. Connexion déclarée

Champs : connection_id opaque stable, machine_id, moteur réel, agent_type interne existant,
enabled, label facultatif sans donnée de compte, référence de définition/digest.
Une connexion vise un environnement déjà authentifié ; les chemins et variables privées restent dans le lanceur local.
L’ID ne certifie pas l’identité du compte et ne donne aucun droit ni accès réseau.
Un lanceur peut exposer plusieurs modèles ; pas de duplication par modèle.
La machine résout localement l’ID et recontrôle sa définition au lancement.
Changer moteur/machine/digest invalide les anciennes attestations ; pas de reprise sur un autre compte implicite.

## 2. Catalogue de connexion

Champs : connection_id, observed_at, provenance, modèles réels, efforts acceptés, état de capacité.
La découverte est bornée ; Codex réemploie model/list, Claude la déclaration validée existante.
Une déclaration supportée n’est pas une preuve de crédit ou d’accès fournisseur.
La liste des versions ne vient ni d’un alias de famille ni d’un registre d’une autre machine.
La projection publique ne contient jamais environnement, token, chemin d’authentification ou email.

## 3. Option de routage

Tuple candidat : candidate_id, machine_id, engine, connection_id, agent_type interne, model,
supported_efforts et champ default_effort obligatoire, imposed_effort optionnel. Si des niveaux
sont exposés, default_effort appartient à supported_efforts ; sinon default_effort=null et liste vide
désignent le défaut natif sans flag. Un effort imposé est alors incompatible. Aucun medium inventé.
Le choix final résout ce défaut ou un niveau autorisé, sans option distincte par effort.
IDs168 : c001..c128, uniques et stables dans le snapshot ; IDs163 c01..c08 inchangés.
La borne globale statique est de128tuples configurés avant filtres de demande ;129est une erreur
de registre, jamais une coupe ou un catalogue partiel.
Les filtres de demande s’appliquent avant le classement.
Les connexions/modèles/efforts non attestés sur la cible ne sont pas introduits artificiellement.
Le mapping opaque→tuple reste maître ; le décideur reçoit les labels utiles et les efforts compatibles,
pas les références privées. Une réponse choisit candidate_id et effort ; une valeur absente emploie
le défaut attesté, une valeur hors liste ou contraire à imposed_effort refuse la réponse.

## 4. Snapshot de machine

Champs : machine_id, worker_id/époque ou identité locale, observed_at source, received_at maître,
cpu_busy_ratio?, memory_available_bytes?, memory_total_bytes?, load_per_cpu?,
compile_slots déclarés?, compile_occupied_slots observés?, continuité dépôt/cache déclarée ou attestée.
Les unités et provenance sont explicites. Les ratios valides sont bornés ; un échantillon invalide devient unknown.
CPU exige deux échantillons valables mémorisés, rafraîchis au heartbeat existant et au probe,
une collecte par machine/lot ; aucune attente bloquante ni nouveau daemon pour fabriquer le premier.
Received_at et âge source empêchent un futur distant incohérent de valider une mesure.
TTL défaut30s, configuration1..300s. Une ancienne époque est rejetée même avec un timestamp récent.
Aucune donnée de processus, utilisateur, chemin privé ou secret n’est fournie au décideur.
La capacité compilation déclarée reste distincte d’un compteur extérieur à Bridget inconnu.

## 5. Fait de quota rattaché

Réemployer les observations157/160 : type, instance attestée, fenêtres, limite, crédits utilisables,
contrôle de dépenses et observed_at. Associer la connexion uniquement avec sa définition et l’instance prouvées.
Ne pas créer un pool financier par libellé de compte ; ne pas fusionner des faits entre machines sans preuve.
Le fait manquant/périmé reste unknown. La remise réelle à l’enfant recontrôle son budget propre.
Une mise à jour partielle ou une identité différente peut retirer immédiatement une autorisation ancienne.
Les soldes opaques ne sont jamais un critère numérique du score ou du décideur.

## 6. Politique de routage168

Version, révision/digest, connexions autorisées, modèles/efforts exposés,
valeurs fixes et axes automatiques indépendants, maximum128options,
réglages par machine : enabled, télémétrie/TTL/seuils ; poids pris uniquement dans la politique
de la machine coordinatrice et appliqués uniformément à toutes les cibles,
service_decision enabled/ref accessible, lightweight_decision enabled/ref accessible,
budget_total_ms et budgets de chaque niveau.
RoutingScoreWeights représente les sept coefficients privés fermés cpu/ram/load/continuity/
compilation/quota/reset, bornes et défauts20/20/20/20/10/5/3. Tous défauts : omettre weights
du JSON canonique pour conserver le digest des anciens snapshots168. Non défauts : sérialiser
et figer dans politique/snapshot. Ce type ne duplique ni les seuils ni une entité de service.
Absent = garder163 ; présent mais disabled = pas d’automatisme168.
Un niveau inaccessible/désactivé n’émet aucun appel.
Une configuration invalide refuse avant réservation ou appel extérieur ; les reçus déjà admis gardent leur politique.
Les URLs viennent de la configuration autorisée, jamais du brief ni du décideur.
Le brief HandoffV1 reste inchangé : AutoDelegateV2.workload est séparé, fermé general|compilation,
general par défaut. Ce champ normalisé est conservé dans canon, snapshot et charge aux décideurs.

## 7. Snapshot d’admission et reçu durable

Table existante auto_admission_receipts ; unicité owner durable/request_id et conflits natifs existants conservés.
Métadonnées sans version =163, ≤16Kio ; metadata_version=2 =168, ≤128Kio.
Le canon complet reste≤512Kio et les limites de trame restent16Mio maître/2Mio worker.
Valider avant écriture ; dépassement explicite sans snapshot partiel ni effet modèle.
Les métadonnées168 figent :
- révision/digest et contraintes de demande ;
- tuples complets et leurs IDs, faits frais utiles/provenance/unknown ;
- score entier et ordre déterministe, candidat de dernier repli ;
- configuration autorisée des décideurs, budgets et niveau attendu ;
- source de décision et raison affichable≤256octets, sans secret automatiquement injecté ;
- workload et expected_base_commit fournis, nécessaires aux recontrôles de reprise.

Le mode SQL garde son CHECK explicit|automatic|fallback : sources explicit/single→explicit,
decision_service/lightweight_model→automatic, deterministic→fallback. La source détaillée reste
exclusivement dans metadata v2 et la projectionv4 ; ne pas ajouter un mode SQL nouveau.
Réutiliser les colonnes candidate_id/mode/reason existantes avec validation par version,
et le canon Delegate choisi. Ne pas copier le corps de mission à chaque transition.
Si le modèle choisi implique un autre agent_type interne, l’autorité vient uniquement du snapshot validé.
Le mapping connexion→définition épinglée doit rejoindre le même puits explicite162.
Aucune table « compte », solde, historique de facturation ou nouveau scheduler.

## 8. Cycle de vie et concurrence

pending → decision_started(service) → decision_started(lightweight) → chosen → admitted.
Branches : pending peut choisir directement un candidat unique ; un niveau peut être sauté si désactivé.
Le score final peut produire chosen après les deux niveaux. Un refus métier fermé produit refused.
Le niveau courant et sa révision sont persistés avant émission ; le changement de niveau est un CAS.
Une réponse tardive doit correspondre à l’époque, l’ID admission, la révision et le niveau encore actif.
Une erreur technique/SQL garde le dernier état confirmé, avec diagnostic ; elle ne fabrique pas un refus.
chosen conserve l’élu et son canon ; admission Task+liaison reste atomique dans la transaction existante.

Après panne : pending/decision_started d’une ancienne époque utilisent les profils et le repli figés,
sans nouveau service ou modèle. Recontrôler droits et disponibilité ; absence de repli admissible donne
le refus existant approprié, sinon laisser une erreur technique récupérable. chosen reprend son seul canon.
Rejouer renvoie l’état durable, jamais une politique recalculée. Une autre enveloppe refuse.
L’arrêt, la pause, l’annulation, la rétention UNION4096 et les limites actives existantes restent applicables.

## 9. Base Git et workload

AutoDelegateV2.expected_base_commit est optionnel dans la forme. Pour tout espace Git168,
Shared ou Worktree, sa présence est obligatoire avant fournisseur ; absence = workspace_base_required.
Il contient un SHA exact source fourni par le coordinateur, jamais un HEAD distant choisi par défaut.
Worktree.base_ref résolue doit produire le même SHA. Shared horsGit avec workload=general
peut omettre la base ; ne pas inventer un dépôt. Le snapshot et probev2 conservent ce champ,
WorkerPrepare réemploie son champ base_commit et verify_base_commit avant lancement.
Le brief HandoffV1 fermé n’est modifié pour aucun de ces champs.

## 10. Projection de décision

Lineagev4 : mode, candidate_id168, connection_id opaque, moteur/modèle/machine/effort résolu choisis,
source=explicit|single|decision_service|lightweight_model|deterministic,
raison optionnelle, provenance/âge des faits utiles.
Aucun sujet fournisseur ou texte de raison ne devient une instruction.
La réponse publique n’expose pas la configuration privée des connexions ni le prompt du décideur.
Lineagev3 pour une tâche168 : placementv2 conservé, decision:null.
Une tâche163 continue à rendre sa décisionv3 c01..c08 sans conversion.

## 11. Rétention et migration

Réemployer l’UNION reçus/Task, purge sûre, tombstones et gardes de vivacité ; mêmes clés et durées.
Les snapshots168 doivent compter en rétention et ne jamais être purgés pendant un état ambigu ou actif.
Une base neuve crée directement le DDL168 avant admission, sans créer puis migrer une table163.
Une base163 existante reconstruira transactionnellement auto_admission_receipts, sans seconde table métier durable :
ajout metadata_version INTEGER DEFAULT1, CHECK16Kio pour1 /128Kio pour2.
INSERT…SELECT recopie explicitement les colonnes legacy et conserve leurs BLOB octet-identiques,
puis remplacement de table et recréation des trois index dans le même commit.
Une table168 existante n’est pas remigrée ; schéma inconnu/FK ou trigger inattendu refuse.
Un échec à n’importe quelle étape laisse le schéma et les lignes163 intacts.
Lire la versionSQL et la versiondu snapshot de façon cohérente ; canon512Kio inchangé.
Oracles : migration de fichier163 avec reçus dans chaque état, hashesBLOB avant/après,
réouverture/rejeu163, reçu168>16Kio, borne128Kio, rollback de copie/index, quick_check/FK0,
CAS et rétention sans double compte. Sauvegarde cohérente et retour arrière sans perte obligatoires.
Dès qu’une ligne metadata_version=2 existe, ancien binaire interdit même avec168OFF :
désactiver168 en conservant le binaire compatible,
ou restauration cohérente autorisée sans perte d’écriture. Aucun rollback automatique de base.
