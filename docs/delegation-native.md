# Déléguer avec Bridget

Bridget possède son moteur de délégation. T3 peut être absent. Le connecteur T3
atteste seulement la session qui appelle les outils. Il ne crée pas les tâches.

Ce moteur n'est pas l'outil `delegate_task` de T3. L'exécution ne dépend pas de T3.
T3 sert seulement à attester la session MCP.

Le parent consulte `bridget_capabilities`. Il choisit une entrée réellement
disponible. Le catalogue provient du registre natif Bridget. Les noms de modèle
et d'effort doivent correspondre exactement aux valeurs déclarées.

Un seul appel `bridget_delegate` crée l'enfant et remet sa mission. Exemple
de forme d'appel, à adapter au catalogue de la session :

```json
{
  "request_id": "revue-contrat-round1",
  "agent_type": "glm",
  "model": "glm-5.3",
  "task": "Lis le contrat indiqué et rends les défauts avec leur preuve.",
  "cwd": "/Users/moi/Nextcloud/10.Scripts/64.bridget"
}
```

`posture` est facultatif. Son absence demande l'héritage de la politique prouvée
du parent (voir plus bas).

`glm` est le nom du profil Bridget observé dans le registre de cette session.
Le nom d'instance T3 `claude_glm` n'est pas un type du registre Bridget.
Cet exemple ne prouve pas que ce profil existe dans tous les registres.
Un profil absent, un modèle différent ou un effort non déclaré produit un refus.
Sans fallback explicite, Bridget ne remplace pas la cible demandée.

### Catalogue, quotas et remplacement explicite (157)

Vérifier le catalogue effectivement chargé. Cette documentation ne prouve pas
l’activation du binaire ni une disponibilité d’authentification chez le fournisseur.

| Champ | Lecture |
|---|---|
| `can_run` | Au moins une posture passe le verdict pour ce parent. |
| `constraints` | Codes et messages français ; vide si can_run, sauf l’information non bloquante provider_limit_reached_window_unknown. |
| `quota` | Fenêtres observées, ou null si rien n’est connu. |
| `checks` | binary:true, quota:"observed", auth:false. |
| `verdict_context` | cwd égal à la racine attestée, ou null, et fact_scope:"session". |
| `features` | fallback:true et fallback_max:3. |

discovery/inherit/development proviennent du même verdict que l’admission.
La garantie porte sur la famille fermée de refus du contrat, avec le même parent,
la même posture, un fait de session non limité à une demande, le même dossier
et un état inchangé. Un autre cwd, un fait scoped à request_id, un binaire
retiré ou un quota changé sortent de cette garantie. L’admission reste l’autorité.
Pour un parent SameProject sans racine, cwd:null et les booléens du catalogue
restent faux ; cela ne retire pas une admission que ses gardes autorisent dans
un projet concret. Aucune racine ni aucun droit n’est inventé. Le catalogue
reste une lecture disponible pendant la pause ; il n’appelle aucun fournisseur.
Au bord exact d'une échéance de quota, le verdict et sa projection peuvent
lire deux instants distincts. L'admission recontrôle le quota.

Quota rend windows[{window,used_percent,resets_at,exhausted}], exhausted,
limit_reached, resets_at et observed_at. Les pourcentages/heures inconnus sont
null. Une fenêtre n’est épuisée qu’avec retour futur et preuve propre : rejected
Claude stream-json, ou au moins 100 % utilisé, toute source. Un statut Codex
global atteint ne désigne aucune fenêtre : limit_reached peut être vrai sans
exhausted. Le retour global est le plus tardif des fenêtres réellement épuisées.
Les faits passés sont ignorés ; la mémoire garde huit fenêtres par agent_type,
survit à la disparition de sa présence, mais pas au redémarrage du daemon.
Des fournisseurs du même compte n’ont pas de quota partagé dans cette projection.

Pour Codex, `exhausted` décrit le quota inclus ; `launch_blocked` décrit le blocage
réel. `credits` expose les indicateurs attestés, sans interpréter un solde.
À 100 %, des crédits disponibles ou illimités permettent de poursuivre si le
contrôle de dépenses est explicitement ouvert. Une donnée absente ou invalide
n'autorise aucun crédit. Un contrôle de dépenses atteint bloque même sans fenêtre.
Les réponses initiales et `account/rateLimits/updated` conservent l'ordre source ;
une mise à jour partielle peut retirer immédiatement une ancienne autorisation.

Ces faits de compte sont liés au profil et à l'instance du wrapper. Un autre
compte ou une autre machine ne transmet aucun droit de consommer ses crédits.
Avant création distante, ce compte reste inconnu : le wrapper peut être admis,
puis la remise de mission vérifie son propre budget. Les refus d'authentification
et les autres gardes restent applicables. Le catalogue local utilise le fait
scoped du même profil/instance ; sans ce fait, le contrôle historique du profil
reste conservateur. Cette admission locale suppose le même environnement de
lancement déclaré dans le registre ; elle ne constitue pas une attestation d'un
nouveau compte. La mémoire scoped conserve au plus 4096 instances et huit fenêtres
par instance ; elle disparaît au redémarrage. Le hold160 utilise le compte enfant,
et ne qualifie pas une fenêtre épuisée lorsque ses crédits restent utilisables.

Fallback est facultatif : 1 à 3 objets fermés {agent_type,model,effort?}, ordonnés,
sans entrée vide, doublon ou cible identique à la demande. Le parent choisit
les alternatives ; Bridget ne crée ni droit ni accord humain supplémentaire.
Pour chaque cible : type/modèle/effort, préflight, politique héritée, gardes
workspace154/child_requests155, puis quota. Modèle/effort invalides ne sont jamais
masqués par un motif remplaçable de la cible demandée. Les droits du parent,
reviewer155, pause, dossier et rétention gardent leurs règles existantes.

Le remplacement commence seulement sur les neuf motifs FR013 : command_missing,
billing_guard, env_unfit, unsupported_capability, negotiation_failed,
permission_source_unavailable, permission_mapping_unavailable,
provider_confinement_unavailable, provider_quota_exhausted. settings_revision_changed,
refus de droits, workspace incompatible et child_requests incompatible de la
cible demandée ne déclenchent pas de remplacement. Pour un candidat listé,
un refus est conservé dans tried et l’admission passe au suivant.

La réponse et bridget_task_status rendent toujours la clé substitution :
{requested,selected,reason,tried}, ou null sans remplacement. Cet ajout concerne
la réponse ; les octets canoniques d'une demande sans fallback restent inchangés.
La tâche porte la cible effective ; son canonical garde
la demande originale et sa liste. Rejouer garde exactement cette substitution,
sans recalculer un quota ni sélectionner à nouveau. En worktree, Probe choisit
une seule cible/définition épinglée ; Reserve et Commit la consomment. Commit
revalide cette cible dans le dossier enfant réel et refuse sans resélection.
Les diagnostics154 et options immuables restent conservés.

Préflight refuse plus tôt command_missing/billing_guard/env_unfit/
unsupported_capability, avant création de tâche. Provider_quota_exhausted rend
detail.resets_at ; fallback_exhausted rend detail.tried si toutes les cibles
échouent. Aucune tâche ni consommation de clé pour ces deux refus. Le MCP rend
code suivi du JSON de detail. Sans fallback, les codes restent ceux du contrat,
avec les refus précoces annoncés. Aucun remplacement après admission ni reprise
sur quota dans le seul contrat157. L’opt-in160 est décrit ci-dessous.
checks.auth:false reste explicite.
Le préflight partagé passe avant le contrôle du cwd des lancements existants.
Si le cwd et l'environnement sont invalides ensemble, env_unfit peut donc
précéder cwd_gone.



Le reçu conserve `task_id`, les références de l'enfant et de la mission.
Si la réponse se perd, reprendre les mêmes arguments avec le même `request_id`.
Le rejeu retrouve la tâche existante. Modifier son enveloppe produit
`envelope_mismatch`, sans nouvelle mission. Une clé neuve crée une nouvelle mission.
Ne pas changer de clé après une issue inconnue.

Le parent reçoit automatiquement un résultat corrélé à la mission.
`bridget_task_status` permet aussi de lire l'état durable avec `task_id`.
Un enfant actif retarde la publication du résultat de son parent.
Le résultat publié d'une ronde ne change plus. Une fin de tour ne suffit pas à le publier.
`bridget_task_cancel` annule sa tâche et ses descendants actifs.
Le catalogue annonce `mission_reply_limit_secs` (3600 secondes, le défaut),
`mission_reply_min_secs` (60) et `mission_reply_max_secs` (86 400).
`reply_timeout` est facultatif : il fixe le délai de réponse de cette mission,
en secondes entières. Sans lui, le délai reste 3600 secondes.
Le reçu donne `mission_deadline_at` après admission de la remise. Cette date brute
reste identique après rejeu ou reprise. Le temps de pause est exclu du délai métier.
L'expiration de ce délai produit `mission_reply_timeout`, puis le nettoyage.
Un délai de transport ou une attente du parent n'annule rien.

Après une panne du daemon, Bridget peut reprendre une mission `starting`,
`mission_pending` ou `working`. Il vérifie l'identité de flotte, les droits encore
valables et le temps restant. La tâche et la mission gardent leurs identifiants.
Deux reprises après redémarrage sont possibles ; la troisième échoue avec
`native_resume_exhausted`. Un résultat déjà capturé reste durable et attend la fin
des descendants avant sa remise. Rejouer la demande retrouve la même tâche.

La vue courante donne `resume_attempts`, le nombre de décisions de reprise après panne.
Ce compteur est sauvegardé avant la reprise. Un crash peut donc consommer une
tentative avant son effet ; il ne permet jamais de dépasser la borne.

## Reprise sur limite d’usage (160)

Vérifier les capacités du binaire chargé avant d’utiliser ce contrat.
Cette documentation ne prouve pas l’installation ni l’activation de 160.
La lecture T002 n’a observé aucun quota réel. Elle ne prouve ni la raison du
refus fournisseur ni l’ordre réel entre son fait de quota et son échec.

`bridget_delegate` accepte le booléen facultatif `resume_on_limit`. Au MCP,
`null`, chaîne, nombre et objet sont refusés en `InvalidParams`.
Seul `true` active la reprise. Absent et `false` la désactivent, mais forment
deux enveloppes différentes : les intervertir avec la même clé produit
`envelope_mismatch`. Sans ce champ, le contrat conserve les octets et le
comportement précédents. Le catalogue annonce `features.resume_on_limit:true`,
`resume_max_wait_secs:21600` et `resume_max_rounds:2` ; ce sont des capacités,
pas une activation automatique pour chaque mission.

Une tâche opt-in `working` peut attendre une fenêtre épuisée attestée par 157.
Le fait doit être récent pour la mission ou sa dernière attente. Le retour doit
être futur et au plus à 6 heures ; deux rondes au maximum sont conservées
même après leur solde ou un redémarrage. Le statut Codex global seul ne suffit
pas. Après un échec sans fait connu, seules les observations faites au plus
5 secondes après le début du hold sont admissibles. La résolution au tick
peut arriver plus tard ; elle utilise l’heure réelle de décision. Un fait
observé après cette borne ne qualifie pas l’échec et ne rouvre pas une tâche
échouée. Cette règle ne garantit pas l’arrivée d’un fait fournisseur à temps.
La pause 152 garde sa priorité.

Le statut ajoute `quota_wait:null` ou `{round,phase,resumes_at}`.
La tâche reste `working` ; `waiting` attend le retour plus 30 secondes et
`relaunching` attend le solde de la remise. Aucun avis d’échec pendant la
décision et l’attente ; la demande suivie reste conservée. Les rappels sont
suspendus pendant le hold et la part gelée de l’attente, jusqu’au retour + 30 s.
Un échec final opt-in utilise le seul avis durable existant. Il n’envoie pas
en plus le message direct d’échec de livraison.

La relance garde tâche, mission, fournisseur, modèle, effort, permissions
héritées et définition épinglée. Elle ne crée aucun droit supplémentaire. Elle ne réévalue pas le fallback 157. Son identité est
`limit-resume:<task_id>:<ronde>:<retour>`. La pause diffère le départ ; l’enfant
doit être connecté et libre. Seul l’ACK solde l’attente : réserver, graver ou
écrire la remise ne suffit pas. Une remise en vol reste `relaunching`.
L’ACK ne prouve pas le résultat métier ni la mémoire d’un modèle réel.

Le gel ne concerne que le délai de l’enfant. Il utilise l’union des intervalles
de pause et de quota, sans compter deux fois leur chevauchement. Le gel quota
s’arrête au retour plus 30 secondes, même si l’enfant reste absent ; son délai
peut ensuite expirer avec `mission_reply_timeout`. Le délai du parent n’est
pas prolongé. Les faits de quota 157 restent volatils au redémarrage ; les
attentes qualifiées et leurs rondes sont durables. Un hold encore ouvert
au redémarrage échoue avec son erreur d’origine et l’avis durable.

| Erreur finale | Lecture |
|---|---|
| `usage_limit` | Retour passé, à plus de 6 heures, ou deux rondes faites ; `detail.limit_reset_at` donne le retour dans la vue de statut détaillée. |
| `native_relaunch_failed` | Remise incertaine, orpheline ou non reprenable ; aucune réussite supposée. |
| `native_refusal_uncorrelated` | Après une relance, le refus ne peut pas être rattaché à l’exécution courante. |

Ces rondes quota ne consomment pas les deux reprises après panne de 152.
La reconstruction quota qualifiée reste distincte de sa reprise ordinaire.
L’annulation interdit toute relance ; une réconciliation non vérifiable garde
`cancelling`, avec `native_cancel_unverifiable:quota_relaunch`.
La corrélation temporelle ne prouve pas que le quota a causé cet échec.
Un fait admissible antérieur peut qualifier une panne sans rapport. Reprendre
la mission peut donc répéter ses effets externes. L’identité de relance, ses
bornes et l’ACK ne garantissent pas l’unicité des effets métier.
Décision et ordre durable :
/Users/moi/Nextcloud/10.Scripts/64.bridget/docs/decisions/160-reprise-sur-limite-d-usage.md.

## Contrôle humain global et plafond MCP — 158A

Ce contrat158A s’applique au binaire et au catalogue effectivement chargés.
Ce texte ne prouve pas leur activation. 158B reste spécifiée seulement :
réveils, déclencheur local, faits manqués et secrets ne sont pas livrés ici.

### Voir et arrêter sans fil T3

```sh
bridget control delegations
bridget control delegations --state all --limit 100 --json
bridget control delegations cancel <task_id>
bridget control halt-delegations --reason "Revue humaine"
bridget control halt-delegations --reason "Arrêt explicite" --cancel-active
bridget control resume-delegations
bridget control status
```

La liste est globale, réservée à l’humain attesté, sans --t3-thread ni
--project-root. Défaut : active, 50 lignes ;maximum 100. Les tâches récentes
viennent d’abord. Une ligne porte task_id, propriétaire, enfant, état, âge,
échéance restante et parent_task_id ; aucun corps de mission ou résultat.
Actif suit l’occupation d’un slot (holds_slot) : tâche non terminale, ou tâche
gardée tant que cleanup_done est faux. Un résultat rendu peut donc rester actif.
Le suivi Lineage ciblé dans T3 garde sa voie existante.

ControlStateV1 et un acteur humain attesté sont requis. La capacité seule
n’accorde pas cette autorité ; un client agent négocié reste refusé. À requête
correctement formée, l’ordre est negotiation_required, capability_not_negotiated,
puis human_actor_required. Un alias from=humain ne suffit pas. Les mutations
halt, resume et cancel exigent un terminal interactif. Codes CLI : 0 succès,
1 refus/erreur daemon, 2 usage. Cette garde conserve la borne ADR011 : elle contrôle
les outils déclarés, pas un processus hostile du même compte.

L’arrêt est persistant et indépendant de la pause. Il refuse une nouvelle clé
bridget_delegate par delegations_halted, sans tâche ni clé consommée ; un rejeu
admis rend sa tâche existante. Le catalogue indique delegations_halted.
Status, catalogue et annulations restent disponibles. Arrêt et pause se
composent : l’arrêt est prioritaire si les deux sont posés. Resume-delegations
ne lève pas la pause ;lever la pause ne lève pas l’arrêt. Sans --cancel-active,
les tâches existantes restent intactes.

Avec --cancel-active, la première application vise les arbres actifs retenus
à cet instant. Le champ cancelled compte ces arbres ciblés AVANT le commit
du reçu ; il ne confirme pas l’arrêt de leurs processus. État et réponse sont
committés ensemble, puis l’annulation utilise human_cancel au plus une fois.
Un command_id connu rend sa réponse historique, replayed:true, sans réécrire
l’état courant ni annuler de nouveau. A arrêt/annulation, B reprise, C nouvelle
tâche, rejeu A : C reste intacte. Un crash après le commit peut laisser une
annulation inachevée ; l’humain utilise delegations cancel, sans promettre une
reprise automatique. Les 256 reçus les plus récents survivent au redémarrage.
Un identifiant plus ancien, purgé, peut être appliqué de nouveau ; la CLI crée
un identifiant neuf par geste. Ces reçus ne sont pas un historique d’audit.

### Portées des outils et enfant discovery

La table classe chaque outil par base et, au besoin, par action ou cible :
reads, writes_own_state, sends, writes_threads, delegates, publishes. Le test
exhaustif de CI vérifie outils→table, table→outils et readOnlyHint ; ce n’est
pas une obligation imposée par le compilateur. readOnlyHint est vrai seulement
pour une BASE reads. Journal a pour base sends, indice faux, même si un appel
sans to est abaissé à reads. Aucun outil MCP ne porte un contrôle humain.

Un enfant discovery peut appeler seulement reads et writes_own_state. Sa posture
effective prime sur la posture omise. Le plafond dure jusqu’à cleanup_done,
même si la tâche est terminale ou si l’enfant attend entre deux rondes. Un
appelant sans cette tâche discovery conserve sa surface d’outils existante ;
les autres droits de projet, d’identité et de portée continuent de s’appliquer.

Permis : lecture d’artefact ; thread list/show/history et read puis ack avant
la page suivante ; events types/list/sub/unsub ; handoff preview ; journal sans
to. La réponse finale automatique au parent reste permise par le wrapper.
Ruptures voulues :thread post/create/add_members/close, journal avec to,
publish_artifact, send, cancel, handoff en envoi et toute délégation sont refusés
par capability_denied, sans effet. Ne pas contourner ce plafond par la CLI ou
une autre identité. Un besoin de publication depuis discovery demande une
nouvelle spec ; le plafond n’est pas élargi pendant cette implémentation.

158A ne borne ni tokens ni tours. Reply_timeout reste la borne de temps150.
Les conditions de reprise d’un budget de tours sont dans l’ADR158 ; la mention
« budgets de tokens ou de coût (session158) » de150 est levée par cette décision.

## Réveils et faits manqués — contrat 158B prévu

Cette section décrit US5–US7. Elle ne prouve pas une livraison 158B, un test vert
ou une fonction chargée. Le reçu 158A reste distinct. Vérifier la version du
binaire et le catalogue MCP chargés avant d’utiliser ces nouvelles surfaces.
US8, secrets et HTTP restent hors implémentation.

### Programmer ou déclencher

L’outil `bridget_wake` a trois actions : `schedule`, `list`, `cancel`.
Une programmation prend `target`, `cadence` et un `label` facultatif de 2048 octets
UTF-8 au plus, sans troncature. Cible fermée : `{"kind":"self"}`,
`{"kind":"agent","id":"UUID"}` ou `{"kind":"thread","id":"UUID"}`.
Cadence : `{"kind":"interval","seconds":60}` au minimum, ou
`{"kind":"at","time":"09:00","weekdays":[1,2,3,4,5]}`. Les jours vont de 1 à 7,
sans doublon ; l’heure suit le fuseau du daemon. `list` et `cancel` portent
seulement sur les programmations de l’appelant ; cancel prend `schedule_id`.
Aucun owner, from, scope ou horodatage libre. Champs inconnus et null explicite
à la place d’une valeur facultative sont refusés.

Base de l’outil : `sends`, `readOnlyHint:false`. `schedule self` et la gestion
propre sont `writes_own_state`. Programmer vers agent ou fil reste `sends`,
même si son UUID est celui de l’appelant. Un enfant discovery peut utiliser
self et sa gestion propre, avec son origine native gelée à l’admission.
La tâche, l’instance, les ancêtres, la mission et l’exécution sont revérifiés
au gel et au départ. Sa fin de vie connue annule les futures occurrences et
solde les préparées non parties `owner_unavailable`. Une preuve disparue ou
une erreur de magasin ferme le départ ; aucun repli ordinaire, processus ou
nouvelle ronde native n’est créé.

Une création schedule n’est pas idempotente : chaque admission crée un UUID
neuf. Si la réponse est perdue, relire list avant de recommencer. Les bornes
physiques sont 16 programmations par propriétaire et 128 au total, annulées
comprises. Seules les anciennes annulées sans occurrence en attente/en vol
peuvent être retirées pour admettre une nouvelle ligne. Sinon schedule_limit.
Cancel d’une ligne connue déjà annulée rend already_cancelled. Après retrait,
son UUID refuse target_unknown ; cela ne prouve pas l’échec de son annulation.

```sh
bridget wake AGENT_CONNU --label 'Point de reprise' --id rappel-1
bridget wake UUID_DU_FIL --label 'Point de reprise' --id rappel-fil-1
bridget wake list --all
bridget wake cancel UUID_PROGRAMMATION
```

Le déclencheur CLI est unitaire, utilisable depuis un script sans terminal
interactif. La cible doit être connue ; une ambiguïté refuse. Réutiliser le
même `--id` et le même contenu au retry. ID omis : une clé neuve. Le reçu099
conserve le canon et issued_at initial pendant 7 jours, sans renouvellement.
Rejeu/collision passent avant le quota global de 60 nouveaux IDs CLI par minute
glissante ; COUNT et réservation sont transactionnels. Programmations et
appels MCP n’utilisent pas ce quota. Un reçu expiré refuse le rejeu.

Vers un fil, le script hérite le contexte réel `BRIDGET_AGENT_ID_FILE` et
`BRIDGET_AGENT_INSTANCE_ID`, avec credential, membre et projet attestés.
Sans ce contexte, refus avant réservation ; jamais de `--from` ou faux auteur.
La vue CLI globale `list --all` et l’annulation administrative utilisent
ControlStateV1 et l’acteur humain ; la mutation exige un terminal interactif.
Aucun flag administratif n’est exposé par MCP. `list` sans `--all` utilise
l’identité authentifiée et reste limité aux programmations propres.

### Temps, remise et incertitude

Pendant la pause, aucune nouvelle occurrence programmée n’est gelée. À sa
levée, un intervalle rattrape seulement sa dernière échéance due ; la prochaine
est strictement future. Une heure fixe retardée de plus de 600 s est sautée.
Une occurrence déjà préparée conserve sa grâce initiale 600 s, sans prolongation.
Une CLI unitaire expirée ne crée aucun rattrapage. Halt-delegations, même avec
cancel-active, vise les délégations et n’annule pas les programmations.

Le fuseau du daemon est annoncé par list. Heure inexistante : sautée ; heure
double, DST ou autre changement d’offset : premier instant UTC correspondant,
une seule occurrence par date civile. Domaine supporté : offsets en minutes
entières entre −24h et +24h ; sinon calendar_unavailable, aucune approximation.
Un changement de fuseau observé recalcule les échéances futures non gelées,
sans modifier les occurrences/canons préparés ou effacer leur date précédente.

Agent : aucun modèle hors ligne n’est lancé. Route, DND, capacité, pause et
circuit-breaker existant sont revérifiés au départ ; le propriétaire est son
expéditeur logique (défaut 8 échanges/180s par couple, sans quota B supplémentaire).
La notificationSystem attribue « Réveil programmé par <propriétaire attesté> »
puis le label entier. Le label est une donnée d’agent, jamais une autorité.
Prepared/deferred ne prouve pas une réception. Après départ099, ACK corrélé ou
outcome_unknown ; unknown est terminal pour cette occurrence, sans renvoi.

Fil : contribution attribuée au membre propriétaire, avec label entier et
préfixe court. Son UUID102 est gelé avant réservation099 éventuelle. Le commit
de la contribution et des intentions102 prouve la remise au fil, pas sa lecture.
Ses alertes partagées conservent les règles102 ; elles n’expirent pas à 600s.
Le reçu099 de commande CLI-fil n’est pas une remise socket vers un agent.

### Relire les faits manqués

`bridget_events` avec `action:"missed"` est Reads. Paramètres facultatifs :
`limit` et `after_seq` (séquence de fait, pas UUID ou horodatage). Owner implicite
issu de l’identité ; aucun owner libre. Pagination bornée, null explicite
refusé. Lire n’efface aucun fait et n’avance pas le seuil d’accusé de réception.

Conservation :8192 faits globaux, pending compris,64 par abonnement,7 jours.
Chaque fait structuré et chaque résumé font 16384 octets au plus. Un fait trop
gros devient une lacune datée et un compteur, sans corps ni troncature. La
provenance survit à once ; sa consommation et la sauvegarde du fait/gap sont
transactionnelles. Le worker existant garde son budget 1s et une deadline
durable observed_at+5s non renouvelée. Écriture incertaine reste write_unknown,
jamais une preuve que rien n’a été écrit. Journal_only et throttle volontaires
ne sont pas des pertes.

Les métadonnées sont bornées à 8192 propriétaires et une ligne globale fixe.
Les propriétaires avec faits ou résumé non terminal restent protégés. Si tous
sont protégés, une nouvelle attribution crée une lacune globale datée, aucune
8193e ligne. La lacune globale ne contient aucun UUID ni corps d’un autre
propriétaire. TTL/évictions ne reconstituent pas une histoire perdue.

Register confirmé et DNDlevé du même retour font un épisode, pas deux résumés.
Le gel attend tous les pending_write du propriétaire soldés à leur deadline 5s.
Puis il fige génération, canon, UUID, issued_at et cutoff. Le résumé est un
agrégat de compteurs/types/périodes/lacunes, jamais la concaténation8192 faits.
Seul ACK099 validé et corrélé avance les seuils gelés ; ni missed en lecture,
ni unknown. Un résumé préparé expire après 600s depuis son gel. Unknown terminal
n’est pas renvoyé ; une nouvelle génération attend un nouveau retour réel,
avec l’incertitude précédente annoncée. Un ACK ancien ne touche pas une
identité recréée. Un fait que le daemon n’a jamais observé n’est pas inventé.

## Espace de travail — contrat 154

Ce contrat exige un catalogue et des processus chargés qui annoncent
`workspace_strategies`. Les sources et cette documentation ne prouvent pas
l’activation de la session 154. Consulter `bridget_capabilities` avant de choisir.

Sans `workspace`, l’enfant garde le dossier partagé. Le comportement de la session 153 reste
le défaut. `{"strategy":"shared"}` demande ce partage explicitement. Ajouter
`{"strategy":"worktree","base_ref":"main"}` à `bridget_delegate` pour demander
un dossier Git distinct, si la paire de fournisseurs le permet. La posture
effective doit être `development`. Les droits hérités ne sont pas élargis.

`base_ref` est facultatif : son défaut est HEAD. Bridget résout la base en un
commit et la fige. `branch` est facultatif : son défaut est
`bridget/<8 premiers caractères du task_id>`. Une branche demandée doit être
nouvelle. Le reçu conserve `task_id`, `cwd` et `workspace` avec état, chemin,
branche et base. Garder les mêmes arguments et la même request_id sur rejeu.
Absence de workspace et shared explicite sont des enveloppes différentes.

Le dossier créé se trouve sous le dossier du parent dans `.worktrees/<task8>`.
Le dossier enfant est ce worktree, avec le décalage conservé si le parent
travaille dans un sous-dossier du dépôt. Le HEAD, l’index et les fichiers du
parent restent dans son checkout. Les modifications non commitées du parent,
les fichiers ignorés et les sous-modules non initialisés ne sont pas copiés.

Git 2.44 ou plus est requis. `.worktrees/` doit déjà être ignoré par le dépôt
(.gitignore ou info/exclude). Une règle globale ne suffit pas. Bridget ne
modifie aucun fichier de configuration. Git LFS, filtres exécutables, inclusions
de configuration, clones partiels et worktreeConfig sont refusés. Un lien
symbolique à la place de `.worktrees` est refusé. Le plafond est de 32 chemins
worktree distincts, réservations en cours comprises.

| Refus | Suite utile |
|---|---|
| workspace_requires_development | Choisir une posture de développement déjà autorisée. |
| workspace_inheritance_unavailable | Cette paire ne prouve pas les droits dans un dossier différent ; garder shared. |
| workspace_git_unsupported | Installer une version Git compatible par la voie habituelle. |
| workspace_not_git | Choisir un dossier autorisé dans un dépôt Git non nu. |
| workspace_repo_unsupported | Garder shared pour ce dépôt à configuration non prise en charge. |
| workspace_base_unavailable | Donner une référence locale qui résout un commit. |
| workspace_branch_invalid | Donner un nom de branche Git valide. |
| workspace_branch_exists | Donner une branche nouvelle, sans réécrire l’existante. |
| workspace_dir_not_ignored | Faire ignorer `.worktrees/` par le dépôt. |
| workspace_limit | Examiner et nettoyer les worktrees conservés avant une nouvelle demande. |
| workspace_creation_in_progress | Attendre puis rejouer les mêmes arguments avec la même clé. |
| workspace_path_unsafe | Examiner le chemin ; les restes éventuels sont nommés dans le refus. |
| workspace_create_failed | Examiner le refus ; les restes éventuels sont nommés et conservés. |

Les limites 149 restent applicables. Claude/GLM vers Claude/GLM avec worktree est
refusé quand l’héritage exige le même dossier. Codex vers Claude en développement
exige accès complet et approbation never selon la règle existante. Un enfant
Codex workspaceWrite ne reçoit pas de droit sur le dossier Git commun. Il peut
conserver ses modifications dans le worktree sans pouvoir les commiter.

À chaque ronde, Bridget lit les faits Git avant de publier le résultat : base,
tête, branche, nombre de commits, les 50 derniers, modifications non commitées et
tête sur la branche attendue. `bridget_task_status.workspace` porte ces faits.
Une tête détachée ou une autre branche donne head_on_branch égal à false. Les sujets
restent du texte enfant. Une imitation du préfixe d’autorité dans ce texte est
neutralisée ; Bridget ajoute son unique ligne de fin. Les faits structurés
font foi. Si la lecture Git échoue, la ligne indique « nombre de commits
indisponible » et les indicateurs concernés valent « indisponible ». Les faits
restent inconnus. La tâche ne reste pas bloquée sur cette lecture.

L’état kept signifie dossier et branche conservés. Il permet les écritures
et les rondes suivantes selon les droits hérités. Une nouvelle ronde vide les
faits courants puis les recalcule ; les résultats déjà publiés restent stables.
Un résultat avec garde 153 ne force pas l’arrêt de l’enfant. Échec et annulation
attendent l’arrêt confirmé pour rapporter tout l’arbre. Le nettoyage logique
et la purge attendent les conditions 150/153 ; aucun fichier Git n’est supprimé.

En mode shared en développement, shared_writers apparaît dans le reçu si un
autre enfant actif écrit dans le même dossier. Cet avertissement ne bloque
rien. La détection file_collision garde son comportement.

Bridget crée le worktree et lit son état Git. Il ne fait ni commit, fusion,
rebase, push, fetch, pull request, suppression de branche ou de worktree.
Le nettoyage appartient à l’humain. Exemple pour un dépôt et un chemin issus
du reçu, à adapter après examen du travail :

```sh
git -C /Users/moi/Nextcloud/10.Scripts/64.bridget worktree list
git -C /Users/moi/Nextcloud/10.Scripts/64.bridget branch --list 'bridget/*'
# Choisir le chemin exact du reçu après avoir conservé/intégré le travail.
git -C /Users/moi/Nextcloud/10.Scripts/64.bridget worktree remove /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/1a2b3c4d
git -C /Users/moi/Nextcloud/10.Scripts/64.bridget branch -d bridget/1a2b3c4d
```

Ces commandes humaines refusent le retrait sale ou une branche non intégrée.
Ne pas ajouter force pour contourner ce refus.

## Demandes de l’enfant — contrat155

Le contrat155 est couvert par les tests ciblés des outils, des vues et de
la liste MCP sûre. La livraison reste distincte de ces preuves. Vérifier les
capacités et outils exposés par la session avant de l’utiliser ; ce texte
ne prouve aucune activation runtime.

### Choisir le mode

`child_requests` est facultatif sur `bridget_delegate`. Son défaut est `deny`.
`"forward"` demande une décision externe pour les seules formes Codex prises
en charge. Le mode reste figé dans la tâche et dans l’enveloppe de rejeu :
conserver la même clé et les mêmes arguments. Les autres fournisseurs sont
refusés avec `child_requests_unsupported`. Une politique incomplète ou un
relecteur autre que `user` rend `child_requests_reviewer_unsupported`.

Ajouter `"child_requests":"forward"` à l’appel de délégation d’une cible Codex
annoncée par le catalogue. Les droits du parent doivent déjà être attestés.
Aucun droit supplémentaire ou grant par mission n’est créé. La limite absolue
vaut aussi pour l’humain. `approval_policy=never` reste inchangée : `forward`
n’ouvre aucune route humaine si le fournisseur ne produit pas d’approbation.

| Demande | Route155 |
|---|---|
| `item/tool/requestUserInput` complète, corrélée, `isSecret=false` (défaut normatif) | Question au parent propriétaire attesté |
| Approbation de commande, `kind=command`, faits complets, effet courant et décision explicitement offerte | Décision humaine dans les droits figés, parent `dangerFullAccess` attesté et enfant non restreint ; `accept` une fois |
| `fileChange` | Refus `approval_method_unsupported` ; `grantRoot` non nul : `duration_unsupported` |
| `permissions`, `writeStdin`, méthode ou effet non établi | Refus fermé, sans réponse d’accord inventée |
| Tout formulaire MCP tiers, même typé, complet ou booléen | Refus `approval_method_unsupported` ; aucune preuve structurée de non-secret ou de non-approbation |

Les modes URL restent refusés. L’exception historique du formulaire vide du
serveur Bridget reste distincte ; elle ne s’étend pas aux serveurs tiers.
Une question ne monte que vers le parent direct. Le parent ne peut jamais
approuver une action et l’humain ne répond pas aux questions. Les chemins
cités ne prouvent aucun confinement et ne sont pas résolus dans le filesystem.

Le dossier fournisseur `cwd` est cité, sans lecture ni résolution filesystem.
Une chaîne est conservée entière jusqu’à 1024 octets UTF-8 et affichée avec
Debug (`{:?}`), entre guillemets et avec les contrôles/bidi visibles. Absence
ou `null` donne `facts.cwd=null` ; `facts.cwd_verified=false` dans tous les cas.
Un autre type ou une chaîne hors borne donne `details_incomplete`. La borne
1024 octets s’applique séparément à la commande et à cwd. Le budget total des
faits reste 4096 octets ; son dépassement reste un refus fermé. Aucun fallback
vers task.cwd, dossier du parent ou dossier courant n’est fabriqué. Ce dossier
cité ne prouve ni l’exécution dans ce dossier ni un confinement ou un droit.


### Répondre à une question

Lire l’avis ou `bridget_task_status`, puis reprendre les identifiants exacts.
`bridget_task_respond` accepte seulement les clés suivantes :

| Clé | Type et règle |
|---|---|
| `task_id` | Chaîne obligatoire ; tâche de l’appelant propriétaire attesté |
| `request_id` | Chaîne obligatoire ; identifiant de la demande, pas clé de délégation |
| `decision` | Chaîne obligatoire : `accept`, `decline` ou `cancel` |
| `answer` | Objet seulement pour une question avec `accept`, 8 Kio au plus ; clés égales aux identifiants proposés, valeurs tableaux de chaînes |

Tout champ inconnu est refusé. Exemple de forme d’appel, avec les identifiants
rendus par Bridget :

```json
{"name":"bridget_task_respond","arguments":{"task_id":"<task_id>","request_id":"<request_id>","decision":"accept","answer":{"<question_id>":["<réponse>"]}}}
```

Pour refuser, utiliser `decline` ou `cancel` sans `answer`. Ne pas ajouter une
identité, une permission, une décision durable ou une réponse à une approbation.
Le parent qui répond à une approbation reçoit `parent_cannot_approve`.

Le mapping Bridget est `answer[question_id]=[chaînes]`. À la frontière fournisseur,
le transport produit `answers[question_id].answers=[chaînes]`. Le parent ne doit
pas envoyer cette enveloppe fournisseur dans `answer`. `user_input` n’a aucun
refus `decline/cancel` natif : le transport écrit l’erreur JSON-RPC fermée
`child_request_refused` pour ces décisions et à l’expiration.

Une réponse réussie suit cette forme :

```json
{"version":1,"request_id":"…","state":"answered","outcome":"answered_by_parent","decision":"accept"}
```

### Délai, état et attente

La fenêtre est de 600 s pour une question Parent et de 3600 s pour une décision
Human. L’échéance ne dépasse jamais `mission_deadline_at - 30 s`. Moins de 30 s
restantes produit `deadline_too_close`. Les fenêtres ne sont pas un octroi de
droits ni le délai d’appel `wait_timeout`.

`bridget_task_status` ajoute `pending_requests` et les dix dernières issues dans
`recent_requests`, avec `delivery_state`. Les demandes Human sont visibles mais
le parent ne peut pas y répondre. Une décision historique ne prouve pas sa remise :
`none`, `pending`, `sent`, `delivered`, `void` et `unknown` distinguent son état.

Dans `bridget_task_wait`, `ready` garde seulement les tâches terminales et
`pending` les identifiants nonterminaux. La clé `pending_requests` est à la racine
du résultat wait, avec `task_id` et la projection de chaque question Parent
ouverte, non expirée et actionnable. Une demande Human seule ne rend pas la main.
`timed_out` est faux si `ready` ou `pending_requests` est non vide ; il est vrai
seulement à l’échéance wait lorsque ces deux listes sont vides.
Une question Parent déjà ouverte peut revenir immédiatement. L’attente ne
consomme aucune demande et n’a aucune mémoire persistante de waiter.

Pendant une pause, `Respond` rend `referent_paused`. Une décision humaine reçue
pendant la pause est acquittée sans exécution ni accord différé. Après reprise,
une réponse réussie concerne une nouvelle demande. Un accord invalidé n’est
jamais remis après reprise. Sans accusé fiable, une issue `unknown` n’est pas
présentée comme une annulation confirmée.

Refus utiles : `request_unknown`, `request_closed`, `request_expired`,
`answer_invalid`, `decision_not_offered`, `parent_cannot_approve`,
`native_owner_identity_mismatch` et `permission_not_inherited`. Un rejeu de
réponse identique rend l’état conservé ; une autre décision sur une demande
close rend `request_closed`. Ne pas contourner un refus par une nouvelle route.

## Rondes et brief - contrat153

Pour faire corriger un travail par le même enfant, ajouter `keep_alive_secs`
à `bridget_delegate`. Ce nombre entier va de 60 à 7 200 secondes. Sans ce
champ, le nettoyage habituel suit le résultat. Vérifier le catalogue réellement
chargé : les sources ou fichiers installés ne prouvent pas l'activation 153.

Avec cette garde, l'enfant reste disponible après l'envoi de son résultat.
La tâche reste `result_available`. `idle_deadline_at` indique la fin de la
garde, calculée dès cet envoi. Ce délai diffère de `reply_timeout`, qui borne
la réponse à chaque mission. Une lecture ou une attente ne prolonge aucun délai.

Après le résultat, ouvrir la ronde suivante avec le même `task_id` :

```json
{"name":"bridget_task_continue","arguments":{"task_id":"<task_id rendu>","request_id":"corrections-r2","task":"Corrige les deux défauts signalés et rends les preuves.","brief":{"objective":"Corriger le contrat relu","summary":"La première revue a identifié deux défauts. Applique seulement ces corrections."}}}
```

Choisir une nouvelle `request_id` pour chaque ronde. Si sa réponse se perd,
reprendre exactement cette clé et ces arguments. Le rejeu rend la vue courante
sans ouvrir une ronde de plus, même pendant la pause. Une enveloppe différente
rend `envelope_mismatch`. Ne pas changer de clé après une issue inconnue.

Une tâche admet au plus huit rondes, première mission comprise. L'enfant,
son instance, le dossier, le modèle et la posture restent les mêmes.
Chaque ronde reçoit une mission et un message de résultat distincts. La vue
de tâche et `bridget_task_wait` concernent la ronde courante. À son ouverture,
le résultat courant devient vide ; les messages de résultats déjà publiés restent
distincts. Les lectures seules ne déclarent pas la remise.

Le statut ajoute `round`, `idle_deadline_at` et `continuable`. Ce dernier
signale un résultat envoyé avec une garde encore présente. L'appel revérifie
le délai, la présence de l'enfant, la pause, le plafond et les droits. Un
indicateur vrai ne garantit donc pas l'admission de la prochaine ronde.
Une source de droits changée ou une politique actuelle différente refuse
la ronde. Aucun champ de sécurité n'est écarté pour permettre la continuation.

Pour finir, appeler `bridget_task_cancel` avec le `task_id`. Sur un résultat
disponible, la fin est durable : l'enfant est nettoyé, le statut et le résultat
sont conservés. Pendant une ronde active, l'annulation habituelle s'applique.
L'expiration, l'arrêt humain ou le nettoyage du parent terminent aussi la garde.
Un redémarrage du daemon la termine ; aucune nouvelle ronde n'est alors possible.
Un enfant inactif tient sa place, parmi les 16 par instance et 128 au total,
jusqu'à la confirmation de son arrêt.

`brief` est facultatif sur Delegate et Continue. Il suit le brouillon de
`bridget_handoff` : `objective` et `summary` sont obligatoires. Résultats,
décisions, questions, prochain pas, références et limites sont facultatifs.
Le dossier rendu fait au plus 16 384 octets, sans troncature. Un objet invalide,
un champ inconnu ou `null` est refusé. Le dossier précède la consigne dans
la même mission ; il n'envoie pas un message de passation séparé.

Rédiger ce dossier depuis son contexte autorisé. Les sources citées ne sont
pas vérifiées (`sources_not_verified`). Bridget ne les lit pas. Le journal
peut être lisible plus largement que l'enfant : aucun secret dans le brief.
Pour une correction, transmettre le delta utile et ses preuves, sans recopier
tout l'historique.

| Nouveau refus | Suite utile |
|---|---|
| `brief_invalid` | Corriger le champ indiqué ; aucune clé n'est consommée. |
| `keep_alive_out_of_range` | Choisir un entier de 60 à 7 200 à la délégation. |
| `keep_alive_not_enabled` | Cette tâche n'a pas de garde ; créer une nouvelle délégation si nécessaire. |
| `keep_alive_ended` | La garde est terminée ou l'enfant est absent ; cette tâche ne peut plus continuer. |
| `round_in_progress` | Attendre la fin de la ronde courante. |
| `result_delivery_pending` | Attendre environ une seconde, puis reprendre le même appel et la même clé. |
| `round_limit` | Huit rondes ont été ouvertes ; créer une nouvelle délégation si nécessaire. |

Le même processus ne garantit pas qu'un vrai modèle se souvienne de tous les
faits. Le contexte accumulé peut aussi renforcer un biais. Pour une contre-revue
indépendante, créer une nouvelle délégation avec son propre brief.

## Attendre un résultat ou une réponse - contrat 151

Le parent peut attendre dans le même tour, sans relire le statut en boucle.
Ce contrat exige un daemon et un serveur MCP qui prennent en charge la session 151.
Les fichiers installés seuls ne prouvent pas que ces processus ont été rechargés.

`bridget_delegate` et `bridget_send` gardent leurs reçus immédiats. Conserver le
`task_id` de la délégation, ou l'`id` de l'envoi avec `reply: true`. Appeler ensuite
l'outil d'attente correspondant. Ces outils et `bridget_task_status` sont des lectures
pures : ils ne marquent aucune livraison, ne suppriment aucun message et ne provoquent
aucun nettoyage ni aucune purge. La remise du résultat suit toujours le chemin normal.

### Attendre une ou plusieurs tâches

```json
{"name":"bridget_task_wait","arguments":{"task_ids":["<task_id>"],"wait_timeout":600}}
```

`task_ids` contient 1 à 16 identifiants distincts de tâches du parent. `wait_timeout`
est un entier en secondes, de 1 à 3600 ; son défaut est 600. Tout champ inconnu ou
type incorrect est refusé. Une tâche inconnue, purgée ou détenue par un autre parent
fait refuser tout l'appel avec `task_unavailable`.

La réponse porte `version`, `timed_out`, `waited_secs`, `ready` et `pending`.
Elle revient dès qu'au moins une tâche est `result_available`, `failed` ou `cancelled`.
`ready` regroupe toutes les tâches terminées à cet instant ; `pending` contient les
autres identifiants. Les deux listes suivent l'ordre de `task_ids`. Une tâche déjà
terminée permet un retour immédiat. Si aucune tâche ne finit avant le délai,
`timed_out` vaut `true`. Les tâches continuent : l'expiration ne les annule pas.

L’extension155 ajoute le retour sur une question Parent actionnable ; voir Demandes de l’enfant.

Les entrées de `ready` omettent le texte complet `result`. Le parent le lit avec
`bridget_task_status` et le même `task_id`, ou par le message de résultat.
Le statut conserve toutes ses clés existantes, dont `result` et `error`.
Il ajoute les champs suivants :

| Champ | Sens |
|---|---|
| `work_state` | `working`, `waiting_for_children`, `result_available`, `failed` ou `cancelled`. |
| `summary` | Début du résultat publié, 2048 octets au plus, avec une coupe sur une frontière UTF-8. `null` hors `result_available`. |
| `summary_truncated` | `true` si le résumé omet une partie du résultat. |
| `result_bytes` | Taille du résultat en octets, ou `null` si aucun résultat n'est conservé. |
| `detail` | `{"tool":"bridget_task_status","task_id":"…"}` pour `result_available` et `failed` ; sinon `null`. |
| `mailbox` | État calculé de la boîte de fin, ou `null` pour une tâche non terminée ou annulée. |

`queued`, `starting`, `mission_pending`, `working` et `cancelling` donnent le même
`work_state: "working"`. Les quatre autres états gardent leur nom. Le résultat
publié est figé pour cette ronde : deux lectures donnent le même résumé.

### Lire la boîte de fin et déclarer la lecture

| `mailbox.state` | Fait représenté |
|---|---|
| `pending` | Résultat ou avis d'échec prêt, encore sans envoi engagé. |
| `claimed` | Envoi engagé, sans accusé de remise prouvé. Une phase indéterminée, orpheline ou absente reste dans cet état. |
| `delivered` | Remise accusée : phase `acked` ou preuve durable `delivery_acked`. |
| `acknowledged` | Le parent a déclaré la lecture par `bridget_task_ack`. Ce fait ne prouve pas la remise du message. |

Une tâche déjà purgée par la rétention 150 n'a plus de boîte consultable.
L'outil rend `task_unavailable`, sans entrée `mailbox`.

Pour déclarer explicitement la lecture après avoir reçu le contenu :

```json
{"name":"bridget_task_ack","arguments":{"task_ids":["<task_id>"]}}
```

Cet outil accepte 1 à 16 identifiants distincts de tâches du parent, en
`result_available` ou `failed`. Il écrit `acknowledged_at` une seule fois.
La réponse sépare `acknowledged` (nouveaux accusés) et `already` (déjà déclarés),
avec `version: 1`. Une tâche non terminée ou annulée donne `task_not_ready` pour
tout l'appel, sans accusé partiel. Un identifiant inconnu ou étranger donne
`task_unavailable`. Cet accusé de lecture est informatif : aucun effet sur la
livraison, les messages, le nettoyage, la purge ou la projection Lineage.

Un résultat peut apparaître deux fois : résumé rendu par l'attente, puis message de
résultat. Le message reste envoyé même si l'attente a rendu le résumé ou a expiré.
L'accusé de lecture ne supprime pas cette copie. La rétention 150 continue à exiger
l'accusé de remise du message pour purger un arbre.

### Attendre la réponse d'un envoi suivi

```json
{"name":"bridget_request_wait","arguments":{"request_id":"<id_rendu_par_bridget_send>","wait_timeout":600}}
```

La demande provient d'un `bridget_send` avec `reply: true`. Seul son émetteur peut
l'attendre. `wait_timeout` a les mêmes bornes de 1 à 3600 secondes et le même défaut
de 600. Il ne remplace pas le `reply_timeout` de l'envoi.

La réponse porte `version`, `state`, `timed_out_wait`, `waited_secs`, `reply` et
`reply_reason` :

| `state` | Sens |
|---|---|
| `answered` | La demande est répondue. `reply` contient le résumé conservé, s'il existe. |
| `open` | Le délai de cet appel d'attente a expiré. `timed_out_wait` vaut `true` ; la demande reste ouverte. |
| `timed_out` | Le délai de réponse de la demande est passé. Cet état peut être calculé sur une ligne encore `open`, sans écriture. |
| `cancelled` | L'émetteur a annulé la demande par `bridget_cancel`. |

`reply`, lorsqu'il existe, contient `id`, `from`, `summary` et `summary_truncated`.
Son résumé garde 1024 octets de corps au plus, sur une frontière UTF-8. Il est figé
avec le premier passage à `answered`, dans la même transaction. Il ne dépend pas
du journal des messages (ledger), dont une ligne peut être remplacée.

Une demande `answered` sans résumé conservé rend `reply: null` et
`reply_reason: "reply_unavailable"`. Cela couvre le guichet, les lignes antérieures
à 151 et une enveloppe indisponible. Si le guichet clôture d'abord, une réponse de
message tardive ne remplace pas ce `null`. Hors de ce cas, `reply_reason` vaut `null`.

La réponse arrive aussi par le chemin normal d'injection. Elle peut donc être visible
deux fois : cette limite est FR-030. Après une coupure de l'hôte, rappeler
`bridget_request_wait` avec le même `request_id`. Cet appel n'envoie aucun message.

### Délais, disponibilité et refus

Les attentes se réveillent sur les changements. Un filet interne de 5 secondes
reste borné par le temps restant ; pour une demande, par son échéance aussi.
La pause du référent laisse les attentes possibles. Le client MCP garde sa connexion
pendant `wait_timeout + 10` secondes.

Il peut y avoir 4 attentes par processus MCP et 64 dans le daemon, tâches et demandes
réunies. Les autres créneaux MCP restent disponibles pour les appels de contrôle.
Une annulation MCP ferme le socket de l'appel, y compris pendant la sonde de support.
La fermeture de l'entrée MCP (EOF) ferme les attentes avant de joindre leurs fils.
Fermeture du client, annulation MCP ou arrêt du daemon libèrent l'attente en 6 secondes
au plus. Si le daemon redémarre, l'appel échoue avec `daemon_unreachable` côté client ;
le parent peut rappeler l'attente, sans renvoyer la mission ou la demande.

Avant une attente ou un accusé de lecture, la façade sonde `DaemonIdentityRequest`
avec un budget de 3 secondes. Le daemon annonce `features: ["wait_v1"]`.
Sans `wait_v1`, un ancien daemon donne `wait_unsupported` et ne reçoit aucune trame
d'attente ou d'accusé nouvelle. Un outil absent du catalogue peut aussi signaler
un serveur MCP ancien ; une installation de fichiers ne recharge pas ce processus.

Le catalogue annonce `wait_timeout_default_secs: 600` et
`wait_timeout_max_secs: 3600`. Le réglage `tool_timeout_sec = 3700` s'applique au seul
montage Codex de Bridget. Dans un autre hôte ou montage, choisir un `wait_timeout`
inférieur au délai d'outil de cet hôte. Une coupure reste récupérable par un nouvel
appel d'attente en lecture pure.

| Refus | Cause |
|---|---|
| `InvalidParams` | Champ inconnu ou type incorrect dans les arguments MCP des trois outils. |
| `invalid_request` | Identifiants de tâche invalides, liste vide, plus de 16 éléments ou doublon ; demande invalide. |
| `wait_timeout_out_of_range` | `wait_timeout` hors de 1 à 3600 secondes. |
| `wait_unsupported` | Le daemon n'annonce pas `wait_v1`. |
| `task_unavailable` | Tâche inconnue, purgée ou détenue par un autre parent. |
| `request_unavailable` | Demande inconnue ou émise par un autre agent. |
| `wait_capacity` | Les 4 attentes du processus MCP ou les 64 attentes du daemon sont occupées. |
| `task_not_ready` | Accusé de lecture demandé pour une tâche non terminée ou annulée. |

Pour l'attente de tâches, l'ordre daemon est `invalid_request`,
`wait_timeout_out_of_range`, `wait_unsupported`, `task_unavailable`, `wait_capacity`.
La façade peut refuser sa capacité avant de valider les arguments. Les refus
n'écrivent aucun état. Les erreurs d'identité restent celles de la session 149.
La session 151 n'ajoute aucune commande CLI humaine.

## Droits du parent et de l'enfant

Le parent doit être inscrit dans Bridget avec une identité et un projet attestés.
Il ne choisit pas son identité dans les arguments des outils. L'enfant reçoit
la mission demandée et les faits natifs d'identité et de Git utiles à sa reprise.
Ces faits sont joints au premier tour. Il ne reçoit ni la conversation du parent
ni ses credentials T3.

Le catalogue indique le périmètre du dossier avec `cwd_scope` :

- `project` : le parent a une preuve de droits. La mission doit rester dans
  `cwd_root`, c'est-à-dire le dossier effectif du parent. Un dossier hors de cette
  racine est refusé (`cwd_outside_parent_project`).
- `root` : un grant humain explicite fixe la racine.
- `same_project` : parent externe avec projet attesté, sans grant. `cwd_root` vaut
  `null`. Ce parent peut inspecter ce dépôt et ses worktrees.

Un dépôt étranger reste refusé. Le dossier `.git` ne sert jamais de racine de droits.

**Posture (facultative) :**

- **Absente (cas normal) :** l'enfant hérite de la politique que le parent a prouvée.
  Bridget n'ajoute aucun droit et ne demande aucun grant. Si la cible ne peut pas
  garder cette politique, le motif figure dans `inherit_refusal` du catalogue.
- **`discovery` :** avec un parent attesté, la mission reste en lecture seule. Sans
  preuve, cette valeur suit le chemin 148 : un grant humain existant est requis,
  sinon `delegation_grant_required`.
- **`development` :** avec un parent attesté, l'écriture n'est accordée que si la
  politique du parent et la cible le permettent. Sans preuve, la demande est refusée
  avec `permission_attestation_unavailable`. Le motif éventuel figure dans
  `development_refusal`.

**Ce qui passe d'un parent à une cible (149) :**

- **Codex vers Codex :** Bridget garde la politique d'approbation, le bac à sable et le
  réseau du parent. Un parent en lecture seule ne donne pas `development`
  (`permission_not_inherited`). Un bac à sable externe est refusé
  (`provider_confinement_unavailable`).
- **Codex vers Claude ou GLM avec écriture :** seulement si le parent est en `dangerFullAccess` avec
  approbation `never`. L'enfant passe en `bypassPermissions`, dans la seule définition
  de cette mission. Un parent en `workspaceWrite` est refusé
  (`provider_confinement_unavailable`). Un parent en lecture seule donne une mission en
  mode plan, avec `Read`, `Glob` et `Grep`.
- **Claude ou GLM vers Claude ou GLM :** la mission réutilise les mêmes entrées : même
  lanceur, même profil, mêmes sources et mêmes empreintes. Le dossier de l'enfant doit
  être celui du parent. Les règles `allow` et `deny` du parent restent en vigueur.
  Bridget n'ajoute aucun bypass universel. Si une entrée change, la demande est refusée
  avec `settings_revision_changed`. Une source opaque, par exemple une politique gérée
  non observable, donne `permission_source_unavailable`.
- **Claude ou GLM vers Codex :** seulement si la politique est un bypass complet sans
  règle, ou une lecture seule limitée à `Read`, `Glob` et `Grep`.

Dans tous les autres cas, la réponse est `permission_mapping_unavailable`. Tous ces
refus arrivent avant la création de la tâche : aucun enfant n'est lancé.

Un humain peut accorder ou retirer des droits natifs à une instance avec
`bridget delegate-grant`, dans un terminal humain. Le grant porte sur l'instance,
une racine absolue et une posture maximale. Le MCP ne crée jamais ce grant.

Un grant n'est requis que pour la voie 148 : `discovery` sans preuve de droits du
parent. Une délégation héritée d'un parent attesté n'en demande aucun.

La révocation reste prioritaire. Elle ferme les nouvelles demandes du même agent,
même après changement d'instance, héritage compris (`delegation_grant_required`).
Seul un nouveau grant humain explicite lève ce refus durable. Elle ne transforme pas
le rejeu d'une demande déjà acceptée en une nouvelle mission.

## Identité dans T3

Deux conversations peuvent partager un processus fournisseur. Chaque montage
MCP Bridget reçoit une preuve privée distincte. Bridget la vérifie auprès du
serveur T3 à chaque opération, puis vérifie son inscription vivante au daemon.
Une preuve partielle, périmée ou révoquée produit un refus explicite.
Bridget n'emprunte jamais l'identité d'une conversation voisine.

Un credential retiré, révoqué ou tourné ferme tous les outils Bridget de cette
session avec `t3_session_unavailable`. Ce refus n'a ni repli par PID ni repli en
lecture seule. Avec T3 Code `0.0.45-local.166` ou plus récent, une reprise Codex
compare la preuve privée courante à la preuve montée avant le premier tour. Si
elles diffèrent, T3 reprend le même fil natif et remonte Bridget avec la preuve
courante. Il attend la fin de ce remontage avant de publier les droits et avant
`turn/start`. Un échec bloque le tour ; il ne réutilise pas l'ancienne preuve et
n'emprunte pas celle d'un autre fil. Une session qui n'a jamais eu de fait de
droits garde l'identité seule (version 1) pour les lectures et le rejeu d'une
demande déjà admise. Une admission héritée ou `development` exige la version 2 :
sans elle, la réponse est
`permission_attestation_unavailable`, sans grant ni repli vers `discovery`.

Les options MCP humaines existantes et les désactivations restent prioritaires.
Les outils148 deviennent disponibles au chargement des nouveaux processus.
Une installation de fichiers seule ne recharge pas une session déjà ouverte.
Les comportements 149 exigent le daemon et le binaire 149 chargés. Ce guide ne
prouve pas leur activation.

Au redémarrage, Bridget réconcilie la commande, l'instance et le lien natifs depuis
les faits attestés. Une incohérence native non réconciliable fait échouer la tâche
avec `native_restart_identity_mismatch` ; le daemon poursuit son démarrage.
Cette garantie couvre un marqueur lisible. Un marqueur corrompu ou illisible garde
le refus sûr de la flotte ; sa réparation générale ne fait pas partie du contrat152.
Une incohérence de cible d'exécution ou une erreur de stockage peut encore
bloquer le démarrage. Bridget conserve ses contrôles de propriété et de clôture.
Les erreurs `parent_task_unavailable`, `native_owner_identity_mismatch`,
`native_execution_changed` et `native_execution_mismatch` exigent aussi un diagnostic
des données conservées. Ne supprimez pas les marqueurs pour forcer une reprise.
Si un wrapper natif reste vivant après le délai commun de 8 secondes, le daemon
refuse aussi de démarrer. Il conserve le marqueur et ne force pas cet arrêt.

Si le suivi durable d'une mission admise manque, Bridget ferme son exécution
avec les contrôles existants avant de rendre `request_unavailable`. Une erreur
de lecture ou de clôture reporte le traitement. Elle ne crée pas de date supposée.
Cette fermeture termine aussi la pause liée. Elle ne crée aucune reprise.

## Suivi humain et panneau Lineage

Le terminal humain lit une tâche avec `bridget lineage inspect`, `watch` et `cancel`.
Ces commandes demandent `--t3-thread` (fil T3 du parent) et `--project-root` (chemin
absolu). Elles n'utilisent pas le MCP. Le fil T3 sert à la lecture : il ne crée pas la
tâche.

Dans T3, le panneau Lineage affiche l'arbre des tâches du fil parent : la tâche
racine, puis les enfants imbriqués. Une tâche se lit dans son journal et son résultat.
Ce panneau n'est pas un fil de conversation. Il n'a pas de zone de saisie, et l'enfant
ne reçoit aucun tour. Le bouton Arrêter demande l'annulation native de la tâche et de
ses descendants.

Statuts natifs : `queued`, `starting`, `mission_pending`, `working`,
`waiting_for_children`, `cancelling`, `result_available`, `failed`, `cancelled`. Un
statut inconnu ferme le contrat.

Preuve : l'interface web a été testée avec un daemon simulé. Aucune preuve ne couvre
le daemon réel, l'application de bureau ou une installation. Ne pas annoncer cette
fonction comme livrée.

## Délai de réponse choisi

`reply_timeout` fait partie de l'enveloppe de la demande. Rejouer la même clé avec un
autre délai produit `envelope_mismatch`. Omettre le champ et envoyer `3600` sont deux
enveloppes différentes.

Une valeur hors de 60 à 86 400 est refusée. Bridget ne la corrige pas.

Un enfant qui fixe un `reply_timeout` ne peut pas dépasser le temps restant effectif
de sa tâche parente. Ce temps exclut les pauses. Sans délai explicite, ce contrôle
reste celui du contrat149. Le suivi de la demande compte depuis l'envoi de la mission,
avec un délai explicite ou omis. Il ne compte pas le retard du lancement.
Les tâches déjà actives sans délai choisi gardent leur délai de 3600 secondes.

La date brute `mission_deadline_at` reste la date durable. Elle ne donne pas à elle
seule le temps restant après une pause. `bridget_request_wait` utilise le délai métier
effectif d'une demande native, sans changer sa date ni son état.
Le budget `wait_timeout` de l'appelant continue pendant la pause. Une demande ordinaire
garde le comportement151.

## Refus

Le refus revient dans `native_delegation_refused`, avec son code. Aucun refus ne crée
de tâche. Les cinq refus définitifs ci-dessous engagent la clé de rejeu pendant
30 jours. Tout autre refus reste sans reçu enregistré par le contrat152.

| Code | Cause |
|---|---|
| `referent_paused` | Le référent est en pause et la clé est nouvelle. |
| `control_state_unavailable` | L'état de pause est illisible : Bridget refuse plutôt que de déléguer pendant une pause possible. |
| `request_expired` | La tâche de cette clé a été purgée depuis moins de 30 jours. |
| `retention_saturated` | 4096 tâches sont conservées et aucun arbre n'est purgeable. |
| `reply_timeout_out_of_range` | `reply_timeout` est hors de 60 à 86 400. |
| `reply_timeout_exceeds_parent` | Le délai explicite d'un enfant dépasse le temps restant du parent. Sans délai explicite, aucun contrôle (comportement 149). |

Ordre des contrôles : identité, forme de la demande, bornes du délai, rejeu
(`envelope_mismatch`, `request_expired`), pause (`referent_paused`, ou
`control_state_unavailable` si l'état de pause est illisible), droits, `task_limit`,
`retention_saturated`, lignée du parent, délai de l'enfant.

`task_limit` reste pour 128 tâches actives, ou 16 enfants actifs par instance.
L'historique cumulé ne le provoque plus.

### Reçus des refus définitifs

| Code conservé | Cause |
|---|---|
| `cwd_outside_parent_project` | Le dossier demandé sort du projet autorisé du parent. |
| `cwd_unavailable` | Le dossier demandé ne peut pas être résolu. |
| `model_unavailable` | Le modèle demandé est absent du catalogue autorisé. |
| `effort_unavailable` | L'effort demandé est absent du catalogue autorisé. |
| `agent_type_unknown` | Le type demandé est inconnu du registre. |

Le rejeu de la même clé avec les mêmes arguments rend le premier refus, même si
le dossier ou le catalogue a changé. Une autre enveloppe rend `envelope_mismatch`.
Une tâche déjà admise prime sur un reçu de refus. L'entretien retire les reçus
âgés de plus de 30 jours ; un reçu reste présent à 30 jours exactement.

La pause, la saturation, un défaut d'identité ou de droits ne sont pas enregistrés
comme refus définitifs. Le registre conserve seulement `agent_type_unknown`,
jamais son texte dynamique. Ne pas changer automatiquement de clé après une panne
de transport ou une issue inconnue.

## Pause du référent - contrat152

Pendant la pause, une nouvelle clé reçoit `referent_paused`. Rejouer une clé déjà
admise rend la tâche existante. Un reçu de refus définitif garde sa réponse.
La lecture du catalogue, la lecture de statut et les attentes restent disponibles.
Une admission refusée seulement pour la pause peut être rejouée après sa levée.

Une tâche `queued`, `starting`, `mission_pending` ou `working` garde son état
et rend `suspended: true` dans sa vue courante. Lineage garde ses statuts existants.
Bridget ne lance pas un enfant retenu en file et ne remet pas une nouvelle mission.
Une interruption due à la pause ne fait pas échouer la tâche.

Le délai métier de mission et le suivi de sa demande excluent les pauses depuis
leur admission durable. Une pause avant l'envoi ne retire pas du temps au suivi.
Ce calcul survit à un redémarrage, même avant le premier passage du moteur.

À la levée, une exécution encore `interrupting` attend de devenir `interrupted`.
La reprise garde le message et la mission d'origine, puis suit leur dernière
exécution. La seule levée de pause ne consomme pas le compteur de reprises après panne.

Un lancement déjà soumis continue pendant la pause. Son délai de 300 secondes
continue aussi ; il peut donc expirer. Son enveloppe reste identique sur rejeu.
Un lancement retenu avant sa première soumission reçoit sa date d'émission à la levée.

Les résultats et avis d'échec déjà acquis peuvent encore être remis. Le nettoyage
et l'annulation restent possibles. Annuler pendant la pause empêche la mission
de reprendre à la levée.

Ces comportements demandent un daemon chargé en version152. Une installation de
fichiers seule ne prouve pas l'activation. Un agent ne redémarre pas le daemon
ou le fournisseur pour obtenir ce comportement.

## Annulation vérifiée avant les arrêts

`bridget_task_cancel` demande l'annulation de la tâche et de ses descendants.
Bridget vérifie d'abord que tous les membres sont gérés et arrêtables.
Si un membre ne l'est pas, aucun arrêt n'est exécuté et aucun descendant n'est
marqué pour annulation. La racine reste `cancelling` avec
`native_cancel_unverifiable:<enfant>`. Le contrôle sera refait depuis cette racine.

Après un contrôle complet, les descendants sont arrêtés avant le parent.
Chaque arrêt confirmé est conservé. Un nouvel essai ou une reprise après panne
ne rejoue que les arrêts sans confirmation durable.
Un arrêt peut échouer après le contrôle initial ; les confirmations déjà acquises
restent conservées et la racine reste `cancelling`.
Une erreur ou un résultat terminal déjà acquis reste conservé pendant le nettoyage.

| Erreur de reprise ou d'annulation | Lecture |
|---|---|
| `native_resume_exhausted` | Deux reprises après panne ont déjà été décidées. |
| `mission_reply_timeout` | Le délai métier hors pause est dépassé. |
| `unreachable` | Les conditions de reprise ne sont pas réunies. |
| `native_restart_identity_mismatch` | L'identité native ne peut pas être réconciliée depuis les faits attestés. |
| `native_cancel_unverifiable:<enfant>` | Annulation bloquée ; racine `cancelling`. Au précontrôle refusé, aucun arrêt. Après un échec d'arrêt, les confirmations précédentes restent conservées. |

## Rétention des tâches

Bridget garde au plus 4096 tâches. Il purge par arbre : une tâche sans parent et tous ses
descendants. L'entretien horaire purge les arbres finis depuis plus de 7 jours, 128 au
plus par passage. Si 4096 tâches sont conservées, les arbres purgeables les plus anciens
partent d'abord, même avant 7 jours. L'entretien le fait dans le même quota de 128 arbres
par passage, partagé avec la purge d'âge. Une nouvelle délégation le fait aussi : elle
retire seulement les arbres nécessaires pour repasser sous 4096, sans limite de 128.

Un arbre n'est jamais purgé dans ces cas :

- une de ses tâches est active ou son nettoyage n'est pas fini ;
- une garde reste posée ;
- un résultat ou un avis d'échec n'a pas été accusé par le destinataire ;
- un résultat d'une ronde passée n'a pas encore son accusé de remise ;
- un de ses sous-agents est connecté, routé, en cours de lancement, en cours de
  décommission ou désiré `Running`.

Avant la purge, chaque sous-agent est retiré de la flotte. Une remise indéterminée n'a
pas d'accusé : elle bloque donc son arbre. Si ces arbres remplissent la borne, la
délégation est refusée avec `retention_saturated`.

`bridget_task_ack` déclare la lecture du parent. Il ne remplace pas l'accusé
de remise du message et ne rend pas un arbre purgeable. Une réponse authentifiée
à une ancienne mission est absorbée sans modifier la tâche ni réveiller le parent.

Après une purge, `bridget_task_status` sur cette tâche rend le refus existant de tâche
inconnue (`task_unavailable`). La vue Lineage fait une lecture complète sans tâche
fantôme. Le rejeu de la clé rend `request_expired` pendant 30 jours. La clé redevient
libre au passage d'entretien qui suit ces 30 jours (1 h au plus de décalage).

## Ce qui ne change pas

- Le lancement ordinaire (`bridget spawn`), ses postures et ses protections du
  terminal humain ne changent pas. Ce guide ne remplace pas leur tutoriel.
- `bridget delegate-grant` reste une commande humaine de terminal.
