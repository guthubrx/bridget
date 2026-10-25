# Archives techniques de l’adaptateur T3

Document d’origine : [specs/139-entetes-complets/plan.md](../../../../../../specs/139-entetes-complets/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

# Plan139
Code : /Users/moi/11.Repositories/t3code-local/.worktrees/139-entetes-bridget.
Base : local/v0.0.45, 3d30a4836b. React19, TypeScript, Tailwind4, Vite Plus.

<a id="extrait-2"></a>

## Extrait 2

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
