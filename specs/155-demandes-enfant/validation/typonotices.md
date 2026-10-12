# Notices155 — deux warnings conservés

Ce checkpoint ne constitue pas un résultat de lint. Les logs ciblés signalent
les symboles suivants comme inutilisés dans la cible de production :

| Symbole | Justification de maintien |
|---|---|
| due_child_requests | API obligatoire de T005, indexée et testée pour les demandes échues ; le tick doit employer child_request_followups pour sent/void et les clôtures |
| format_cited_paths | Helper pur prévu par T009, testé, 64 chemins/4096 octets UTF-8 ; sa phrase d’affichage ne remplace pas le tableau JSON 1024 octets du core |

La présentation core est lexicale, bornée et marquée paths_verified=false.
La différence de forme et de budget justifie la duplication légère ; aucun
raccord expansif n’est requis. Les deux warnings restent non bloquants ;
aucune suppression de code, nouveau helper ou nouveau test dans ce passage.
La revue complète T038 et la suite T039 ne sont pas déclarées terminées.
