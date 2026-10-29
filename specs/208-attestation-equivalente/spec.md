# 208 — Réattestation sans faux changement de droits

Deux admissions ont échoué avant Release avec le seul champ revision différent. Les droits doivent être vérifiés à nouveau ; un renouvellement technique de preuve provider_turn ne constitue pas, seul, un changement des permissions.

Réutiliser l'équivalence déjà appliquée aux continuations : source, version, identité fournisseur, pilote, dossier, modes et politique complète restent identiques. Seuls run_id, provider_session_id et revision peuvent se renouveler pour provider_turn, après la réattestation native du même fil et du même propriétaire. native_wrapper reste strict. La preuve fraîche reste obligatoire. Aucun affaiblissement des révocations, de l'identité de connexion, du routage, des sources, du contrôle pause/gel et du permis worker.

Le snapshot admis, son digest et sa révision de préparation worker restent inchangés ; ne pas réécrire de tâche ou de reçu réel pour cette correction. Les tâches cancelled ne sont jamais relancées. Préserver28414216 et son enveloppe, sans substitution.

Deux api_error Opus/Cartae sont établis dans leurs journaux : CLI2.1.251 incompatible avec le modèle,2.1.280 minimum demandé par le fournisseur. Aucun fait de quota, compte ou durée fautive. Le statut doit guider la vérification corrélée d'un api_error sans déduire une cause et fournir une référence native au responsable. La mise à jour de Claude reste une opération distincte, non exécutée ici.
