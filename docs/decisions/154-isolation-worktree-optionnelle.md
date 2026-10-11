# ADR 154 — Worktree optionnel pour une délégation native

Date : 2026-10-12. Statut : Accepté pour conception et implémentation.
Validation de code et activation de la session 154 : en cours, aucune livraison déduite.

## Contexte

Un enfant natif partage le dossier de son parent. Plusieurs écritures peuvent
se mélanger. file_collision avertit après les faits. L’utilisateur veut pouvoir
choisir un dossier distinct et connaître la branche et les commits produits.
Bridget reste un moteur de communication et de délégation. Il ne devient pas
un environnement de développement.

Base : 89f7a4163c2fb27229587f50a46414295ba3674d, incluant la session 153 livrée.
La stratégie shared reste le défaut. La garde, les rondes et le brief 153
restent inchangés. L’héritage 149 est conservé sans gain de droits.

## Décisions D1 à D14

| Décision | Choix accepté |
|---|---|
| D1 | workspace facultatif sur Delegate ; aucun outil distinct. |
| D2 | Worktree sous cwd/.worktrees/task8 ; sous la racine parent. |
| D3 | Rejeu admis et autorisation du cwd canonique avant les lectures Git de la session 154 ; réservation sous verrou, création hors verrou, revalidation. |
| D4 | Posture development et paire d’héritage prouvée ; refus nommés avant effet. |
| D5 | Exécuteur Git unique ; formes/options en liste blanche, temps et sorties bornés. |
| D6 | Aucune suppression de branche ou de worktree par Bridget, même après échec. |
| D7 | Cleanup après tentative de rapport terminée et arrêt confirmé/fin de garde 153 ; kept ne suffit pas. |
| D8 | Faits Git structurés et suffixe écrit par Bridget ; imitation enfant neutralisée, budget exact et coupe UTF-8. |
| D9 | shared_writers informatif ; aucune nouvelle interdiction du partage. |
| D10 | Refuser filtres exécutables, inclusions, clones partiels et worktreeConfig. |
| D11 | .worktrees déjà ignoré ; plafond de 32 par union de chemins, aucune configuration écrite. |
| D12 | Rapports hors verrou ; CAS état/mission/ronde ; une tentative bornée, même en erreur Git. |
| D13 | Réservations dans Arc<Mutex> séparé ; aucune garde Drop reprenant le verrou global. |
| D14 | Git exécuté par le daemon pour parent autorisé ; modèle T3 durci, risque résiduel accepté. |

L’exécuteur couvre avant-vol, création, rapport et deux lectures existantes du
wrapper. Aucun nouvel outil MCP, dépendance Rust, table SQL ou migration. Le code
de droits reste inchangé. communication.rs et la résolution du projet commun
restent hors périmètre ; leur lecture Git historique non bornée n’est pas couverte
par l’exécuteur 154. La création par le wrapper est rejetée : elle serait
asynchrone, après admission, sans preuve suffisante des droits au bon dossier.

La whitelist valide les formes et options. log --output et config écriture sont
interdits. Le seul exécuteur pose les options de contexte. Sur Git 2.55, status
refuse --no-textconv et --no-ext-diff (RC129 constaté par le principal). Les
formes compatibles de log seules utilisent ces options. Hooks, fsmonitor,
configurations système/globale, programmes externes et réseau sont neutralisés
par les gardes communes d’environnement et de configuration prévues.

## Rapport, rondes et reprise

Avant publication, une sauvegarde conditionnée à waiting_for_children, mission
et ronde courantes range rapport, suffixe et result_available ensemble. Une
lecture Git qui échoue rend des faits inconnus et « rapport indisponible ».
L’échec de lecture ne crée pas une attente infinie. Pour failed/cancelled,
rapporter chaque membre en worktree de l’arbre après arrêt confirmé.
Sauvegarder d’abord le rapport. Poser ensuite cleanup_done dans une seconde
sauvegarde, seulement quand le rapport est prêt.

Le marqueur privé Task.workspace_report_mission (default/skipNone) identifie
la mission déjà rapportée. Il couvre publication, erreur de livraison, cleanup
et reprise. Preuve : native_delegation.rs:2193-2228 peut passer un résultat
à failed ; le CAS de succès seul ne suffit pas. begin_round vide marqueur et
faits. Aucun champ public de ronde dans WorkspaceRecord. kept conserve le
dossier et permet de continuer selon les droits hérités. Les historiques
publiés restent immuables.

L’ancre Git vient du registre du parent hors dossier enfant, avec parcours borné et
correspondance toplevel/path enregistrés. Ne pas lire enfant.git. Ancre absente
→ rapport indisponible. Aucun champ d’ancre supplémentaire. Cette conception
reste à vérifier par les tests prévus. H12 est tranché : WorkspaceWork unique, table des créations et ensemble des rapports (task/mission/round)
inflight ; les ticks concurrents sautent le même rapport. Garde RAII retire seulement
la réservation de table séparée, aucun thread/moteur en arrière-plan ni quota inventé.
Validation native à venir.

## Droits hérités

Le refus global des enfants Claude proposé en contre-revue est rejeté. La
preuve 149 exige déjà full+never pour Codex vers Claude en développement
(native_permissions.rs:218-225). Claude avec règles vers Codex reste fermé
(:253-268). Le schéma Codex est fermé (transport/native_permissions.rs:55-65).
Les réglages Claude ignorés du parent ne sont pas des droits Codex perdus. Les
règles de dossier pour Claude vers Claude restent applicables. Aucun accès Git
commun ajouté à un enfant Codex workspaceWrite pour lui permettre de commiter.

## Risque accepté

Un parent confiné qui peut modifier la configuration du dépôt entre le contrôle
et l’exécution de Git peut faire exécuter du code avec les droits du daemon.
Ce risque est accepté explicitement, au même niveau que T3, dans le cadre de
menace de l’ADR 011. Les durcissements réduisent la surface sans la fermer.

Le cadre local de l’ADR 011 constate qu’un agent de la machine peut exécuter
commandes et écrire fichiers. Référence :
/Users/moi/Nextcloud/10.Scripts/64.bridget/docs/decisions/011-approbation-exposee-puis-second-facteur.md
L’hypothèse est un daemon et un dépôt sous le même compte Unix. La course sur
un lien .worktrees est recontrôlée par canonicalisation après création ; son
résiduel est accepté dans ce cadre.

## Comparaison T3 et alternatives rejetées

Comparaison déjà recherchée dans T3 : GitVcsDriverCore.ts:3418-3419 et :3789
exécutent worktree add avec les droits du serveur ; GitVcsDriver.ts:430 décrit
le retrait. Cette ADR rapporte cette recherche, sans prétendre avoir retesté T3.
Bridget ajoute gardes Git, liste blanche, bornes et aucune suppression.

Git confiné : variantes macOS/Linux, profil Git plus large que workspaceWrite et
maintenance supplémentaires. Isolation coopérative par fichier .git : isolation
non imposée, enfants confinés exclus et faits non fiables. Réserver aux parents
non confinés : fonction réduite, contrairement au choix utilisateur. git2 :
dépendance lourde. existing_worktree : cwd existant couvre déjà le cas.
Écrire info/exclude : surface de liens et modification de configuration non requise.
start_from_origin : accès réseau hors périmètre.

Nettoyage automatique rejeté : worktree emboîté ignoré et effaçable, fichiers
non suivis masqués par configuration, tête détachée non protégée par branche,
branche orpheline après retour arrière. L’humain nettoie après examen.

## Conséquences

Positives : dossiers distincts, parent inchangé, rapports corrélés à chaque
ronde, daemon sans verrou pendant Git, aucune décision de suppression.
Négatives : worktrees conservés s’accumulent ; plafond de 32 et recette humaine.
Un enfant workspaceWrite peut rendre modifications sans commit. Git LFS et
certaines configurations sont refusés. Les changements non commités du parent,
fichiers ignorés et sous-modules ne sont pas copiés.

## État de validation

Baseline canonique : 193 PASS/0 FAIL/1 ignoré. Mesure : 5 créations, RC0, médiane 0,588788 s,
maximum 0,662820 s ; délai de 20 s conservé. RED T003/T022 :1 PASS/6 FAIL, 23,45 s, RC101.
Aucun GREEN 154 ni activation proclamé. Les reçus et la revue des 15 hypothèses sont
sous la spec 154. Les tâches restent ouvertes jusqu’aux preuves du principal.

## Limite d’arrêt Git acceptée

En délai dépassé, SIGTERM vise le seul PID obtenu de child.id, puis attente de
récolte bornée avec try_wait. SIGKILL est interdit. Git normal et la fixture
fakeexecsleep sont couverts par le test prévu ; un programme hostile ignorant
SIGTERM peut persister. La garantie porte sur le délai de retour, pas sur un
arrêt inconditionnel. Risque résiduel explicite dans le modèle ADR 011 durci accepté.
L’ancre ignore entièrement enfant.git ; registre common recherché à partir du parent
ou de l’ancêtre principal, qui reste source de confiance de ce modèle. Aucun outil
ou mécanisme d’arrêt supplémentaire.

Fondations R1 : 15 PASS/7 FAIL, 49,10 s, RC101 ; 203 fichiers sources stables. T004/T021/T022
acquises à ce passage historique. La fermeture du protocole est acquise au transport R2 ; T005 est acquise après R4.

Point du 2026-10-12 après R4 : 19/28 tâches acquises, 9 ouvertes. Les passages
restent distincts : R3 finale 35 PASS/9 FAIL ; R4 finale 4 PASS/6 FAIL, avec six
fixtures de socket à corriger. Aucun GREEN global ni activation déclarés.

Point après R5 : 6 PASS/0 FAIL ciblés, RC0. Les preuves distinctes permettent
26/28 tâches acquises. T025/T026 attendent la suite finale et la revue du diff.
Aucun GREEN global ni activation déclarés.

## Contre-revue du code R1 — décisions et limites

La contre-revue conclut APPROVE_WITH_CHANGES. Le principal retient F1 et F2.
Leurs correctifs ont une preuve ciblée : RED 0 PASS/2 FAIL puis GREEN 2 PASS/0 FAIL.
Les reçus et empreintes sont dans validation/tests-154.md. Aucun GREEN global.

F1 : ne plus compter un chemin de tâche retiré par l’humain, uniquement si
symlink_metadata renvoie NotFound. Toute autre erreur reste comptée. Les
réservations restent comptées. Le test union32 existant est étendu : retrait
de seed et last, trente fixtures Git conservées, nouveau chemin possible.
Le RED avant correctif et le GREEN ciblé sont acquis sur les reçus finaux du principal.

F2 : une entrée voisine incomplète ou illisible du registre ne doit pas rendre
la cible entière indisponible. Passer à l’entrée suivante. Si l’ancre de la
cible demeure illisible ou introuvable, renvoyer une erreur ; ne pas inventer
une ancre. Conserver les bornes de 4096 entrées et 8192 octets et ne pas lire le
fichier .git de l’enfant. Le test d’ancre existant est étendu. Preuve ciblée acquise.

F3 : aucun lookup Git supplémentaire. worktree add -b garantit l’exclusivité
sans écrasement. Branche et base restent immuables dans le plan. Le message
chemin/branche décrit une tentative. Il ne prouve ni présence de restes ni
propriété de la branche en cas de création concurrente.

F4 : après création, un refus tardif donne workspace_create_failed, sauf le
cas path unsafe. Le motif interne reste disponible dans le diagnostic. Un
rejeu peut revalider puis rendre le refus d’origine. Limite de diagnostic
acceptée ; aucun retrait de fichiers ni élargissement de droits.

F5 : le décodage reste strict en UTF-8. Un sujet non UTF-8 peut rendre le
rapport indisponible avec des faits inconnus. Aucun transcodage ajouté.
F6 : la version Git est contrôlée avant chaque commande, car PATH peut changer.
Ce coût hors verrou est accepté. Aucun cache ou état global nouveau.

Pour failed/cancelled, rapport puis cleanup_done restent deux sauvegardes
ordonnées après arrêt confirmé. La publication d’un résultat, elle, range
rapport, suffixe et result_available dans sa sauvegarde conditionnée.
