# Frontière d'événements de coordination — entrée 015 gelée

Ce document ne définit encore aucun DTO 016. Il fixe la frontière à laquelle
T1603 devra ajouter une extension versionnée minimale.

## Corpus disponible

Le corpus de base est la copie octet pour octet de la négociation de service
015 v1 : `fixtures/service-negotiation-v1.jsonl`. Il établit le rôle
`service`, la version `1` et la capacité `maicie_guichet` avant toute relève
ou réception d'événement.

Les seuls états de cycle de vie actuellement publiés par 015 sont :

- `answered` ;
- `cancelled` ;
- `timed_out`.

Ils restent des faits Bridget, persistés avec leur `event_id`, leur
`request_id`, leur instant `observed_at` et leur corrélation de réponse. Une
observation incomplète ne devient jamais un fait métier Maicie.

## Extension interdite avant T1603

`reminder_sent` n'existe pas dans 015. Avant T1603, aucun client ne peut le
fabriquer à partir d'un texte, du ledger, d'une échéance locale ou d'une
absence de message. T1603 devra publier sa version, son canon, ses refus, sa
corrélation demande/message/destinataire, sa génération et ses fixtures avant
que tout consommateur 016 le lise.

## Règle de couture

Le consommateur Maicie relira les bytes produits par Bridget ; il ne partage
ni le store Bridget ni un DTO local supposé équivalent. Tout curseur, toute
fraîcheur et toute déduplication devront être exposés par le protocole public
et conserver la reprise des mêmes bytes et du même `event_id`.
