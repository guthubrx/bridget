# Réponse finale157 aux revues

R2 APPROVE_WITH_CHANGES sans objection bloquante. Sources R4 inchangées ; Converge2 et audit final restent applicables. Le reçu de livraison doit rappeler les limites ci-dessous.

La réponse est additive : substitution:null est toujours présente sans remplacement. Elle n'est donc pas byte-identique à150. FR019 et son test protègent les seuls octets de la demande canonique sans fallback.

Quota : le dernier fait reçu de même agent_type/fenêtre gagne, toutes sources confondues, conformément à FR007. FR011 retire cette clé si le nouveau fait est passé. Une observation de status-line peut donc remplacer un rejet stream-json ; les sources n'ont pas de priorité. Le catalogue reste une indication observée, sans garantie d'authentification ni de disponibilité réelle du fournisseur. Aucun nouveau mécanisme de priorité n'est introduit.

La faiblesse alléguée de l'oracle est réfutée : après retrait de five_hour, seule unknown reste ; quota(...).expect("autre fenêtre conservée") échoue si elle est effacée. L'assertion !exhausted suit cet expect. Lecture provider_limits.rs166–199 et GREEN13 le prouvent ; aucun test supplémentaire.
