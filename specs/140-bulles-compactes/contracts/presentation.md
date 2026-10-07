# Contrat140 — Enveloppe conservée et présentation compacte

## Remise et compatibilité

BridgetMessage accepte le champ racine thread_display_title facultatif. Les lecteurs historiques ignorent le nouveau champ ; son absence conserve la forme historique. ThreadNotice reste strictement inchangé.

Le daemon résout le titre pour le destinataire au moment de defer_idempotent_delivery et de Deliver classique, avec Store.thread_show. Aucun titre n'est enrichi dans project_thread_wake ou écrit dans le journal/canon. Le nom d'affichage n'accorde jamais de droits.

## Ligne de titre dans l'enveloppe

L'en-tête historique de sollicitation conserve exactement sa syntaxe et ses identifiants. Si le titre est disponible, le pont ajoute après cet en-tête et avant le corps :

```text
Titre du fil Bridget : "Organisation politique"
```

La valeur après le séparateur est une chaîne JSON, pas une interpolation brute. Le pont normalise contrôles et espaces, puis limite à200 caractères. Cette ligne est facultative. T3 ne la consomme que dans la position attendue et avec une valeur JSON valide ; une ligne invalide reste du texte, sans titre prétendu. Le brut conserve la ligne même après projection.

## Libellés visibles

| Famille | Information fiable | Libellé |
|---|---|---|
| Direct | Nom présent et reply explicite | Message / Réponse attendue |
| Sollicitation | Titre transmis facultatif | Titre réel ou Fil partagé · Nouveautés |
| Observation | Famille de notification | Notification |
| Lot direct | Nombre de messages de l'en-tête | N messages |
| Lot observation | Nombre de notifications de l'en-tête | N notifications |

Un nom absent ne devient pas un nom humain inventé. La bulle droite indique qu'il s'agit d'une entrée reçue par l'agent. Aucun type action/blocker/decision/history n'est inféré de son contenu.

## Interaction et conservation

La ligne compacte, logo et chevron compris, forme une commande accessible. Clic, Entrée et Espace ouvrent/replient le contenu. L'état ouvert est exposé. Le focus reste visible. Le contenu lisible emploie le renderer existant. Les détails techniques montrent le texte source complet, sans interprétation HTML. La copie et les actions existantes gardent leur source originale.

Reconnaissance ancrée avec en-tête borné à1024 caractères. La ligne de titre ne supprime jamais du texte inconnu. Les tests vérifieront sa borne et la normalisation à la source. Une enveloppe incomplète suit le rendu ordinaire. Aucun résultat de parsing ne devient une attestation de provenance.

## Interdits

Pas d'ACK réseau sur un clic d'affichage, suppression de consignes fournisseur, backfill, lookup navigateur, migration SQL, nouveau destinataire, altération de mission, arrêt d'Agent Loop ou restart de production.
