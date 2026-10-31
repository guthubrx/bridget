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

### C02–C05 — faits de catalogue lisibles et datés
Le cache maître conserve la date UTC réelle de réception avec son Instant
monotone existant. La projection calcule une fois son heure murale et son
Instant pour toute la vue ; l'expiration conserve ses deux contrôles existants.
Publier séparément date/âge de la source et date/âge de réception. expires_at
est une estimation locale, seulement pour reported_models, à partir du budget
restant le plus court ; le statut reste l'autorité de fraîcheur, pas le cache
réseau fournisseur qui demeure non attesté.

Chaque liste de modèles reste contenue dans sa connexion, qui porte déjà
machine/moteur/nonce opaque/date. Ne pas détacher les lignes de ce contexte
dans un client. Répéter ces champs par ligne n'ajoute aucune autorité et peut
faire dépasser la borne de réponse ; le test C03 doit prouver le lien existant
avec le même nom de modèle sur deux serveurs.
Aucun nouveau fait de compte, secret ou époque d'exécution exposé. Les efforts déclarés sont comparés aux efforts annoncés :
announced/not_announced/not_configured/outside_policy, jamais accès autorisé.
La liste annoncée et les candidats admis ne sont pas réécrits.

Décrire les signaux réellement observés par220 : métadonnées auth/config/
programme, révision de définition et reconnexion worker ; sans signal de fichier
du login externe, seule la lecture périodique peut constater le changement.
Objectif120s, pas un délai garanti pour toutes les connexions séquentielles.
Aucun nouveau processus, signal, compte, sonde ou format de trame.

Des profils peuvent partager le même compte/home intentionnellement. Ne pas
ajouter un refus d'unicité de home : plusieurs connexions ne promettent pas
plusieurs comptes. L'ajout d'un compte isolé reste explicite et officiel ; aucun
identifiant principal copié/écrasé. C06 et découverte Claude restent à qualifier.


Complément de garde : aucune nouvelle observation, même Observed, ne libère StopUnconfirmed. Le worker retire cette contrainte uniquement après récolte confirmée du groupe détenu ou récupération complète des marqueurs. Les erreurs ultérieures de configuration ne remplacent pas le premier fait d'arrêt incertain. Le cache worker reste borné à32 clés, anciennes gardes comprises ; remplacer une clé existante ne consomme pas de créneau. Une politique indisponible ou désactivée ferme le catalogue avec un conseil de contrôle des marqueurs ; l'absence de projection ne constate aucun arrêt.



### Plancher conservateur de formats — décision de raccordement

Un inventaire absent ne sera pas remplacé par un inventaire vide.
Le contrôle séparé formats impose, au minimum, tous les formats que le
contrôleur installé peut produire. Ce plancher s'applique même à un store
custom non découvert ; il ne dépend pas d'un scan de répertoires.
C'est plus conservateur qu'un inventaire exact et évite une nouvelle autorité
de découverte des racines. Le rapport209 reste inchangé.

Le rapport de candidat est lié à un nonce et à l'empreinte du fichier exécuté.
Le contrôleur installé valide sa forme, ses limites et la couverture de son
plancher. Des identifiants de lecture additifs ne donnent aucun droit de mission.
Absence de déclaration ou de vérification : refus avant Prepare et bascule,
avec conservation des données. Vérifier séparément le binaire de retour.

La transition C19 nécessite une étape lecteur avant toute nouvelle écriture :
capacité de relire, rapporter et fermer les données de vue, sans admission
de vue activée. Une image de retour compatible doit être qualifiée avant
que le parcours complet puisse produire ses premiers reçus. Le seul numéro
0.6.5, la présence d'un fichier ou une sauvegarde CE08 ne le prouvent pas.
Aucune restauration ancienne ne sert à faire disparaître ces reçus.
Ce raccordement et cette transition restent à tester ; aucun gate levé.

### C19 : intention et publication privées
La clé de stockage est SHA256 du tuple worker, commande et empreinte du canon de
préparation. L'intention lie ce tuple, la source admise, le dossier de données
privé déterministe et le manifeste complet. Elle est créée exclusivement,
synchronisée avant l'export et ne peut pas être remplacée au rejeu.
La publication privée est séparée. Elle lie l'empreinte des octets de l'intention,
le périphérique et l'inode de la vue auditée. Elle est écrite seulement après
synchronisation et audit. Au rejeu, intention, publication, tuple et racines
attendues sont comparés avant un nouvel audit. Sans publication complète,
une vue existante reste conservée et refusée ; aucune suppression/recréation.
Les plans et publications appartiennent au budget existant de WorkerStore.
Le nombre exact d'octets réservés est calculé avant écriture, sous son verrou.
La vue reste sur le volume du projet ; les preuves sont hors de la vue et de
toutes les racines de travail exposées. Ces règles n'établissent pas une
interdiction de lecture au niveau du système pour deux processus du même UID.

### C19 : possession du processus de préparation
L'export de fichiers et ses appels système s'exécutent dans un helper du même
binaire, sous managed-bootstrap. Le marqueur est durable avant RELEASE.
Les sous-processus Git héritent de son groupe. Aucun lancement fournisseur,
aucune donnée d'authentification et aucun shell. L'environnement ne garde que
PATH, HOME et TMPDIR de la connexion, puis LC_ALL=C.
Le verrou par tuple et tout marqueur existant empêchent un deuxième lancement.
La fin du leader ne prouve pas la fin des descendants. Une preuve de groupe
absent est requise avant retrait du marqueur ; un leader absent avec groupe
encore présent garde un arrêt non confirmé, sans signal à une identité inconnue.
Le sous-mode interne exige son canal géré et l'identité du reçu. Sans ce canal,
il refuse avant lecture des chemins de preuve. Une fin sans publication privée
ne devient jamais un export réussi. Le budget et le reçu de WorkerStore
doivent compter les fichiers privés avant le premier lancement.

### C19 : réservation dans le reçu existant
WorkspacePreparationReceipt reçoit un champ optionnel fermé commit_view,
omis à None dans les reçus historiques : key, intent_digest et reserved_bytes.
La clé est recalculée depuis worker/commande/digest du reçu. L'empreinte
d'intention empêche de changer les chemins ou le manifeste sous le même canon.
Le budget réserve avant écriture les octets de l'intention, de la publication
et du marqueur, temporaire atomique compris. L'inventaire ajoute la différence
entre réservation et fichiers compagnons déjà présents, sans les compter deux
fois. Les orphelins restent comptés par l'inventaire privé existant.
Ces fichiers sont des compagnons d'un reçu, pas une seconde tâche ou un second
store. Un groupe orphelin de fichiers de preuve sans reçu compte une entrée
conservée ; les noms non reconnus comptent chacun une entrée.
Un reçu incomplet conserve sa réservation après redémarrage. Aucun ancien
reçu n'est réécrit pour lui ajouter le champ. L'écriture de ce format n'est
pas encore activée dans le parcours d'admission ; le plancher de lecture doit
être qualifié avant sa première utilisation réelle.

### C19 : ancre de WorkspaceRecord et rapport
Le champ optionnel commit_view du WorkspaceRecord contient seulement le
répertoire privé de preuves, la liaison worker/commande/canon et le digest
d'intention. Il est omis dans les worktrees historiques. La vue n'emprunte
ni branche ni HEAD : les champs de faits Git restent absents.
Pour une vue, toplevel désigne le cwd source admis, pas un HEAD à examiner.
Le rapport retrouve l'intention depuis le store privé, compare base, chemins,
liaison et digest, confirme l'arrêt du groupe d'export puis refait l'audit.
Il fonctionne sans source Git disponible. Une divergence ou une preuve
incomplète refuse le rapport et conserve la vue ; aucun état propre déduit
d'une erreur de lecture. Le statut doit distinguer ce rapport de fichiers
d'un rapport Git avant l'admission publique.
La comparaison d'ancre inclut ces champs privés ; changer la preuve sous le
même path/base ne permet aucune adoption. La création et l'écriture de ces
champs demeurent fermées au parcours public pendant l'implémentation du lecteur.

### C19 : preuve d'arrêt du processus source pour l'annulation
Le reçu de préparation conserve aussi source_marker (ManagedMarker existant)
et source_stopped. Le marqueur complet est enregistré dans ce reçu avant
RELEASE par un rappel d'autorisation ; un échec abandonne le bootstrap et
conserve son Child, sans RELEASE. Le marqueur fichier du superviseur peut être
retiré après récolte, mais son absence ne remplace jamais la preuve du reçu.

La clôture C201 examine les préparations source de la même Task avant de
publier StartFenced. Un marker dans le reçu impose de constater le groupe
absent, puis de durcir source_stopped. Tant qu'il est vivant ou indécidable,
refuser la clôture sans signal à un PID déclaré. Sans marqueur de reçu, vérifier
aussi tout marqueur fichier d'un bootstrap avant libération. Après la preuve
de clôture, aucune nouvelle autorisation RELEASE de ce reçu n'est possible.
Un reçu Fence déjà durable empêche une nouvelle réservation source de cette
Task. Ces phases réutilisent le même reçu et son budget de croissance de64Kio,
sans tâche technique fictive ni second registre de processus.

### C19 : propriétaire en mémoire du worker
WorkerRuntime conserve les helpers source dans une collection distincte des
wrappers fournisseurs, liée au reçu de préparation. Elle ne crée ni Fleet ni
Task supplémentaire. Un même tuple reprend le contrôle existant ; un autre
digest refuse. Une sortie de groupe constatée est durcie dans le reçu avant
de libérer ce propriétaire. Un arrêt incertain conserve le Child et ses preuves.
Le tick existant peut récolter ces seuls helpers, sans relancer un export.
Une demande native de clôture de la même Task peut arrêter son helper détenu ;
un PID seulement déclaré par le réseau n'est jamais signalé.
Le raccordement public et la récupération du contrôle après perte du worker
restent des gates séparées : la collection en mémoire seule ne suffit pas.

### C19 : contrôle d'un groupe retrouvé après redémarrage
La clôture retrouve les seuls marqueurs de la Task dans les reçus privés
validés. Elle n'envoie TERM qu'après comparaison fraîche de la naissance du
leader avec celle du reçu. Si cette identité n'est plus vérifiable alors que
le groupe existe, refuser et conserver les preuves, sans signal à un autre PID.
Le contrôle reprend le signal unique déjà utilisé par le worker ; la fin
n'est confirmée qu'après disparition du groupe. Une reprise de préparation
ne relance jamais un helper dont le reçu contient déjà un marqueur.

Le signal source vise aussi la commande de préparation de la demande de
clôture, pas le seul identifiant Task. Une autre tentative de la même Task
ne signale pas ce groupe ; le reçu de groupe encore vivant empêche néanmoins
de publier une clôture qui prétendrait que toutes les sources sont arrêtées.


### C19 : relecture des droits et du dossier d'exécution
La relecture d'une vue conservée repart du mapping vivant et du reçu de
préparation du store de ce worker. Elle exige la même commande, le même
canon, une source d'export arrêtée, la même base et une posture discovery.
La référence privée ne peut désigner qu'un manifeste du répertoire de ce
store ; un chemin reçu dans WorkspaceRecord ne donne aucune autorité de
lecture ailleurs. Une réparation de worktree ne s'applique pas à une vue.

La racine et le cwd d'exécution sont déduits du décalage Git du manifeste,
en retirant le décalage du cwd source par rapport au mapping. Le nouveau
mapping remplace sa racine distante, sans ajouter l'ancienne ni modifier
un additional_root explicitement hérité. Une incohérence de sous-dossier
refuse avant lancement. La vérification des droits et de la définition
fournisseur reste celle du worker, sur cette vue ; aucune lecture de HEAD
n'est faite dans l'export sans .git.


### C19 : compte rendu propriétaire
Le résultat natif nomme une vue de lecture, le SHA complet et le dossier
conservé. Il rend le dernier contrôle de fichiers, jamais une branche, un
nombre de commits ou un worktree inventés. Un rapport absent ou des faits
Git contradictoires restent indisponibles. Le format historique des comptes
rendus Worktree reste inchangé. Le texte n'autorise pas le retrait du dossier.


### C19 : processus de lecture d'un fichier omis
Réutiliser le propriétaire ViewJob, son bootstrap, son verrou par intention et
son marqueur avant RELEASE. Une requête fermée de matérialisation lie la
commande complète de l'instance utilisatrice, un numéro d'opération positif
et un seul chemin du manifeste. L'identité de processus est dérivée de ce
tuple ; aucun SHA, commande Git ou chemin source libre n'est admis.
Le helper exige son canal géré, compare cette identité, retrouve la publication
privée puis matérialise le blob exact sans écrasement. Toutes ses E/S et son
Git restent dans le groupe possédé et sous la même échéance que l'export.

Le raccordement au contrôleur de mission doit encore durcir cette requête
et son marqueur dans le reçu avant RELEASE, revalider l'instance/mission et
fermer l'opération lors d'un Cancel ou d'une reprise. Une identité dérivée
seule ne remplace pas ces gardes : aucune API de matérialisation publique ni
aucun nouveau plancher lecteur ne sera activé depuis le seul helper.


### C19 : autorisation durable d'une matérialisation
Le même WorkspacePreparationReceipt conserve au plus une matérialisation
courante : identité complète de résultat (mission/ronde/exécution/instance),
requête fermée, marqueur possédé, arrêt et disponibilité constatés.
Une nouvelle opération exige une commande Released/NativeReady non expirée
et son slot de résultat encore ouvert, non terminal et sans disposition.
Elle conserve le binding et le manifeste de la vue ; pas de seconde Task.

Le numéro d'opération croît dans ce reçu. Tant que le processus précédent
n'est pas arrêté, seule la même opération peut être retrouvée ; un autre
chemin reste occupé. Le marqueur est durci avant RELEASE après relecture de
la même exécution active. Un Cancel ou un StartFenced ne peut pas ignorer ce
groupe, même si le sidecar a disparu. La disponibilité true exige un contrôle
exact après disparition du groupe ; false n'est pas une preuve d'absence.
Un résultat non retrouvé reste inconnu. Aucun parcours public n'est ouvert
par cette préparation du contrôleur.


Le budget garde une croissance de métadonnées réservée tant que l'arrêt ou
le constat de disponibilité ne sont pas durcis. La maintenance conserve
les identifiants de signal déjà émis pour ces opérations. Le Reconcile
terminal, le rapport de repli et la collecte des commandes ne peuvent pas
masquer une matérialisation encore détenue. Les contrôles par commande sont
indexés par la clé de préparation ; la maintenance fait un seul parcours
borné des reçus, sans parcours global pour chaque commande terminale.


Le propriétaire runtime conserve aussi l'exécution et la requête du fichier
jusqu'au constat durable. Avant réservation puis avant RELEASE, il relit
mapping, définition et droits projetés sur la vue, sans retirer la garde
d'admission de inspect_candidate. Une réponse perdue avec marqueur ne relance
pas de helper ; un résultat de fichier exact peut être retrouvé après arrêt.
Le tick finalise le même reçu. Le signal natif Cancel reçu dans le canal
existant interrompt le helper possédé, sans attendre une nouvelle mission.
L'annulation du fichier est distincte d'un dépassement de délai.


Une lecture de fichier déjà terminée donne un nouveau contrôle borné lors
d'un prochain appel ; la disponibilité mémorisée n'est pas annoncée comme
preuve actuelle. Une même opération encore en cours reste retrouvée sous le
même numéro. Les données exactes déjà présentes ne sont jamais réécrites.


Un bootstrap abandonné peut laisser un marqueur alors que son arrêt est
déjà durci dans le reçu. Avant une opération distincte, le contrôleur peut
retirer ce seul marqueur privé, après comparaison complète et contrôle du
groupe. Un groupe vivant de même naissance, une identité inconnue ou un
marqueur divergent refuse le retrait. Aucun signal ni retrait de données
n'est déclenché par ce rangement. Le reçu arrêté est conservé ; la nouvelle
séquence n'est durcie qu'après rangement, sans perdre l'identité précédente.
Un PID réutilisé avec une autre naissance n'est jamais signalé.

### C19 : activité de mission et annulation d'un fichier

Le délai de commande borne le bootstrap avant Release, pas la mission déjà
Released/NativeReady. Une lecture de fichier exige le slot de résultat exact,
non terminal, ouvert et sans disposition, puis les droits, le mapping et la
définition courants. Le délai initial reste inchangé ; aucune prolongation
de mission n'est déduite de cette correction.

Avant réservation et juste avant l'autorisation durable du helper, relire le
signal d'annulation natif de la commande complète. Une annulation déjà reçue
avant la demande refuse sans nouveau reçu ni processus. Un mutex d'annulation
illisible donne StoreUnavailable, jamais une annulation supposée. Si ce constat
arrive avec un helper déjà détenu, tenter son arrêt sans oublier le Child ni
son marqueur si l'arrêt reste incertain. Cette borne de lecture reste distincte
de la décision d'annulation du fournisseur et de sa preuve finale d'arrêt.

### C19 : verrou du contrôleur et marqueur durable distincts

Le verrou flock sérialise les opérations d'un tuple. Sa garde exécute LOCK_UN
au départ du propriétaire, au lieu de dépendre de la dernière fermeture d'une
description héritée par un fork concurrent avant exec. CLOEXEC reste posé.
Ce retrait local ne confirme aucun arrêt. Un marqueur privé présent, même
sans propriétaire en mémoire ou sans verrou, interdit encore tout nouveau
helper tant que le reçu d'arrêt durable n'a pas été établi puis rangé.

### C19 : retour à la boucle worker pendant une omission

Le démarrage rend un jeton interne lié à la liaison privée, à l'exécution et à
la requête de fichier. Il ne copie pas le manifeste et n'est pas un RPC ni une
preuve d'autorité. Le futur port conserve ce jeton et revient à chaque tick ;
aucun parcours de production n'attend en boucle la fin du helper.
Un poll en cours ne recharge pas le manifeste et ne refait pas l'audit de la
vue. Un autre contrôle de commande peut être traité pendant ce temps.
La collecte existante surveille aussi le slot et le signal d'annulation,
même après disparition du lecteur. Slot fermé ou preuve illisible : demander
l'arrêt du helper, conserver son propriétaire jusqu'au constat d'arrêt, sans
prétendre qu'une annulation utilisateur a été reçue. Un poll périmé ou étranger
ne rend pas le résultat d'une nouvelle requête et ne signale pas son helper.
Une complétion déjà enregistrée par la collecte n'est pas réinterprétée comme
un succès par le poll. La relecture des droits avant remise et les audits de
complétion restent requis ; ces audits sont synchrones et leur coût doit être
qualifié avant le raccordement public. Le démarrage reste borné par le
bootstrap géré existant, sans file ni ordonnanceur supplémentaire.

### C19 : avant READY, absence de marqueur et clôture native

Le reçu enregistre source_attempted, ou attempted pour une matérialisation,
avant tout spawn. Le constructeur de production start_registered exige ce
callback ; le raccourci sans enregistrement est cfg(test) seulement.
Une réservation jamais tentée et un lancement tenté sans READY/marqueur ne
sont donc plus confondus. Au redémarrage, attempted avec arrêt non durci et
sans marqueur reste StopUnconfirmed ; aucun nouveau helper n'est lancé.

L'arrêt observé du Child possédé, ou un échec de spawn sans Child rendu,
permet au contrôleur courant de durcir l'arrêt. Ce chemin n'est pas utilisé
par un lecteur qui retrouve seulement un reçu ou une absence de sidecar.
Le résultat inconnu d'une matérialisation tentée sans marqueur ne se complète
pas depuis cette seule absence. Marqueur présent avec attempted faux :
refus de reçu incohérent. Les worktrees historiques gardent leurs octets.

check_view_native_closure contrôle sous la transition existante les commandes
de la Task, leurs preuves d'arrêt ou le fence de non-naissance, et la disposition
finale des résultats. « Terminal » seul n'est pas une preuve d'arrêt.
Annuler un slot contenant un résultat ne vaut pas acquittement de ce résultat.
Les commandes et compacts de slot restent conservés pendant qu'une vue existe,
même après sept jours ; un tombstone de retrait devra ensuite lever ce pin.
La garde ne lit pas les fichiers de la vue, ne vérifie pas ses lecteurs externes
et n'autorise aucune suppression à elle seule.

Une preuve d'arrêt déjà durcie n'est pas annulée parce que le numéro de groupe
a été réutilisé par un processus étranger. La reprise d'une publication close
ne lui envoie aucun signal et continue de vérifier l'intégrité de la vue.

### C19 — conservation sans privilège, clôture et petite référence
Choix utilisateur confirmé le31octobre : contrôle sous les droits du worker ;
inventaire incomplet ou usage trouvé => conserver les fichiers. Aucun contrôle
administratif ajouté. Cette décision ne déduit aucun arrêt depuis un inventaire
incomplet.

Le reçu existant porte closed, monotone et omis si false. Seule la clôture native
prouvée sous le verrou permet true ; le record passe Kept avant la réduction.
Aucun export, matérialisation, Release ou fin tardive ne réouvre cette vue.
Le port propriétaire qui autorise cette transition reste à raccorder/qualifier.

La carte de données inclut le manifeste entier. L'intention privée peut ensuite
devenir closed_ref_v1 : binding/source/data_root et hash SHA256 de l'intention
canonique originale. La publication privée atteste encore hash/dev/inode.
La carte reste non fiable : aucune racine ou commande n'en est tirée. Le lecteur
reconstruit puis exige le hash, les formes fermées, les bornes et l'audit exact.
Carte absente/altérée ou répertoire substitué : refus, jamais recréation.

L'ordre durable est closed avec réservation pleine -> petite preuve atomique
sous lease -> réservation réduite. Les temporaires privés et orphelins restent
comptés. La réservation initiale couvre aussi le cas où la référence d'un tout
petit manifeste serait plus longue. L'histoire de commandes/slots n'est plus
épinglée après la clôture prouvée ; le reçu workspace sans expiration conserve
l'interdit de recréation et le rapport sans Git source.

Aucune écriture ou déplacement de fichier de vue pendant la réduction. Avant
un éventuel retrait physique, durcir à nouveau le plan complet privé requis
par sa reprise. Retrait physique, lecteur de retour, ancien pair et usage natif
restent à qualifier ; aucune admission C19 ni format de stockage activé ici.

### C19 — préparation source en plusieurs pas
La préparation conserve un jeton interne ViewSourceRead portant seulement la
liaison privée. Begin réserve avant spawn, refuse une annulation déjà observée,
puis rend le jeton après READY/RELEASE bornés. Poll lit les petits reçus, observe
l'annulation et revient sans attendre le helper ; il ne recharge ni manifeste
ni fichiers tant que le helper travaille. Le constructeur synchrone est limité
aux fixtures de test. Le raccordement public devra garder le jeton jusqu'au
prochain tick et refaire les droits avant remise de Prepared.

Le reçu existant garde source_failure facultatif, code fermé après arrêt
confirmé. Le collecteur peut donc fermer son Child sans perdre un échec quand
le lecteur revient plus tard. Une réussite exige encore publication et audit ;
aucun processus ne repart depuis un reçu attempted d'issue inconnue.
La clôture close interdit cette préparation et ses résultats tardifs.
Pas de thread/store/capacité supplémentaire, aucun Start fournisseur.

### C19 — ancre originale et reprise fencée
Une commande de reprise B ne crée pas une seconde vue ni réservation. Réutiliser
BindProof198 et original_workspace_at : le reçu B sans record/réservation garde
son lien durable vers A, après fence exact. La requête B doit correspondre au
canon198 et à la même Task, base, source, droits et placement. La preuve privée
de données reste liée à A ; une vue close ne peut être réouverte par B.

Le port de fichier authentifie l'exécution/slot B et remonte cette chaîne avant
de charger le manifeste A. La requête de helper porte l'identité complète B,
séquence et chemin ; ce tuple n'est pas à lui seul une autorité de reprise.
Begin/Release contrôlent la liaison durable et les droits effectifs.
Fin de helper peut remonter la même ancre après fin de la mission, sans relance.
Cancel B ne cible que son helper B, avec déduplication par identité d'opération,
pas le vieux numéro de commande A ni une opération suivante.

Les reçus et formats198restent inchangés. L'ancre A porte seule les métadonnées,
la publication et le budget des données. Gardes négatives requises : B sans
preuve, mauvais canon/base/Task/worker/slot, ancre close, ancien Cancel A,
répétition de Cancel B et autre opération. Créations publiques restent fermées.


### C19 — conservation native négociée
L'opération ConserveCommitView porte seulement l'empreinte du canon de préparation. La commande externe est l'identité de préparation originale (instance vide, génération0), déjà liée au worker et à la Task. Aucun chemin, résultat, signal, commande de shell ou autorité de fichier n'est reçu. Le worker relit la réservation par cette liaison et sa preuve privée ; il exige la clôture de toutes les commandes/slots de la Task avant freeze. La réduction du manifeste reste séparée et garde sa réservation pleine tant qu’elle n’est pas achevée. La réponse répète la même empreinte, après preuve durable. Répéter constate la même conservation sans recréation.
Négocier WorkspaceViewV1 sur le lien courant ; ancien maître/worker : refus avant envoi, jamais conversion en Cancel/Start. Cette capacité de lecteur/conservation ne lève aucune garde de création des vues. Le port maître propriétaire sera raccordé seulement à la clôture explicite d'une Task non continuable, pas à une lecture de statut ou au seul ACK d'un résultat. Aucun retrait physique : fichiers conservés selon le choix utilisateur. L'inventaire de lecteurs externes et les gates de formats/retour restent distincts.

La conservation native ne parcourt pas les fichiers de données et ne réduit pas le manifeste : aucun audit de512Mio dans la boucle de contrôle. Elle rend seulement la clôture durable et garde les fichiers/réservations. Les rapports et la réduction restent des opérations distinctes à qualifier sous leur budget.

Le raccordement maître réserve le claim de rapport existant dans cancel_tree après arrêts/gardes/rapport et dispositions des descendants. Il garde cleanup_done=false tant que l'ACK exact de conservation n'est pas acquis. Après l'ACK, un CAS court revalide Task/mission/ronde/liaison/non-continuabilité avant le bit final. Un échec garde les fichiers et l'erreur métier d'origine ; aucun marqueur ni tâche neuve. Les espaces Git suivent leur chemin historique.

### C19 — contrôle de lecteur avant bascule et retour

L'updater interroge le candidat sans service, avec un nonce neuf et son empreinte
d'image. Le binaire déjà installé valide ce rapport avec son plancher compilé,
avant tout Prepare, arrêt ou changement du lien. Le candidat ne choisit pas le
plancher exigé. Une commande absente ou un rapport invalide refuse la bascule ;
aucun inventaire absent ne devient un magasin vide. Le bootstrap du lecteur A
reste une livraison explicitement qualifiée tant que l'ancien installé n'expose
pas ce port. Ne pas autoriser une exception depuis le seul numéro de version.

Un retour automatique et une récupération après coupure vérifient l'ancienne
image avec le validateur de la nouvelle image, car celle-ci peut avoir persisté
de nouveaux formats. Ce contrôle précède aussi Prepare en récupération. Si la
preuve manque, garder le lien et l'intention de reprise, rendre la cause fermée
et demander une image compatible ; ne pas effacer les données pour rétrograder.

La commande native formats réutilise le runner borné de clients (sortie32Kio, délai20s), puis le
validateur limite le rapport à8Kio. Aucun réseau, store neuf, scan de secrets,
mission ou processus de modèle. Ce contrôle prouve le contrat de formats
déclaré et validé ; les tests de lecteur de tous les états restent une gate
distincte avant ajout de workspace.commit_view au plancher. Le plancher reste
workspace.worktree.v1 dans cette étape. L'installateur reste à raccorder et à
qualifier ; cette étape seule n'ouvre aucune création C19.

### C19 — préparation typée entre export et Prepared
Étape interne avant raccordement public : le begin reçoit un plan ViewIntent
du constructeur privé, jamais un chemin fourni par le lecteur. L'appelant
doit avoir authentifié la commande du maître ; le jeton mémoire ne crée
aucune autorité. Vérifier canon/worker/commande, mapping, mode discovery,
définition et plafond de droits avant la réservation et le helper.
Le begin ne crée aucun PreparedRuntime ni token utilisable par Start.

Le poll conserve le petit jeton de source. Avant publication de Prepared,
relire le registre et le mapping, comparer les digests initiaux, vérifier le
rapport privé et remapper les droits vers le seul chemin attesté de la vue.
Aucun shared_refresh ni comparaison avec le HEAD actuel du dossier partagé.
L'échéance de construction est120s ; un délai dépassé signale seulement le
helper possédé et le conserve jusqu'à preuve d'arrêt. Un Prepared final garde
la durée historique30s. Une répétition avant son expiration rend le même
token, sans réexport ni réécriture du reçu ; une expiration ne se transforme
pas en nouveau token. Une révocation laisse les données et reçus, sans Start.

Le plan de placement des données, la boucle de dispatch publique nonbloquante,
la négociation, le lecteur de retour et les preuves fournisseur restent des
gates distinctes. Ne pas présenter cette primitive interne comme une
admission disponible ni ouvrir les refus actuels de prepare_for/MCP.

La consommation d'un Prepared est contrôlée depuis le reçu de commande
existant, durci avant retrait de la table mémoire par Start. Aucun nouveau
tombstone n'est ajouté : la collecte garde déjà ces commandes tant qu'une
vue est ouverte ; une vue fermée refuse ensuite toute reprise. Begin et poll
refusent toute réémission sous la commande consommée. Le rejeu d'un token
encore disponible compare aussi le routage, la référence de vue et le cwd.

### C19 — possession légère des helpers
Le contrôleur garde la liaison typée worker/commande/canon avec le Child et
l'opération de fichier, pas une seconde copie du manifeste complet.
La preuve de fin relit le petit reçu privé par cette liaison. Un marqueur ou
une liaison différente reste un refus. L'autorisation du contenu et du
chemin reste faite avant RELEASE ; ne pas la déduire de ce cache de Child.
Une clôture tardive peut libérer son Child possédé sans recharger un manifeste
réduit ou altéré, seulement quand les champs de clôture et d'opération exacts
du reçu le prouvent. Aucun fichier de données n'est retiré par ce chemin.
Cette séparation prépare l'étape de planification gérée sans nouvel exécuteur.


### C19 — planification possédée avant lecture Git (révision 48)
Une graine privée fermée lie worker, commande, canon de préparation, racines,
base A et échéance absolue de 120 secondes. Elle ne contient aucun manifeste.
Le reçu natif existant réserve avant fork le pire cas des métadonnées
(manifeste 8 Mio, graine 16 Kio, publication et marqueurs). Les données
exportées restent hors du store de 16 Mio. Aucun second magasin de tâches.
Le même bootstrap possédé planifie, exporte et audite dans son processus.
Tentative durable avant fork, marqueur avant RELEASE. La graine et son délai
ne sont jamais remplacés au rejeu. Le contrôleur reste le seul écrivain du
reçu de préparation : après arrêt confirmé, il compare la graine et le
manifeste puis adopte son empreinte. Un crash sans preuve ne permet aucun
nouveau processus. Une graine expirée refuse la publication, même si les
fichiers ont été écrits ; ils restent conservés. La primitive demeure privée
jusqu'au raccord des contrôles de mapping, droits, retour et clôture.


C19 — audit différé (révision 52). Après arrêt durci de l'export, une seule
lecture d'intégrité par vue est possédée par le runtime worker, dans un thread
de lecture borné par l'audit existant. Elle ne reçoit ni store, ni commande de
mutation, ni processus fournisseur. Le worker ne joint le résultat que lorsque
le thread est fini. Il reste seul écrivain du reçu et contrôle encore liaison,
marqueurs, annulation et clôture avant publication. Un Cancel interdit la
publication ; il ne prétend pas interrompre une lecture système déjà engagée.
Le contrôleur garde cette lecture jusqu'à sa fin. Après crash, aucun effet de
thread à récupérer ; un nouvel audit est permis, jamais un nouvel export.
L'audit atteste l'instant observé, pas une immutabilité OS contre le même UID.
Le rapport et le contrôle avant Release doivent toujours détecter une altération.


C19 — échec avant publication (révision 54). La réservation précède toujours
la graine. Une réservation non tentée peut compléter uniquement la même graine
et son même délai après une coupure ; aucune preuve partielle n'est remplacée.
La conservation native peut clore une préparation non publiée : aucun export
tenté sans arrêt durci, aucun marqueur vivant, aucune commande ou remise active.
Elle garde tous les fichiers et un tombstone durable ; elle libère seulement
la réservation future, jamais les octets réellement présents. Une issue de
processus inconnue refuse cette clôture. Une expiration conserve sa cause
preparation_expired ; elle n'est ni une annulation humaine ni un délai Git.


C19 — précision conservation et concurrence (révision 58).
Les fichiers conservés sont les données, la graine, l'intention et les reçus.
Le marqueur de processus est transitoire : ManagedChild le retire après récolte
du groupe. Le reçu conserve son tuple PID/naissance/génération, puis l'arrêt
durci ; aucune clôture n'est déduite de la seule absence du marqueur.
Cancel et le CAS de publication sont ordonnés par le même mutex d'annulation,
pris après les lectures et gardé jusqu'au persist. Aucun audit sous ce mutex.
Le lecteur réseau ne prend pas le verrou de transition du store.
Une tentative sans marqueur et sans preuve d'arrêt reste volontairement non
clôturable automatiquement. Conserver fichiers et réservation, demander au
responsable une vérification ciblée de ce bootstrap. Ne pas créer de nouvelle
mission ni affirmer l'arrêt ; les autres travaux suivent leur propre état.
Les nouvelles variantes de cause font partie du format C19 : lecteur compatible
requis avant toute écriture publique et pour tout retour, sans exception vide.


C19 — placement natif (révision60).
Le créateur calcule une réserve .bridget-commit-views à côté de remote_root ;
aucun chemin de données fourni librement par l'agent. Le helper possédé vérifie
la base avant tout dossier, déduit le toplevel depuis le manifeste, puis exige
une réserve hors de tout le dépôt Git et sur le même volume. Parent canonique
possédé par le compte, non modifiable par groupe/autres ; réserve privée0700,
sans lien. Aucun chmod de dossier existant. Un mapping de sous-dossier dont le
voisin serait dans Git, un point de montage ou un parent inconnu est refusé,
sans repli ni réécriture du mapping. La place libre est un fait instantané,
jamais une réservation du volume ou une preuve de quota. ENOSPC/accès gardent
leurs causes. Une réserve native vide peut rester après une interruption ;
la réservation privée et le tombstone restent l'autorité du rejeu.


C19 — arbitrage de placement après observation réelle (révision63).
Le parent Cartae /home/moi/10.Scripts est root:root0755 ; le voisin direct
n'est donc pas créable sans privilège. Aucune permission n'est modifiée.
La graine native engage désormais native_placement_v1 : le helper choisit
le premier ancêtre canonique, possédé par le compte et non modifiable par
groupe/autres, sur le même volume et hors du toplevel Git. La réserve porte
toujours le nom fermé .bridget-commit-views et reste0700. Aucun chemin libre
fourni par le modèle, aucun repli de mapping/base/droits. Le manifeste lie
le chemin final choisi ; l'adoption vérifie son appartenance à cette politique.
Cette politique prend aussi en charge un mapping de sous-dossier, sans écrire
dans son Git parent. Aucun ancêtre admissible ou changement de volume : refus
nommé, fichiers gardés. L'export exige toujours la liaison privée et l'arrêt.
Fixture : le runtime a montré parent_mode504 (0770),uid1002,dev2307 ; son
dossier de test privé est maintenant créé0700. Les vrais dossiers existants
ne sont jamais changés pour faire passer un test.


C19 — budget après adoption (révision64).
Avant planification, réserver le pire cas du manifeste et de la graine.
Après arrêt confirmé et adoption exacte, réduire atomiquement au manifeste
connu, sa référence de clôture future, la graine et les marqueurs/publication.
Aucun budget inconnu de8Mio ne reste détenu pendant une revue achevée de préparer.
Les octets physiques restent comptés ; la transition est dans le reçu existant,
avant publication. Le rejeu ne modifie plus un budget déjà exact. Une autre
préparation peut être admise sans attendre la fin du modèle de la première.


C19 — capacité de création distincte (révision65).
WorkspaceViewV1 atteste la lecture/conservation ; elle n'autorise pas la création.
WorkspaceViewCreateV1 est négociée séparément après DaemonIdentityRequest,
avant Hello. Le lecteur A ne l'annonce ni ne l'accepte. Le créateur B exige
cette capacité avant tout Prepare/sonde contenant commit_view ; sans elle,
refus avant émission, sans repli Shared/Worktree. Aucun ancien pair ne reçoit
une variante inconnue. La borne de capacités devient12, sans doublon.
Le gate de lancement reste faux jusqu'au raccord complet et à sa qualification.
La conservation d'un échec réduit seulement le budget futur, jamais ne l'augmente.


C19 — diagnostic de création (révision67).
Une seule enum fermée CommitViewFailure est partagée entre helper, reçu et
trame CommitViewRejected. Aucun message Git libre, stderr, compte, chemin ou
variable d'environnement n'est projeté. Le maître n'accepte cette réponse
qu'avec WorkspaceViewCreateV1. Forme de préparation stricte : pas d'instance
active ni génération dans la réponse. Le code public et la clé restent liés
à la tâche existante. Échec de capacité de métadonnées, volume différent,
placement dans Git, placement indisponible, espace libre non observé et base
absente restent distincts. Un état inconnu ne devient pas disque plein.


C19 — propriétaire avant export (révision68).
Task.commit_view_prepare conserve la liaison worker/commande/canon avant
l'effet de préparation, même sans WorkspaceRecord. Champ absent des tâches
historiques. La liaison source originale est immuable pendant les reprises.
La conservation choisit cette liaison uniquement si le dossier n'a pas encore
de référence ; une référence contradictoire ou une autre stratégie refuse.
Sans rapport publié, l'état reste 'publication non attestée', pas 'dossier
absent'. cleanup_done exige l'ACK natif exact, jamais l'absence de dossier.
Le claim/CAS de rapport existant protège la clôture ; aucun second magasin.
La projection complète donne la preuve de préparation et le conseil utile,
sans inventer un chemin, une intégrité ou une permission de retrait.


C19 — coupure avant réception de Prepare (révision69).
Une Task peut déjà porter sa liaison alors qu'aucune réservation worker n'a
été créée. L'absence seule ne prouve rien. Le worker peut conserver ce cas
seulement avec le fence natif pré-démarrage exact et durable, aucune commande
ou remise active de cette tâche, aucune vue ni preuve compagne de cette liaison
(même un verrou vide compte), et aucun helper détenu en mémoire. Il écrit alors
un tombstone fermé dans le reçu existant : aucun export passé n'est inventé,
toute création ultérieure de cette commande reste refusée après collecte.
Un reçu manquant avec une preuve orpheline ou une naissance possible reste
inconnu ; ni effacement ni clôture par déduction.

C19 — préparation différée (révision71).
Réutilisation du contexte privé begin/poll, possédé par le runtime unique :
128 préparations au plus et huit corrélations par préparation et époque.
Le canon complet et la commande doivent rester identiques au rejeu ; les
corrélations ne renouvellent jamais l'échéance de la graine. La reconnexion
oublie les destinataires de l'ancien lien, conserve le helper et sa preuve,
puis accepte seulement le rejeu exact du maître pour répondre sur le nouveau.
Aucun thread ne possède le store ou l'autorité d'admission. Le contrôleur
réseau exige WorkspaceViewCreateV1 ; sa porte reste fermée dans ce lecteur.
Les autres commandes continuent de passer par le même runtime entre polls.

C19 — annulation et reconnexion (révision73).
Une reconnexion ne retire pas une annulation déjà reçue pour un export, une
préparation ou un audit encore détenu. Les identités hors de ces contextes
reprennent la vérification de leur reçu durable. Un duplicata arrivé après
l'échéance refuse avant de renouveler ses corrélations.

C19 — inspection préalable et création (révision74).
L'inspection automatique lit le commit A et le miroir local déjà configuré,
sans rafraîchir le checkout B, sans examiner sa propreté et sans créer de vue.
Une absence ou une erreur de lecture refuse ; aucun repli Shared ou Worktree.
La création différée suit seulement la Task durable et sa liaison. Son attente
maître est de 125 secondes pour la graine bornée à 120 secondes ; les autres
préparations conservent leur délai. Le refus corrélé conserve sa cause fermée.
Cette branche reste derrière la porte créateur désactivée. Le raccord direct
sans expected_base_commit, Start/Release et le retour restent à qualifier.

C19 — audit d'intégrité avant effet (révision76).
Start et Release consomment chacun le rapport d'un lecteur privé ; l'ancre
doit correspondre exactement au reçu courant. La relecture des autorités,
de la définition, des permissions et de l'annulation reste dans le runtime,
après cet audit et avant l'effet. Aucun droit ne vient du thread lecteur.
Le contrôle conserve l'échéance Prepared ou Release existante, sans la prolonger.

C19 — reprise d'une vue déjà publiée (révision79).
PrepareFenced avec vue exige WorkspaceViewV1 et StartRecoveryV1, sans accorder
la création d'une nouvelle vue. La capacité créateur reste distincte et fermée.
Le fence exact précède l'audit privé ; la chaîne de reçus remonte à la vue A,
avec arrêt source confirmé, vue non conservée/fermée et aucun fichier en cours.
Le contexte de contrôle possédé existant audite cette vue puis relit les droits,
la sélection et le placement. Même base, même vue, même source et même plafond ;
aucun export ou déplacement de HEAD. La liaison d'alias198 reste dans son reçu.
Un ancien pair sans la capacité lecteur est refusé avant l'envoi de la variante.

C19 — revue R75, placement et volume (révision84).
Le budget16Mio du WorkerStore couvre uniquement ses reçus et preuves privées.
Les données de la vue sont plafonnées par le manifeste et l'espace libre
observé sur leur volume, sans réservation contre les autres programmes.
Un seul export natif possédé par worker est lancé à la fois. Les préparations
suivantes gardent leur graine et leur délai, attendent ce créneau puis observent
l'espace après l'export précédent. Un marqueur d'export ou de fichier inconnu
bloque un nouvel export. Cette règle ne promet pas un quota système partagé
avec d'autres comptes ou programmes. Un ENOSPC conserve sa cause et les preuves.
L'adoption native recalcule exactement la politique de placement du helper,
y compris le toplevel Git déduit du manifeste ; un ancêtre situé dans Git refuse.
L'inspection automatique routée lit A sans effet ; la préparation différée
vérifie aussi A dans le helper avant ses premières écritures, pas par une
seconde sonde implicite dans begin. Les graines en attente ne gagnent aucun délai.

C19 — borne réelle des écrivains et preuve de budget (révision86).
La réservation d'une graine inconnue excède huit Mio ; deux graines inconnues
dépassent les seize Mio existants avant tout fork. L'hypothèse de 128 exports
natifs simultanés ne tient donc pas dans le contrat actuel. Le test le vérifie
sans augmenter ce budget. Une garde d'écrivain explicite couvre en plus les
exports et les matérialisations sur des vues déjà adoptées : un seul helper
d'écriture de données à la fois, les marqueurs inconnus gardant ce créneau.
Un fichier demandé pendant ce travail refuse avant sa réservation avec
operation_capacity ; la préparation native attend sous sa graine existante.
Les autres lectures et missions gardent leur propre état. Le volume peut
toujours être consommé par d'autres programmes : aucun quota système promis.

C19 — port de lecture d'un fichier omis, avant code public.
Réutiliser NativeDelegationRequest, le port MCP/CLI existant et les jetons
begin_view_file/poll_view_file du runtime. Ajouter une opération fermée de
lecture de fichier de commit, avec path relatif exact et task_id facultatif.
Sans task_id, la connexion enfant vivante désigne sa propre Task. Avec task_id,
le parent doit prouver qu'il en est le propriétaire par can_own_task ; aucun
identifiant de client, de worker, d'exécution, de preuve ou de chemin absolu
fourni par le lecteur ne donne autorité. Le maître dérive le slot d'exécution
actuel et son identité depuis ses magasins, sans nouvelle mission.
La Task doit être active, sa vue publiée, son instance et sa ronde courantes ;
révocation, annulation, pause ou clôture refusent avant effet. Refaire ces
contrôles après l'attente. child_requests=deny reste inchangé : cette lecture
d'état propre n'est pas une demande fournisseur à faire approuver par le parent.
Le worker exige la capacité WorkspaceViewV1, l'exécution exacte, un chemin du
manifeste dans la racine remappée autorisée et ses relectures de droits avant
fork puis avant mise à disposition. Aucun chemin ../, absolu, lien libre,
commit ou blob fourni par le lecteur. La vue garde sa base et son intégrité.
Le contexte différé possède le ViewFileRead et réutilise les bornes128/8,
l'époque et le canon exact ; une reconnexion oublie les corrélations anciennes.
Le retour positif contient le chemin de données de la vue et la base, jamais
les chemins de preuves privées, l'environnement ou un stderr. Il n'autorise
aucun écrasement, Git, exécution de fichier ou effet dans le dépôt source.
Le MCP classe ce port WritesOwnState, disponible en discovery, et exige une
capacité client négociée spécifique avant l'envoi à un ancien maître. La lecture
ne réactive aucun créateur. Un refus nomme cause, portée, prochaine action et
prérequis de reprise ; une issue inconnue conserve cette Task et sa vue.
La CLI et le MCP utilisent la même autorité et la même opération native.
Ce contrat décrit le raccord restant ; aucun port nouveau n'est attesté ici.

C19 — fin de lecture sans audit dans le runtime (révision95).
La récolte du helper durcit son arrêt avant de créer le lecteur d'intégrité.
Ce lecteur possède le manifeste, sans store ni droits d'exécution. La boucle
ne joint que les lecteurs terminés et publie ensuite le CAS de l'opération
exacte, en refusant toute conservation ou opération remplacée. Annulation ou
fin de tâche ne supprime pas le lecteur encore détenu ; aucun fichier n'est
rendu à une exécution périmée. Un arrêt déjà durci ne suit pas un PID recyclé.

### Gate du lecteur et installateur autonome — R110
Le lecteur comprend tous les reçus de vue avant que le créateur public ne soit ouvert. formats report annonce workspace.worktree.v1 et workspace.commit_view.v1 ; le contrôleur installé exige ces deux lecteurs à toute bascule ou tout retour, même si COMMIT_VIEW_LAUNCH_ENABLED reste false. La capacité de lecture de fichier est indépendante du créateur. L'installateur autonome réutilise ces ports fermés : rapport sous limite de fichier, taille8Kio, nonce neuf et SHA256 avant/après ; validation par le programme installé quand il existe. Cette gate précède les fournisseurs, profils, remplacement et services. Sans ancien lecteur capable du port, refuser et faire qualifier le bootstrap de lecteur par le responsable. Exception historique0.6.3 sans port : uniquement première installation, jamais par-dessus un contrôleur courant. Aucune liste de formats dans le shell, aucun inventaire de store supposé vide. La qualification réelle des images de retour reste requise avant activation ; ce contrat n'est pas une preuve de livraison.

### Reprise du constat de fichier arrêté
Le résultat de l'audit reste possédé jusqu'au CAS. Après redémarrage du worker, seuls les reçus de matérialisation déjà stopped et sans available autorisent la reprise du lecteur de données. Ce lecteur ne démarre ni processus de copie ni mission et ne change aucun droit. Toute opération d'arrêt inconnu est exclue. La garde de quarantaine du store demeure ; un échec de vraie écriture exige toujours le parcours de maintenance autorisé.
