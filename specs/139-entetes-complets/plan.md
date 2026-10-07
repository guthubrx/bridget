# Plan139
Code : /Users/moi/11.Repositories/t3code-local/.worktrees/139-entetes-bridget.
Base : local/v0.0.45, 3d30a4836b. React19, TypeScript, Tailwind4, Vite Plus.

## Conception et réutilisation
Étendre hasBridgetEnvelopeHeading, sans changer UserMessageBody ni son texte.
Réutiliser le style du premier paragraphe, secondary-label et text-xs.
Ajouter les sollicitations de fil et leurs lots, après inventaire du pont.
Reconnaissance ancrée, bornée à1024 caractères : O(1), aucune dépendance.
Pas de modèle persistant, d'API, de migration ni d'effet fournisseur.

## Gates et livraison
Deux worktrees isolés ; scripts/templates locaux absents, protocole manuel.
Tests RED avant correction, GREEN ensuite ; lint/types/build ciblés web.
Aperçu avec composant réel et données synthétiques, jamais la base active.
Construire par build-desktop-artifact.ts, pas t3-local-build qui réinitialise.
Conserver les valeurs publiques de relais du paquet local existant.
Identifier le commit source et le SHA256 du paquet, sans fausse installation.
Après autorisation explicite : commit/fusion/push, manifeste durable, sauvegarde,
job de livraison indépendant avec contrôle santé et retour arrière du paquet.
Article XIX/XX : une extension ciblée, aucun composant/service/stockage nouveau.
