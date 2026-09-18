# Contre-revue externe bdget — 2026-09-16

Agent :127bccff-8490-453a-8182-884b749ec41e, fil bdget.
Fournisseur déclaré par l'agent : Anthropic, Claude Fable5.1 ; non déduit du seul typeclient.
Demande liée : review-103-104-plans-20260916, envoyée21:55:24CEST, borne5min.
Réponse reçue21:57:51CEST, IDt3-fbec7719-89f6-4f48-9904-fd16ce0503b6.
Lecture seule demandée : dix documents, logique/ACL/pagination/perf/idempotence et minimalisme ;
interdiction écriture, commit, services/tests et dépense. L'agent confirme n'avoir rien lancé.

Verdicts reçus : **APPROVE_WITH_CHANGES** pour103 et104. Pas de verdict d'implémentation.
Les corrections retenues sont intégrées avant Tasks. Pas de seconde approbation externe prétendue.

| Objection | Vérification | Retenue | Traitement |
|---|---|---|---|
|103rendu lié à serde_json peut changer | canonical_send incorpore body ; contratsv1 relus | Oui | Échappement figé + fichiers dorés Unicode/contrôles/listes |
|103authority ambiguë | Champ déclaratif insuffisamment défini dans data-model | Oui pour le problème | Remplacé par source_label clairement indicatif, sans routage ni droits |
|Utiliser daemon_instance101 + nouveau paramètre104 | Instance de daemon n'est pas une identité documentaire durable ;103indépendante101 | Non pour cette solution | Pas de dépendance101ni d'APIde routage ; lecteur choisit la connexion, pas fallback |
|103preuve des7jours | daemon.rs:648 retention_days:7, store.rs:323, purge startup/horaire | Déjà présente | Preuves confirmées, aucune nouvelle promesse |
|104extraitpréfixe peu utile | fold_for_search change les longueurs Unicode : position brute non gratuite | Oui avec correction technique | match_offset reconverti vers corps original par secondparcours ; body_digest pour read ciblé |
|104pire cas16Mio | Plan avait borne<17Mio etSC006, formulation à renforcer | Oui | Au plusun corpshorsbudget, arrêtensuite, testéexplicitement |
|104fil avant102 | Base1738a072 n'a pas les tables102 | Oui | capability_unavailable explicite ; livraisoncomplète bloquée sur102 |
|104querybrute dans fingerprint | Contrat choisit queryoriginale volontairement | Oui documentation | Requête strictementidentique exigée même si repliéquivalent |

Minimalisme et responsabilité future : aucune duplication de transport/base, aucune IA de résumé ;
risques hérités de visibilité/rétention annoncés. Recherche bornée et sans effet sur les lectures102.
Les seuils de performance sont encore à mesurer, pas validés par cette revue.

