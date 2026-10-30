# Contrats221 — règles communes

## Autorité et compatibilité

Toute opération publique désigne propriétaire, projet, machine, ressource et version attendue lorsque nécessaire. Conserver l’enveloppe d’admission existante. Une forme nouvelle exige capacité négociée ; un ancien pair refuse avec une action utile, sans mutation partielle. Les clients humains authentifiés ne confèrent pas ces droits aux agents.

## États observables

requested, admitted, running, result_available, stopped et cleanup_done restent distincts. not_observed, unsupported, expired, refused et outcome_unknown ne deviennent pas succeeded. Les références de compte sont opaques. Un catalogue est une observation, pas un droit.

## Mutations

Clé stable et canon avant effet. Une mutation initiale ne promet pas un effet confirmé. Prévisualisation et version/CAS avant édition de file, restauration, changement de fournisseur ou modification de configuration. Aucun nouvel essai de mission d’issue inconnue.

## Erreur

Code technique stable ; prochaine action utile, portée, cause constatée ou inconnue, vérification manquante, tâche/clé conservées, décision humaine si nécessaire. Les autres travaux suivent leur état. Aucune valeur secrète ni stderr libre.

## Surfaces à préciser avant leur implémentation

L’extension des verbes de file, des sessionsoutils, des imports/checkpoints, des secrets et des événements doit obtenir un schéma borné, les rôles autorisés, la machine cible, le cycle de vie et les preuves d’effet dans ce dossier. Ces contrats ne sont pas supposés définis par ce seul document.

## C26 — clé, canon et transaction existants

RÉUTILISER les stores idempotency et DelegationStore. La clé est bornée au propriétaire authentifié. Même clé et même canon rendent la même issue logique ; une différence du canon refuse avec envelope_mismatch. Un reçu Admitted conserve la même Task ; un reçu Chosen conserve le choix avant toute nouvelle décision. Aucun résultat de transport ne permet de fabriquer un remplacement.

L'admission atomique existante lie Task et reçu dans une seule transaction Immediate. La projection est publiée après commit. Une panne avant commit laisse la transaction annulée ; après commit, la relecture part du reçu avant toute nouvelle sélection. Le rejeu ne promet pas une exécution exactement une fois d'un effet métier externe non transactionnel. Les opérations qui ne sont pas déclarées rejouables gardent leur contrat propre.

La recette C26 exerce directement les vrais handlers et les bases SQLite fermées/réouvertes, avec panne SQL et concurrence. La perte physique d'un paquet TCP n'est pas exercée. Le résultat du handler n'est remis à l'appelant qu'après transaction ; perdre cette réponse après retour ne change donc pas l'état qui est relu par le rejeu. Cette limite ne devient pas une qualification réseau.

## C27 — refus de sonde guidés, sans cause inventée

Étendre seulement la projection existante. Les raisons demeurent des triplets fermés kind/code/candidates, avec les mêmes bornes et les mêmes codes publics. Ajouter une vue de vérification dérivée de ces codes après leur validation ; jamais depuis le stderr, un chemin, la note libre ou les réglages d'un fournisseur. Relecture du code sérialisé et projection initiale doivent produire les mêmes conseils. Le reçu et son canon ne sont pas réécrits.

Chaque motif affiche une prochaine vérification précise, la portée limitée aux candidats concernés, la cause détaillée explicitement inconnue quand elle n'a pas été observée, et la décision à demander à l'utilisateur si la correction dépasse le mandat. Le seul code CapabilityUnavailable ne prouve ni programme absent, ni abonnement inadapté, ni panne, ni refus de droits : vérifier la déclaration et les capacités négociées du même profil/modèle/effort sur la même machine. L'état ancien ne prouve pas l'état actuel.

La réponse générique sans motif ou avec un motif invalide reste exploitable : preuve manquante indiquée, recours explicite, aucune valeur libre exposée. Conserver clé/enveloppe et toute Task connue. Tout rejeu exige la condition de detail.admission et la fin des appels concurrents ; pas de repli de moteur, machine ou droits. Les autres travaux suivent leur propre état. Cette correction ne prétend pas reconstruire le refus historique de la revueL1a.

## T177 — forme workspace refusée avant connexion
Réutiliser WorkspaceRequest et le code JSON-RPC InvalidParams (-32602). Le client valide le même schéma fermé pour delegate et auto_delegate, sans modifier les formes admises. En shared, seul strategy est autorisé ; base_ref et branch appartiennent à worktree. Dans worktree, ces champs sont des chaînes si présents, jamais null. Un refus rend la règle et l'action, pas le champ libre fourni, sa valeur ou l'erreur serde brute.

Ce refus concerne cet appel local non envoyé. Il ne prouve pas qu'une demande antérieure de même clé soit absente. Avant correction, attendre la fin de l'appel et vérifier tout reçu ou Task déjà connu ; ne pas changer le canon d'une admission persistée ni créer un remplacement d'issue inconnue. Les autres travaux suivent leur propre état. Si le schéma chargé diffère, demander la reconnexion complète du client MCP ; constater l'image réellement chargée, sans boucle de reprise.

Recette : formes nominales inchangées, shared avec champ interdit, stratégie/type/valeur invalides, chaînes privées jamais recopiées, aucun échange sur le socket en cas de refus, code JSON-RPC stable. Le schéma décrit ces règles sans ajouter de champ ni capacité.

## T178 — aucune combinaison déclarée après les filtres
Réutiliser routing_admission_plan, le refus public routing_candidate_unavailable et la projection failure. Lorsque select_routing_candidates retourne une liste vide après la politique et les contraintes, conserver le seul sous-code fermé catalogue_no_match. Ne pas réutiliser ce sous-code après une sonde ou une revalidation : la phase a alors changé.

La projection expose stage=catalogue_selection, cause=no_active_declared_combination et scope=this_admission. Ce fait décrit le catalogue et ses filtres à cet instant, jamais l'accès fournisseur, l'état d'un compte ou l'état courant. Lire fleet.options avec les mêmes contraintes et la politique de la machine coordinatrice ; aucune combinaison manquante ne justifie un repli silencieux. Si nécessaire, demander au responsable de déclarer ou qualifier exactement la combinaison autorisée, ou à l'utilisateur de décider d'une contrainte explicitement modifiée. Le rejeu suit detail.admission et l'absence d'appel concurrent. Ni le canon, ni les comptes, ni la preuve d'une Task existante ne changent.

Le code ancien sans sous-code et les suffixes invalides restent inconnus. Ce sous-code est un diagnostic sans nouvelle opération ni autorité ; les anciens clients relaient Value et les anciens maîtres restent génériques. Tests : sélection initiale et relecture, politique active sans correspondance, aucun fournisseur/Task/reçu, ancien refus inconnu et suffixe non reconnu sans valeur libre.

## C18 — matérialisation native Git sans écrasement
Réutiliser git checkout-index sans --force, avec --ignore-skip-worktree-bits et -- avant le chemin explicite. Cette opération rend disponible la version de l'index Git dans le worktree concerné ; elle n'importe pas une autre branche et ne réinitialise aucun fichier. Un fichier déjà présent doit être refusé, même si ses octets correspondent à l'index. Ne pas ajouter --force, --all ni changer les drapeaux manuellement. Aucune nouvelle opération, permission, API ou dépendance Bridget.

Le conseil actuel git restore permettait de remplacer un fichier, ce que son avertissement reconnaissait. Le remplacer par la primitive sans écrasement, puis vérifier avec le Git réel : première extraction d'un fichier omis, seconde tentative sur modification locale, octets/inode préservés, rapport propre puis dirty, HEAD/index du parent inchangés. Tester aussi un lien et un dossier déjà présents. La documentation Git et la source2.56 entry.c corroborent O_EXCL ; ne pas transformer cela en garantie contre toute modification adverse du système.


## Liaison d'un fil long — T179
L'adaptateur demande la représentation bornée compacte officielle lorsqu'elle existe. Le drapeau explicite de réponse détermine la reconstruction des seuls événements locaux, avec identité de fil et unicité vérifiées. Les événements hérités ne deviennent pas ceux du fil courant. Les événements de sous-agent passent encore par les contrôles de filiation existants. Questions et outils gardent leurs identités et bornes de détail.
Le client n'augmente pas la limite HTTP, ne raccourcit pas silencieusement une réponse, ne supprime pas l'historique et ne déclare aucune présence depuis une seule attestation. Si la représentation compacte dépasse encore la limite, le lien reste non vérifié. Les anciennes réponses non compactes et le protocole précédent gardent leur lecture existante.


## C16 — préférence future sans transfert de l'autorité courante
Conserver la clé et les octets du mapping A utilisé par une Task. Ajouter B sous une autre clé avec ses racines qualifiées ; ne déplacer ni A, ni son worktree, ni son reçu. La préférence n'est pas une permission. Les rechecks, reprises, callbacks, slots et annulations continuent de relire la clé exacte figée dans la Task et les révocations courantes.

Réutiliser la racine JSON tolérante du registre privé, sans sidecar, nouveau store ni nouveau protocole : section optionnelle project_preferences, forme {"version":1,"machines":{"cartae":["projet-b"]}}. Les listes contiennent au plus64clés distinctes déjà déclarées sur cette machine. Maximum16machines. L'opérateur publie cette section après qualification de B et de la présence du commit demandé ; aucune admission n'installe cette préférence. Le fichier reste soumis aux contrôles privés du registre. Les configurations origine, maître et worker doivent déclarer B identique avant la préférence ; les publications existantes vérifient le mapping exact et refusent une divergence. Une publication partielle n'est jamais annoncée complète ; rejouer les mêmes octets après contrôle CAS, ne retirer A ni sur échec ni au retour.

La sélection avec project_id explicite reste stricte et ignore la préférence. Sans identifiant, un seul mapping correspondant conserve son comportement. Plusieurs correspondances ne sont départagées que si exactement une est préférée et si toutes partagent les mêmes source_host/source_root et additional_root. Ne pas départager des projets imbriqués ou des droits additionnels différents. Préférence invalide ou référence absente : refuser seulement cette sélection implicite à plusieurs correspondances, sans invalider la lecture du registre ni les tâches A exactes. Aucun changement du canon de la demande ; transmettre seulement l'identifiant résolu à la préparation source afin qu'un ancien helper puisse préparer B explicitement.

Aucun digest historique supplémentaire : le choix est effectué dans le registre courant déjà autorisé, puis les empreintes et placements de l'admission existante restent stricts. Les anciens lecteurs ignorent la section racine ; avec A+B ils refusent l'implicite ambigu et continuent de connaître A et B explicites. Retour : retirer la préférence ne déplace aucune tâche et rétablit l'ambiguïté fermée. Ne retirer un mapping qu'après clôture native et inventaire de ses usages.

La création physique de B réutilise prepare-project : destination existante qualifiée ou nouvelle destination dans son espace géré. Ce lot n'élargit pas ce périmètre aux chemins arbitraires et n'ajoute pas une API humaine de configuration depuis un simple même UID. Les opérations de configuration hors de ce périmètre restent opérateur ; aucune promesse de migration automatique de données ou de création de volume.

Une observation déjà choisie conserve sa clé pendant les sondes de revalidation, même si la préférence future change entre-temps. La sonde reçoit séparément la clé précédente lorsque l'enveloppe initiale omettait project_id ; sa demande originale, son canon, son admission_id et le reçu restent inchangés. Le coeur d'observation est unique et n'accepte aucune dérive du mapping, des droits ou de l'époque. Un projet unique ou encore absent n'utilise pas la préférence ; une préférence périmée d'un autre projet ne bloque donc pas sa sélection ou sa préparation neuve.

## C19 — contrat de la vue de source, avant intégration

État : conception éprouvée en prototype, intégration non qualifiée.
FR-019 reste entier ; T174 puis T030/T031/T032 et le gate T033 restent ouverts.

### Demande et autorité

La vue est demandée explicitement. Posture effective discovery et commit exact
expected_base_commit obligatoires ; aucune conversion implicite de shared ou
development. Les pairs doivent négocier une capacité de vue avant toute
opération correspondante. Un pair ancien refuse cette demande avant écriture.
La préparation origine doit importer les objets du commit sans créer un
linked checkout ni déplacer HEAD. Si le port origine ne sait pas le faire,
refuser avec la capacité manquante ; ne pas retomber sur PrepareProject
qui créerait un worktree.

La vue est extérieure au dépôt partagé et dérivée de la Task, de la commande
et du digest canonique. Le mapping source et sa révocation restent relus
strictement. La projection de droits remplace la racine de travail par la vue,
elle n'ajoute pas l'ancienne racine comme autorité de travail. Les droits
additionnels explicitement hérités gardent leur propre contrôle. Les outils
fournisseur restent réellement en lecture seule. Aucun droit n'est déduit
de chmod ni du manifeste.

Cette projection ne crée pas une garantie générale d'invisibilité de tous
les autres fichiers du serveur. Les politiques existantes readOnly et les
racines d'écriture ne démontrent pas cette garantie. Tester le remplacement
de la racine et l'absence d'écriture ; ne pas promettre une isolation de lecture
que le fournisseur ne prouve pas. Le contexte de mission nomme le commit exact,
le dossier de vue et la portée partielle, pour ne pas confondre A et HEAD B.

### Sélection et lecture

Réutiliser la sélection compacte207, en séparant le choix pur des fichiers
de la configuration qui protège les worktrees Git. Arbre borné à 8 Mio,
diagnostic Git à 1 Mio ; des bornes d'octets et d'entrées de vue sont fixées
et testées avant activation. Lire les blobs exacts avec un seul cat-file batch
dont les entrées et sorties sont drainées simultanément, en mémoire bornée.
Aucun filtre, export-ignore, export-subst, hook, remplacement d'objet Git
ou transport réseau implicite. Aucun processus par fichier.
Complexité visée O(n+B), n entrées, B octets effectivement lus ; mémoire bornée
par l'arbre, le manifeste et des tampons de flux, pas par B.

Valider les chemins avant création, les modes et types Git, les collisions
du système de fichiers cible, les liens internes et les chemins requis.
Refuser sous-module, nom non représentable, lien sortant ou cyclique.
Les liens internes pendants sont explicitement signalés ; ils ne prouvent
pas que leur cible existe. Le rapport et le retrait ne suivent pas les liens.

L'agent reçoit un résumé borné de vue partielle et une carte lisible de tous
les chemins omis, avec motif et commit. Une copie de carte dans la vue n'est
jamais l'autorité ; son nom réservé ne peut pas écraser un chemin Git.
L'ancre d'intégrité privée reste hors de la vue. Le contexte précise que
cette vue n'est pas un dépôt Git ; git log/show/diff ne sont pas promis
depuis ce dossier. Les fonctions diff/PR du programme C35 restent distinctes.

Une omission ne devient pas un fichier absent du commit. L'accès à un fichier
omis doit rester possible par une lecture native bornée liée au manifeste :
seulement chemin et blob exact de ce commit, jamais un SHA libre ou HEAD.
Réutiliser le lecteur de blobs et la règle de non-écrasement C18, sans appeler
checkout-index dans un dossier sans Git. L'opération de matérialisation appartient
à l'infrastructure et garde les contrôles de Task/instance/commande/droits ;
elle ne donne pas un outil d'écriture arbitraire à l'agent discovery.
Un fichier déjà présent avec les octets attendus peut être constaté, mais
une divergence n'est jamais remplacée. Le manifeste complet permet d'auditer
un fichier omis devenu présent contre le même blob, sans changer de commit.

Ne pas promettre la présence future des objets Git sans ancre contre GC.
Le contrat initial ne crée aucune référence dans le dépôt partagé : si un blob
omis a disparu, rendre son indisponibilité exacte et conserver les fichiers
déjà exportés. L'agent peut demander la remise à disposition du commit publié
à son responsable ; aucune substitution de base ou nouvelle mission automatique.

### Reçu, coupure et concurrence

Réutiliser le reçu de préparation et Task.workspace, avec une variante explicite
de vue. Ne pas sérialiser une vue comme strategy=worktree. Le reçu lie identité
worker, Task, commande, canon, mapping, base et empreinte du manifeste privé.
Le manifeste complet ne contient aucun chemin libre vers une autre source.
La carte publique n'est pas une preuve d'intégrité.

Durcir l'intention avant extraction. Publication seulement après vérification
complète. Après coupure, le même tuple/digest vise le même chemin. Une vue
publiée avec preuve privée complète et audit exact est adoptée au rejeu.
Une vue incomplète, sans preuve ou divergente est conservée et refusée ;
elle n'est ni supprimée ni recréée automatiquement. Le guidage nomme la
vérification manquante et la décision ciblée à demander à l'utilisateur.
Un autre digest sur la même identité reste CommandMismatch.

La vue ne prend pas de garde Shared et ne demande aucun shared_refresh.
Une préparation Shared ou un descendant actif n'est donc pas une raison
de refuser sa lecture. L'import d'objets et la lecture doivent conserver les
gardes Git déjà nécessaires ; ne pas retirer un verrou d'une tâche étrangère.
Une disparition d'objet pendant la lecture donne un refus explicite, pas B.
Inspect168 reste sans matérialisation avant la Task durable.
Start et Release refont l'audit privé et les droits sur la vue ; ils ne cherchent
pas un HEAD ou une branche Git dans cette vue.

### Rapport, ancien lecteur et retrait

Le rapport est un contrôle de fichiers/types/modes/octets contre les blobs
du manifeste privé. Les fichiers requis initialement doivent être présents ;
un fichier omis présent est accepté seulement s'il correspond au blob attendu.
Toute addition non réservée, suppression requise ou divergence est signalée.
Aucun dirty=false n'est dérivé d'un git status qui ne s'applique pas.

Une nouvelle variante persistée est illisible aux anciens lecteurs stricts,
y compris après fin de Task. Le support minimal doit être contrôlé avant
Prepare de mise à jour ou de retour, sur maître et workers, et dans le parcours
opérateur. Réutiliser le contrôle de candidat existant, avec une déclaration
fermée des formats lus et un inventaire exact des formats conservés.
Un candidat ancien sans déclaration ne prouve pas sa compatibilité.
La barrière avant libération doit empêcher tout nouveau reçu C19 pendant une
bascule qui pourrait encore revenir automatiquement. Après premiers reçus,
aucun retour ancien non compatible ni restauration d'une sauvegarde ancienne
comme moyen d'effacer les tâches admises. Ce contrôle reste à implémenter
et à éprouver : aucune activation C19 avant cette preuve.

Capacité absente, format incompatible, source absente et audit incomplet rendent
un refus guidé : portée cette préparation ou cette vue, cause observée, Task/clé
conservées, action de vérification précise et décision humaine nécessaire.
Les autres travaux suivent leur propre état. Aucun arrêt global n'est déduit
d'un refus local.

Retrait après clôture native et contrôle complet des lecteurs/processus.
Ne pas suivre un lien, ne pas effacer une vue d'issue inconnue. Réutiliser le
cycle de vie de préparation, sans nouveau magasin de tâches ou service.


## T180 — absence d’observation et changement de version
Un cache vide, une donnée périmée ou une capacité non négociée ne prouvent pas l’arrêt d’un processus de sonde antérieur. Le prochain conseil rend explicite le contrôle des marqueurs privés et des processus de cette sonde par le responsable avant tout changement de version. Aucun effacement de marqueur, signal, refresh, retour ou mission n’est déclenché par la projection. Le statut, les contraintes, le TTL et les modèles restent identiques. Le contrôle est limité à cette observation ; les autres travaux autorisés suivent leur propre état. Couvrir la vraie projection périmée et non négociée ainsi que le Hello qui vide le cache, puis la republication de la contrainte. Un état inconnu ne devient jamais StopUnconfirmed sans preuve.
