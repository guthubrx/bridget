# Livraison — 24 octobre 2026

L'installateur est intégré sur main et poussé. Source du script livré :
`27f7833d4cee9814094c1b487b79d20569211056`.

Commandes publiques disponibles :

```sh
curl -fsSL https://cartae.app/bridget/install.sh | sh
curl -fsSL https://cartae.app/bridget/install.sh | sh -s -- --source
```

Le site redirige vers la copie GitHub au commit livré. Cette référence fige le
bootstrap audité, pour éviter de servir une ancienne réponse CDN de main pendant
les quelques minutes après un push. Elle ne fige pas la version à installer :
le défaut résout la dernière release et --source récupère toujours main.
La redirection est à mettre à jour lors d'un changement de l'installateur.
Le script peut aussi se télécharger directement depuis main sur GitHub.

Vérifications :
- script téléchargé via cartae.app identique octet par octet au script livré ;
- commandes publiques curl|sh : release Mac et Linux, succès réel ;
- commande publique --source sur Linux : commit 27f7833d construit et installé,
  code 0, 1 min 40 s, puis 0,41 Gio d'intermédiaires purgés par le wrapper ;
- 16 tests ciblés, liens documentaires et syntaxe shell conformes ;
- CI GitHub du commit 27f7833d : success (secrets, docs, contrat installateur) ;
- Caddy validé avant rechargement, ancienne page /bridget toujours HTTP 200.

Revue initiale BLOCKED traitée, contre-revue APPROVE_WITH_CHANGES sans blocage,
remarques vérifiées/corrigées dans review.md. Résultats lus/ACK ; clôture des deux
continuations demandée, sans en déduire l'absence de processus.

Les HOME jetables, leurs binaires et checkouts sources ont été retirés après
conservation des reçus et journaux privés. Aucun compte utilisateur réel, daemon,
tunnel ou processus T3 remplacé. Branche de travail locale fusionnée puis retirée.
Worktrees de revue distants conservés : leur statut ne prouve pas à lui seul
l'absence de processus ni de fichiers ignorés. Aucun retrait forcé.

SYS-LE-2 et KS-STOR ne sont pas installés par cette livraison. Le DMG, la
publication d'une nouvelle release binaire et le rattachement de ces serveurs
à un maître restent exclus du périmètre exécuté.
