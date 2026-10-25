# ADR-165 — Cycle de vie des liens adaptateur de bureau

**Date :** 2026-10-14. **Statut :** accepté après contre-revue du plan R5.

## Contexte

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/165-cycle-vie-liens-t3.md#extrait-1).

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/165-cycle-vie-liens-t3.md#extrait-2).

## Décision

Le pont conserve toujours les racines, les forks et les fils V1. Il conserve un sous-agent pendant un tour actif ou non terminal. Un sous-agent terminal sans obligation durable reste déconnecté.

Un sous-agent qui vient de terminer garde son lien pendant un drain. Le worker traite le dernier snapshot. Il publie une clé drainée seulement si sa file, son journal, ses corrélations et ses réponses sont vides. Une remise invalide cette clé dès sa lecture réseau. Un dispatch accepté ou incertain interdit aussi le drain jusqu'à un nouveau snapshot. L'horloge `drain_since` ne repart pas sur un changement terminal. `turn_wait` reste la borne dure.

Le premier snapshot réussi relit les états durables des seuls sous-agents terminaux. Un `pending` ou un tour connu non projeté crée une obligation. Le pont rouvre au plus 32 liens de drain simultanément. Une tentative est consommée seulement après un drain attesté ou après `turn_wait`. Une mort imprévue avant cette limite autorise une reprise. Une nouvelle activité autorise une future tentative.

Le propriétaire `Link` garde le worker, le lecteur et un clone de fermeture du flux. La fermeture désinscrit le lien, attend le worker, ferme le flux de secours, puis attend le lecteur. Une panique du worker ne laisse donc pas le lecteur ni son descripteur détachés.

## Invariants

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/165-cycle-vie-liens-t3.md#extrait-3).

## Alternatives écartées

Augmenter encore la limite de fichiers conserve la fuite logique. Fermer tous les sous-agents dès leur état terminal peut perdre le dernier résultat ou son accusé. Déduire la filiation du titre est instable. Balayer les fichiers d'état est impossible car leurs noms compacts ne sont pas réversibles. Un nettoyeur périodique créerait une seconde autorité de cycle de vie.

## Conséquences

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/165-cycle-vie-liens-t3.md#extrait-4).

La compatibilité V1 garde le comportement ancien. Le retour arrière consiste à restaurer le binaire précédent. Il ne demande aucune migration de données.

Un sous-agent terminal fermé retrouve son identité Bridget au prochain snapshot quand il redémarre. Le délai nominal est de trois secondes. Il peut atteindre environ six secondes si le redémarrage arrive juste après une lecture.

Pendant cet arrêt, un message Bridget direct vers ce sous-agent est refusé. Le
message ne provoque pas sa reprise. La même règle s'applique à un sous-agent qui
n'a encore démarré aucun tour.

## Validation

Les tests couvrent le contrat V1/V2, la matrice racine/fork/sous-agent, 1 000 enfants terminaux, le drain, cinq lectures silencieuses, la réactivation, les obligations persistées, le plafond 32/100, la course de remise, le dispatch pendant fermeture et la panique du worker. La recette réelle mesure les liens et les descripteurs avant et après activation avec la limite normale de 4 096.
