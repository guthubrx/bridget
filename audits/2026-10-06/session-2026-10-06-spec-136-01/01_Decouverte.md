# Découverte SPEC136
178 fichiers source repérés (163 Rust, 15 Python) ; contexte global LIGHT.
Profondeur : six fichiers modifiés, uniquement leurs changements136 et les
invariants directement appelés. Aucune prétention à 100% de178 fichiers.

## Hot Paths et Threat Model
Le manifest décrit quatre chemins. L'identité vient de la connexion active
(daemon.rs:10817–10841), jamais d'un actor fourni par le demandeur.
Le dépôt revalide membre/auteur/audience avant ACK ou INSERT.
L'attaquant pertinent est un agent local qui tente de masquer le travail d'un
autre ou un client non enregistré. La socket Unix n'est pas un endpoint Internet.
Assets : corps/provenance, curseurs/reçus, identités, opérations de rejeu.
Aucune commande fournisseur, exécution de mission, dépendance ou réseau ajouté.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/audits/2026-10-06/session-2026-10-06-spec-136-01/01_Decouverte.md#extrait-1).

## Passe préparatoire interrompue
La première suite s'est arrêtée sur spec102_v34 : MAX(version) attendu1,
obtenu2. Mise à jour de l'attente de migration, sans retirer les assertions de
corps, demandes, clés étrangères, contraintes ou version d'idempotence6.
La recette vérifie maintenant dix remplacements, et le texte libre
« annule tout » ne remplace pas implicitement d'autres corps. Seuls tests et
Gherkin changés après la revue adverse ; code de production inchangé.
Nouvelle passe audit sur cet état figé ; suite complète relancée.
