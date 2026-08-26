# Session 035 — Garde de mandat émis

## Propriété

Une délégation dont aucune remise du mandat n’est attestée ne doit pas
apparaître comme engagement en cours dans la ronde. La projection doit
conserver cette délégation visible comme `prepared` (en attente d’émission),
puis inclure le participant seulement pour un état explicitement attestant une
remise : `accepted`, `outcome_unknown` ou `rejected`. Tout état non listé reste
hors de l’en-cours jusqu’à ce que son sens soit instruit.

## Source et rattachement

La source existante est `delegation_outbox.state` ; aucun champ ni état nouveau
n’est ajouté. Cette garde rend opposable l’arête R1 du graphe de contrôle décrit
dans `specs/034-orchestration-graphe-de-controle/decision.md` (mandat attesté),
sans bloquer la création d’une délégation et sans appeler de modèle.

## Vérification

Le témoin `mandat_non_emis_ne_parait_pas_en_cours` place une délégation
`prepared` et exige que son participant reste non affecté. Le contrôle positif
`remise_attestee_fait_progresser_la_delegation` place une outbox `accepted` et
exige que son participant soit affecté. Un état futur `dispatching` est aussi
présent et doit rester hors de l’en-cours. Retirer la liste positive tue le
témoin `mandat_non_emis_ne_parait_pas_en_cours`; l’ajouter à tort tue
`etat_non_attestant_ne_parait_pas_en_cours`.
