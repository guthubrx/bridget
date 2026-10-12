# Décision157 — catalogue fiable, quotas et remplacement explicite

Date : 2026-10-12. Statut : **Accepté**.

Cette décision fixe le contrat157. Elle ne prouve pas l’implémentation, les tests,
une latence atteinte ou l’activation du binaire. Les preuves sont conservées séparément dans la validation157.

## Contexte

Le catalogue et l’admission peuvent calculer des droits différents. Les refus
prévisibles de lancement arrivent après création de tâche. Les quotas observés
partent avec la présence. Le parent ne peut pas choisir une alternative ordonnée.
L’incident du 11octobre reste un scénario de code : aucun fait réel corrélé n’est
retrouvé, launch_context inconnu. Aucun fournisseur n’est sondé pour le confirmer.

## Décisions

- **D-A** : un verdict resolve_target commun au catalogue et à l’admission.
- **D-B** : extraire le préflight depuis submit_spawn et prepare_spawn_parts ;
  capacités/facturation/protocole/environnement/binaire réemployés, erreur
  build_environment propagée, définition épinglée. Cwd reste une garde du parent.
- **D-C** : mémoire volatile par agent_type/fenêtre, huit fenêtres au plus ; source
  et observed_at conservés. Purger les passées, puis évincer le plus ancien
  observed_at à la neuvième clé, départage lexical. Pas de table SQL supplémentaire.
- **D-D** : garder Err(String), enrichi code:JSON(message/detail), étendre le seul
  failure existant. Comparer les listes au code canonique. Codes nus et anciens
  préfixes restent compatibles ; JSON non conforme fermé, aucun porteur nouveau.
- **D-E** : request porte la cible effective, fallback vidé ; canonical garde la
  demande originale. Substitution durable requested/selected/reason/tried reste
  sur le rejeu. Probe sélectionne une seule cible/définition ; Reserve/Commit
  la consomment. Commit revalide le dossier enfant réel sans nouvelle sélection.
- **D-G** : mesurer T001/T012 avant d’optimiser. R8 prévoit une politique par
  fournisseur dérivée vers trois postures seulement si le rapport dépasse 2.
  Cette préparation ne prouve ni mesure finale ni gain d’optimisation.
- **D-H** : type/modèle/effort avant préflight/politique/options/quota. Aucun modèle
  ou effort invalide n’est masqué ; aucune augmentation des droits du parent.
- **D-I** : cohérence bornée au parent/posture/fait de session/cwd/état égaux,
  pour la famille verdict. Cwd est nullable. SameProject sans racine donne
  catalogue faux ; les admissions autorisées dans un projet concret restent.
- **D-J** : épuisement attesté par fenêtre avec retour futur et rejected Claude
  stream-json ou >=100 % utilisé toute source. Codex global atteint ne désigne
  aucune fenêtre ; limit_reached peut être vrai sans exhausted. Checks.auth:false.

Le catalogue conserve ses clés et ajoute can_run/constraints/quota, puis
checks/verdict_context/features à la racine. Constraints est vide si can_run,
sauf l’information unique provider_limit_reached_window_unknown. Fallback est
opt-in, 1 à 3 candidats fermés et ordonnés. Les neuf motifs remplaçables sont ceux
de FR013 ; autres refus initiaux rendus tels quels. Pour un candidat listé,
chaque refus est noté dans tried. fallback_exhausted/provider_quota_exhausted
exposent detail.tried/resets_at au MCP, sans tâche ni clé consommée.

Les gardes workspace154 et child_requests155/reviewer sont conservées avant
quota. Un refus de cible initiale sur ces options n’est pas remplaçable. Pinning
et diagnostics de chemin154 restent. Aucun remplacement après admission,
aucune autoreprise quota157, aucun grant ou choix humain durable nouveau.

## Conséquences et limites

Le parent reçoit un verdict borné et un diagnostic tôt, pas une promesse
d’authentification. Le catalogue ne vérifie ni login ni credentials fournisseur.
Le quota survit à la présence, mais est perdu au redémarrage ; les sources
republiquent. Deux types partageant un compte ne partagent pas cette mémoire.
Le changement de faits entre catalogue et admission peut produire un refus.
La mesure privée R8 est dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/157-catalogue-quotas/validation/mesure-catalogue.md. Elle ne prouve pas une latence chez un fournisseur réel.

La logique réemploie droits, registre, préflight, enveloppe et sources. Le module
provider_limits demandé a trois usages réels : écriture/catalogue/admission.
Aucune nouvelle dépendance, CLI, table, couche générique ou outil. Le remplacement
est une boucle bornée à quatre cibles, pas une bascule automatique en mission.

## Report en160 — six exigences d’entrée exactes

La reprise sur quota est reportée en160. Elle exige 152 livrée avec ses primitives
prouvées et 157 livrée. Texte normatif repris exactement de la spec157 :

1. **Compteur de rondes durable.** Le compteur `round` était porté par
   `limit_resume`, effacé après l'incrément. Le compteur doit survivre à
   l'effacement de l'attente, sinon la borne de 2 rondes n'existe pas.
2. **Reconstruction d'une exécution `failed`.** Les requêtes de sélection et de
   mise à jour de la reprise ordinaire excluent l'état `failed`
   (`execution_store.rs:1325` et `:1393`). Étendre `may_resume` ne suffit pas :
   la 160 doit définir et prouver comment relancer une mission dont
   l'exécution est `failed`.
3. **Identité de relance vérifiée et solde de l'attente.** L'identité
   `limit-resume:<tâche>:<ronde>:<reset>` doit être vérifiée avant l'appel, et
   un rejeu après crash doit solder l'attente (pas de seconde relance, pas
   d'attente orpheline).
4. **Borne de pannes de 152 non consommée.** Une continuation due au quota ne
   doit pas consommer la borne de 2 pannes de 152 (`resume_attempts`, 152
   `data-model.md` I4).
5. **Définition de `suspended_total_secs` et pas de double comptage.** Définir
   l'usage de `suspended_total_secs` pour une attente de quota, sans double
   comptage quand une pause et une attente de quota se chevauchent.
6. **Chemin `rejected()`.** Après `rejected()`, le gestionnaire général supprime
   le suivi de la demande et notifie le parent (`daemon.rs:18095`). La décision
   d'attendre doit empêcher cette notification et cette suppression, pas
   seulement la mise à `failed` (`native_delegation.rs:810`). La 160 doit aussi
   traiter le refus de remise émis par le wrapper (`wrapper.rs:6218-6262`).

Ces exigences restent hors implémentation157. Leurs références de lignes sont
historiques dans la spec et devront être revalidées à l’entrée de160. Aucun
champ d’attente ou compteur de reprise supplémentaire n’est ajouté en157.

## Références

- Spec : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/157-catalogue-quotas/spec.md
- Contrat : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/157-catalogue-quotas/contracts/delegation157.md
- Modèle : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/157-catalogue-quotas/data-model.md
- Recherche : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/157-catalogue-quotas/research.md
- Plan : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/157-catalogue-quotas/plan.md
