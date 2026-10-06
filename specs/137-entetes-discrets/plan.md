# Plan137
Code: /Users/moi/11.Repositories/t3code-local/.worktrees/137-entetes-bridget.
React19, Tailwind4, Vite Plus, TypeScript ; base58b212bccb local/v0.0.45.
Artefacts dans Bridget, conformément à l'interdiction de plans committés dans T3.

## Réutilisation et implémentation
UserMessageBody utilise ChatMarkdown.className et rend des paragraphes directs.
Étendre MessagesTimeline.logic.ts avec un prédicat borné au préfixe1024 caractères.
Enveloppe complète en première ligne et ligne vide obligatoire.
Dans UserMessageBody, classes du premier paragraphe seulement: secondary-label/xs.
Texte transmis intégralement, HTML brut toujours désactivé. Aucun composant,
état, dépendance ou plugin nouveau. Complexité constante et bornée.

## Gates et vérification
Worktrees isolés, aucun changement utilisateur écrasé. Couleurs de thème et
copie existantes réutilisées. Scripts/templates SpecKit locaux absents: protocole
des skills appliqué manuellement. Tests avant code puis timeline, lint ciblé,
typecheck/build web. Aperçu autorisé, base production jamais ouverte en écriture.
Le script t3-local-build réinitialise son worktree et refuse l'installation si
l'application tourne: ne pas l'exécuter pendant cette retouche.
