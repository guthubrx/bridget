# ADR 160 — Reprise sur limite d’usage

Statut : **Acceptée** par ROOT après T001/T002, Analyze et corrections du plan R1.
Date de préparation : 2026-10-12. Ce texte ne prouve aucune mise en œuvre,
validation de test, activation ni observation d’un quota réel.

## Contexte

Le catalogue 157 observe des fenêtres de quota. Une mission déjà admise peut
échouer lorsque sa fenêtre est épuisée. Changer de fournisseur après admission
romprait la définition épinglée et le choix du parent. La reprise 152 traite
les pauses et les pannes ; elle exclut une exécution `failed` ordinaire.
T002 n’a vu aucun quota réel. La raison du refus et l’ordre réel quota/échec
restent inconnus ; les deux chemins sont établis par lecture seulement.

## Décision

Ajouter uniquement l’opt-in `resume_on_limit` à la délégation existante.
Une tâche `working` attend un retour futur dans 6 heures au plus, avec deux
rondes durables au maximum. Une décision sans fait dure 5 secondes observées.
La relance vise retour + 30 secondes, sous les gardes existantes. Elle conserve
mission, fournisseur, modèle, effort et définition épinglée. Le fallback 157
n’est pas recalculé. Les droits ne sont pas augmentés.

| Exigence d’entrée de la spec | Solution retenue pour la conception |
|---|---|
| Compteur durable | Liste `quota_waits` sans retrait ; une ronde soldée reste comptée. |
| Reconstruction ciblée de `failed` | `reconstruct_quota_failed` exige qualification, dernière feuille et identité inutilisée ; la reprise ordinaire 152 reste distincte. |
| Identité exacte de relance | `limit-resume:<task_id>:<ronde>:<retour>` liée à la continuation dans la même transaction ; canonique et date d’émission figés au rejeu. |
| Cause quota distincte | Qualification durable et cause `usage_limit_reset` ; aucun incrément de `restart_resumes` 152. |
| Gel sans double comptage | Union pause/quota commune au délai effectif, suivi de demande et reprise ; quota borné au retour + 30 secondes. |
| Gestionnaire `DeliveryRejected` | Sortie avant avis direct et retrait du suivi pendant la décision ou l’attente ; seul avis durable en échec final opt-in. |

## Ordre entre les deux magasins

1. Écrire la qualification de l’exécution échouée dans le magasin d’exécutions.
2. Ajouter l’attente dans le magasin de tâches. Une qualification orpheline ne
   sert jamais ; une attente sans qualification ne relance pas.
3. À l’heure admise, écrire `relaunching` dans la tâche.
4. Reconstruire la continuation et son identité dans une seule transaction du
   magasin d’exécutions. Le parent reste `failed`.
5. Réserver et graver la remise existante. Sa date d’émission est l’instant
   `observed_at` de la continuation ; le rejeu conserve le canonique.
6. Solder l’attente seulement après ACK. `dispatching` reste ouvert ;
   `indeterminate`, `orphaned` ou remise non reprenable échouent explicitement.

Une chaîne réservée par quota ne passe pas en parallèle dans la reprise 152.
Chaque frontière reprend à l’état durable trouvé, sans deuxième continuation.
T036 doit capter `execution_id` et `generation` depuis la liaison AVANT la
transition terminale du wrapper : celle-ci retire la liaison. Les champs sont
optionnels pour la compatibilité. Après une relance, un refus non corrélable
échoue explicitement ; un refus ancien ne devient pas celui de la nouvelle feuille.

## Conséquences et limites

La tâche reste `working`, avec `quota_wait` additif. Aucun avis d’échec ni rappel
pendant la décision ou l’attente. Le gel concerne l’enfant seul ; le parent ne
reçoit pas de délai supplémentaire. Pause et quota se chevauchent sans addition.
Le gel quota finit au retour + 30 secondes même si l’enfant est absent.
Les faits de quota sont volatils, les attentes qualifiées sont durables.
Un redémarrage pendant la décision de 5 secondes rend l’erreur d’origine.
L’ACK solde une remise, pas son résultat métier ni la mémoire d’un modèle réel.
L’annulation réconcilie la dernière feuille et la remise avant de fermer
l’attente. Si cette vérification échoue, la tâche reste `cancelling`.

Ne pas assimiler quota et panne, changer de fournisseur, additionner les gels,
ou considérer une remise écrite comme acquittée. Aucun nouveau moteur ni canal
de notification n’est prévu. Les tests et les frontières de crash prescrits
restent à prouver ; les 37 tâches existantes ne sont pas cochées par ce document.

Références de conception :
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/160-reprise-quota/spec.md
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/160-reprise-quota/contracts/reprise160.md
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/160-reprise-quota/data-model.md
Preuve de préparation en lecture seule :
/Users/moi/.cache/bridget-forensic160-20261012/forensic160-readonly.md

Précisions ROOT R1 : un faux positif peut répéter des effets externes ; aucune preuve de causalité ni exactly-once. Les limites Codex sans fenêtre et non-vu réel restent explicites. La sortie d’échec relaunching ferme la feuille via close_lost_execution avant Task.failed ; une erreur garde le checkpoint pour réessai. Past est limité à une observation fraîche pendant le hold, admission157 inchangée. Le canon relit le premier observed_at.

## Précisions de la contre-revue code R1
La quarantaine de `dispatching` sur échec ou annulation applique FR011b/FR017 ; elle ne décrit pas le solde ACK normal.
`indeterminate` est absorbant : aucun renvoi par Task.failed ; un ACK tardif ne modifie que `dispatching`.
Un enfant peut encore exécuter les effets déjà engagés alors que la Task parent est failed ; l’ACK tardif ne rouvre pas cette tâche. Aucune garantie exactly-once des effets externes n’est ajoutée.
Une erreur `defer` rendue en String reste classée Storage ; elle conserve le checkpoint au lieu d’inventer un refus métier.
La remise est gravée `dispatching` avant `defer` ; le retry relit cette phase sans réémettre, puis échéance ou reconnexion permet la clôture.
Ce cas ne constitue pas une cohorte Prepared ambiguë ni une autorisation de rejouer l’émission.
Des resumeIDs incompatibles indiquent une corruption étrangère : le refus protège de la clôture d’une autre exécution.
Le checkpoint peut alors rester bloqué indéfiniment ; l’erreur est locale à la tâche et ne bloque pas tout le tick.
