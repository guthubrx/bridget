# Modèle de présentation141

## Source immuable

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/data-model.md#extrait-1).

## Projection éphémère

BridgetEnvelopePresentation conserve label, kind et readableText. La projection peut exposer une collection facultative de messages directs groupés. Une collection n'existe que si le découpage complet est certain.

Chaque section projetée contient le nom d'affichage reçu et le corps exact. L'indice est l'ordre de réception, pas une priorité. Un identifiant présent dans l'enveloppe peut servir à la validation. Il ne remplace pas un nom humain disponible. Si seul un UUID est fourni, cet UUID reste visible sans nom inventé. L'aperçu reproduit au plus les 120 premiers caractères du corps ; il n'est ni stocké ni résumé.

## Validation

Le total est positif et cohérent avec les sections. Les indices couvrent exactement 1 à total dans l'ordre. Chaque en-tête de section est complet. Les expéditeurs sont non vides. Un séparateur supplémentaire ou ambigu empêche la collection projetée. Le corps entier reste alors dans readableText.

La projection n'altère jamais le message source. Un suffixe inconnu reste dans readableText. LF et CRLF sont pris en compte sans modifier les octets copiés.

## État local

Le groupe passe de fermé à ouvert et inversement. Sa première ouverture ouvre la première section seulement. Chaque section possède ensuite son propre état ouvert, conservé quand le même groupe est refermé. La clé existante fil/message isole cet état. Il ne devient ni une préférence persistante ni un état global. Chaque transition de hauteur informe l'ancrage existant.

## Invariants

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/data-model.md#extrait-2).
