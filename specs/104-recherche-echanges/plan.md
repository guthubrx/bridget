# Plan 104 — Recherche bornée, sans second moteur

## Base et dépendances

Base observée main1738a072. Racine de travail :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges

102 n'est pas encore implémentée. Pour livrer104 entière, intégrer son code validé en premier,
puis utiliser ses tables et son contrôle d'appartenance. Le volet messages peut être développé
et testé avant, mais cela ne permet pas de cocher la partie fils ni de déclarer104 livrée.
Ne jamais copier de fichiers102 non commités à l'aveugle ; demander/intégrer sa branche une
fois publiée, avec vérification du contrat exact.103 n'est pas une dépendance de code.

## Décision et minimalisme

Étendre le module store/ledger_requests.rs et les accès ledger existants.
Supprimer la recherche SQL interne morte une fois remplacée, pas la conserver à côté.
Réutiliser fold_for_search comme UNIQUE normalisation des corps et termes en Rust :
la variante SQL actuelle n'est pas équivalente et ignore certaines erreurs de lignes.
Aucune nouvelle dépendance, table de recherche, service, FTS ou index sémantique.

On accepte un coût linéaire dans les corps effectivement parcourus pour une recherche de
sous-chaînes, mais borné par appel et reprenable ; on ne promet pas une recherche exhaustive
en1s sur n'importe quelle archive. Indexer les voies d'accès par participant, pas le texte.

## Parcours d'une page messages

1. Sous verrou daemon bref : extraire identité+instance attestées et chemin de base canonique
   déjà configuré ; réserver un permis global (maximum2 recherches/lectures simultanées).
   Si plein : busy immédiatement, aucune file d'attente sans borne.
2. Relâcher le verrou ; utiliser le thread de connexion existant, pas créer un pool/service.
   Ouvrir une connexion SQLite READ_ONLY ; ne pas appeler Store::open qui initie le schéma.
   Busy timeout100ms, toute erreur storage_unavailable, aucune base de secours.
   Ouvrir une transaction de lecture courte pour capturer les clés et corps de la page ;
   la terminer AVANT normalisation, hash, rendu, envoi réseau ou retour sous verrou daemon.
   Ne pas supposer que WAL est activé : le socle ne le configure pas explicitement.
3. Valider requête/cursor. Pour les messages, construire deux plages SQL indexées :
   sender=actor ; target=actor AND sender<>actor. Date et borne de curseur dans SQL.
   Contrôler les longueurs en SQL avant matérialisation des identifiants : marqueur d'erreur
   plutôt que chaîne>256octets transférée. Conserver le tri sur les colonnes originales,
   pas sur les alias CASE. Erreur de métadonnée avant construction d'un curseur invalide.
   Sélectionner au plus129 clés par branche avant fusion ; ordre (ts,id,target) DESC,
   comparaison lexicographique binaire. Fusionner les clés (≤258) puis retenir129.
   Ces premières requêtes rapportent aussi sender et length(CAST(body AS BLOB)), jamais body.
4. À partir des métadonnées, sélectionner au plus128 candidats, leurs corps admissibles
   totalisant<17Mio : arrêter la sélection dès1Mio atteint, avec une seule ligne entière
   supplémentaire dépassant le budget restant. Les lignes rejetées par peer/author ou
   taille>16Mio n'ont pas de corps à charger, mais gardent leur clé et leur motif.
   Charger UNIQUEMENT les corps admissibles sélectionnés par UNE requête VALUES/JOIN,
   dans la même transaction de lecture ; revérifier sender/target dans cette requête.
   Ne pas sélectionner les129corps puis compter les octets côtéRust : le triSQL pourrait
   déjà avoir matérialisé plusieursGio. Le lot SQL lui-même doit être borné<17Mio.
   Capturer les corps, puis fermer statement ET transaction avant tout traitementCPU.
   Les lignes préchargées mais non traitées (limite de résultats atteinte ensuite) ne sont
   pas consommées dans le curseur. Les rejets par filtre comptent comme candidats traités.
5. Pour chaque ligne autorisée : contrôler taille, normaliser corps UNE fois et rechercher
   chacun des≤8 termes avec find. Tous doivent être présents. Garder le plus petit offset
   trouvé, puis refaire UN parcours des caractères originaux en cumulant la longueurUTF-8
   de leur repli, jusqu'à l'offset normalisé : match_offset est le début du caractère
   original correspondant, jamais l'indice normalisé réutilisé comme indice brut.
   Partager la règle de repli d'un caractère avec fold_for_search, sans allouer une chaîne
   par caractère ni un tableau d'offsets pour tout le corps. Un caractère replié peut
   donner plusieurs caractères : prendre son début original. Extrait depuis match_offset,
   ≤512octets UTF-8 ; body_digest=SHA256 du corps entier pour les hits uniquement.
   Cela permet read(offset=match_offset,digest=body_digest) sans relire tout le préfixe.
6. Stop après128 candidats réellement traités, ou≥1Mio de corps cumulés, ou limit résultats.
   Une ligne dépassant le budget restant est néanmoins traitée entière pour garantir le
   progrès, tant qu'elle≤16Mio. Au plus UNE ligne hors budget par page ; après elle arrêt.
   Maximum de corps traités/page<17Mio.
   Une ligne>16Mio est ignorée explicitement avec warning oversized_source et compteur local
   skipped_oversized ; ses corps ne doivent pas être chargés (prélecture length en SQL).
7. Construire next_cursor à la dernière clé réellement traitée, pas à la dernière trouvée
   ni à la dernière clé préchargée. Il peut exister avec hits=[].
   Si arrêt anticipé et présence d'au moins une clé suivante connue : has_more=true.
   Si128 traitées, la129e sert seulement de témoin d'une suite, pas d'un résultat.
8. Avant publication de la réponse, revérifier connexion/instance toujours attestées.
   Identité révoquée : abandonner le contenu et retourner identity_unavailable.
   Relâcher le permis en RAII sur tous les retours et déconnexions.

La sortie bornée inclut≤50 extraits de512octets et métadonnées, plafond60Kio. Si la prochaine
entrée dépasse la place restante, ne pas avancer son curseur. Un identifiant hérité
(id/sender/target)>256octets provoque source_metadata_too_large sans réponse partielle
ni valeurs dans l'erreur ; ne pas fabriquer un curseur tronqué pour l'ignorer. Contrôler
leurs longueurs avant chargement. Une ligne normale doit toujours pouvoir tenir seule.
Aucun corps/terme/fingerprint de requête dans logs ou métriques.

## Parcours d'un fil102

source=thread exige thread_id UUID ; vérifier membership dans la même connexion/transaction
de lecture que les lignes. Réutiliser le contrôle102, ne pas déduire membership de l'annuaire.
PK(thread_id,seq), ordre seq DESC, plafond128candidats et budget1Mio comme messages.
À la première page fixer upper_seq=last_seq ; les suivantes restent sous cette borne.
Les entrées102 sont immuables : cette source offre un instantané de borne haute.
Filtres auteur et dates appliqués au parcours ; peer interdit. Réutiliser la normalisation.
Pas de modification thread_reads/discussion_members/thread_wakes.
Résultat : thread_id,seq,message_id,author_id,created_at,excerpt,history locator.
Titre absent par défaut pour éviter de relire la table titre pour chaque résultat.
Non-membre ou fil absent : même not_found_or_forbidden, aucun compteur distinct.

## Pagination et relecture

Voir contracts/search-api.md et data-model.md, normatifs.
Messages : pagination vivante, non snapshot (INSERT OR REPLACE existe déjà et purge possible).
Une borne supérieure de première page stabilise le haut ; aucun rowid ni OFFSET.
Corps inchangés : zéro duplication/perte ; changements en cours : recommencer pour vue fraîche.
Relire un message exact exige id+target et accès participant. Fragments16Kio UTF-8,
digest SHA256 du corps entier ; si digest diffère au prochain appel, content_changed
sans morceau. Pas de fusion silencieuse entre deux versions ; ne pas exposer corps ni nouveau
digest au refus d'autorisation. Fils : contexte par bridget_thread history, pas read/ack.

## Exposition et compatibilité

bridget_ledger conserve son contrat historique si action absente ou recent.
Actions nouvelles search et read ; schémas fermés par action. CLI ledger historique inchangé ;
ledger search et ledger read apportent les mêmes actions, sortie humaine et --json.
Nouveau protocole WrapperToDaemon::LedgerSearch/LedgerRead et réponses typées, sur
registered_connection + live_connection_identity099. Aucun agent/instance choisi en paramètres.
Volet messages développé avant102 : source=thread valide syntaxiquement renvoie
capability_unavailable, jamais invalid_params ou un succès vide. Ce mode intermédiaire
ne satisfait pas la livraison104 entière. Ancien daemon : refus/erreur explicite
capability_unavailable ou daemon_protocol ; timeout
reste une erreur technique, jamais fallback ledger global ni accès direct au fichier client.
Aucun outil MCP supplémentaire ; mettre à jour le schéma de bridget_ledger et ses tests.

## Index et complexité

Ajouter dans Store::init_schema, idempotemment :
idx_ledger_sender_page ON ledger(sender,ts DESC,id DESC,target DESC)
idx_ledger_target_page ON ledger(target,ts DESC,id DESC,target DESC)
Garder idx_ledger_ts/idx_ledger_conv nécessaires aux fonctions existantes.
Aucune colonne/body normalisé dupliqué ; aucune donnée migrée. Les index ajoutent du coût
aux écritures : mesurer messages témoins avant/après, rollback possible sans perte de données.

Messages : O(M log N + M log M + K×B) avec M≤258 clés et K≤8 termes, B<17Mio/page ;
mémoire des corps : capture<17Mio + normalisation d'un corps≤16Mio (expansion Unicode
bornée) + sorties60Kio et clés bornées ; aucune duplication de tous les corps normalisés.
Fil : O(log N + M + K×B). Total de toute l'archive demeure O(B_total×K) sur plusieurs appels.
Lecture exacte : index PK O(log N), empreinte O(B) bornée à16Mio, fragment16Kio.
Capturer le corps sous transaction courte, fermer celle-ci avant hash/découpage.
Lire intégralement un gros message par fragments coûte O(B×ceil(B/16Kio)), car les écrivains
hérités ne conservent pas de digest/version stable. Compromis explicite de compatibilité,
B≤16Mio, chemin froid : ne pas ajouter un cache d'instantanés ou modifier tous les écrivains.
Le chemin quotidien utilise match_offset+body_digest pour ne lire que le passage utile ;
aucune boucle automatique de lecture intégrale n'est prévue. Mesurer le pire cas par appel.
Vérifier EXPLAIN QUERY PLAN des deux plages : index, LIMIT effectif avant fusion, pas
de scan intégral ni tri non borné. Une amélioration d'index optionnelle exige mesure,
pas ajout opportuniste de FTS5 ou dépendance.

## Fichiers prévus

Préfixe absolu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/

- crates/bridget-daemon/src/store/ledger_requests.rs : remplacer search_messages, normalisation,
  accès paginé exact, sélection bornée ; retirer les helpers morts après rg.
- crates/bridget-daemon/src/store.rs : index et ouverture read-only étroite réutilisant les validations.
- crates/bridget-daemon/src/ledger.rs : projections résultats/validation de limites et curseurs.
- crates/bridget-transport/src/protocol.rs : contrats104.
- crates/bridget-daemon/src/daemon.rs : identité, permis, absence de verrou pendant scan.
- crates/bridget-daemon/src/mcp.rs et src/cli.rs : mêmes actions sans logique SQL.
- module Store des fils créé par102 : lecture séquencée et contrôle d'accès réutilisés.
- crates/bridget-daemon/tests/search_104_test.rs et src/store/tests.rs : oracle de résultats,
  erreurs, EXPLAIN, droits et performance.
- skills/bridget/SKILL.md, skills/bridget/references/commandes.md, README.md : recettes.

Pas de GUI, Maicie, fournisseur ou permission système nouvelle. Tests Rust natifs, scénarios
métier Given/When/Then dans test-plan.md, pas de pile Pytest ajoutée à ce projet Rust.
