# Revue indépendante — delta compte/contexte

Tâche6c0833f9-d175-4fcc-b0fe-a6e8a41a6950, clé
bridget221-revue-l2a-compte-contexte-20261031-01.
Autre fournisseur Claude, axes libres, Haiku/SYS choisi par Bridget.
Base1463bcf5, patch complet fourni en texte, lecture seule sans shell/tests.

Verdict APPROVE du delta. Deux P2 de couverture retenus :
1. compte non identifiable : assertions reported_models/not_verified/nonce nul ;
2. expiration de réception : assertions auth non vérifiée et nonce nul.
Ces ajouts passent dans la suite L2a R4. Ils étendent des tests existants.

Réserve non retenue après lecture : le relecteur indique que valid() accepte
un nonce pour un état autre qu'Observed. ProviderCatalogue::valid, dans
crates/bridget-transport/src/provider_catalogue.rs, impose déjà dans la branche
des autres états models.is_empty() ET account_scope.is_none(). Ingest appelle
update.valid avant d'insérer. Aucun normaliseur redondant ajouté.

La première fixture de réponse ancienne Observed utilisait une liste vide,
donc InvalidEnvelope. Échec R2 conservé. R3 utilise une enveloppe valide,
assert!(late.valid()), ancien modèle et séquence1, puis prouve l'absence
d'effet. R4 ajoute les deux contrôles demandés. Le commentaire sur le test220
est exact : son ancienne réponse est ignorée par refresh_id ; la preuve du
garde de séquence vient de spec221_l2a_same_model... seulement.

Résultat entier lu, accusé et annulation propriétaire demandés une seule fois.
Le statut conserve result_available et le worktree propre ; aucun retrait
sans vérification complète des usages. Envoi complémentaire in_flight :
ne prouve pas lecture du complément par le relecteur.
Résultat privé : /Users/moi/.cache/bridget221-l2a/review-result.json.
Aucune recette humaine de changement de compte, aucun runtime ni égalité
globale attestés par cette revue. C01/T036 reste ouverte.
