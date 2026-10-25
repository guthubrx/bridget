[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/137-entetes-discrets/plan.md#extrait-1).

## Réutilisation et implémentation
UserMessageBody utilise ChatMarkdown.className et rend des paragraphes directs.
Étendre MessagesTimeline.logic.ts avec un prédicat borné au préfixe1024 caractères.
Enveloppe complète en première ligne et ligne vide obligatoire.
Dans UserMessageBody, classes du premier paragraphe seulement: secondary-label/xs.
Texte transmis intégralement, HTML brut toujours désactivé. Aucun composant,
état, dépendance ou plugin nouveau. Complexité constante et bornée.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/137-entetes-discrets/plan.md#extrait-2).
