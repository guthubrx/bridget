[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/137-entetes-discrets/implementation.md#extrait-1).

## Implémentation et preuves
MessagesTimeline.logic.ts: hasBridgetEnvelopeHeading, préfixe1024 caractères,
reconnaissance bornée de trois enveloppes. Ce prédicat n'atteste pas une identité.
MessagesTimeline.tsx: UserMessageBody garde le texte et parseRawHtml=false ;
deux sélecteurs CSS visent uniquement le premier paragraphe direct.
Pas d'état, composant, table, protocole, dépendance ou modification du pont.

Tests ajoutés avant code:12 RED (fonction absente),120 existants PASS.
Après code:200/200 PASS sur MessagesTimeline.logic.test.ts et MessagesTimeline.test.tsx.
Commande depuis apps/web: vp test run src/components/chat/MessagesTimeline.logic.test.ts
src/components/chat/MessagesTimeline.test.tsx. Sortie0,5,02s.
Typecheck TypeScript7.0.2 existant --noEmit: sortie0.
Lint ciblé sur les trois fichiers: sortie0, avertissements préexistants ailleurs
dans la timeline, aucune erreur. Fmt ciblé --check et git diff --check: PASS.
vp build web: sortie0,31,85s ; warning générique de taille de chunks existants.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/137-entetes-discrets/implementation.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/137-entetes-discrets/implementation.md#extrait-3).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/137-entetes-discrets/implementation.md#extrait-4).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/137-entetes-discrets/implementation.md#extrait-5).

## 2026-10-06 — Livraison des sources

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/137-entetes-discrets/implementation.md#extrait-6).
