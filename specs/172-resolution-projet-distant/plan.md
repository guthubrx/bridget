# Plan 172

1. Résoudre une seule fois par machine la correspondance configurée.
2. Conserver son identifiant dans les observations durables de routage.
3. Réutiliser cet identifiant pour la délégation et contrôler sa stabilité.
4. Tester le passage jusqu'à la sonde distante, le reçu, les gardes et les régressions ciblées.
5. Compiler le maître macOS depuis la source validée sur Cartae, activer et
   faire rejouer par le parent opus2D sa demande réelle vers Cartae.

Référence T3 lue : apps/server/src/workspace/WorkspacePaths.ts dans
/Users/moi/11.Repositories/t3code. T3 résout et contrôle les chemins dans
une racine de projet. Il ne fournit pas l'équivalent de la correspondance
Mac/Cartae : Bridget conserve son registre de destinations et les gardes worker.

Complexité : O(M) mappings par machine, avec cache par sonde ; aucune sonde
supplémentaire, aucun appel modèle pour découvrir une configuration.
Compatibilité : champ optionnel du snapshot, anciens reçus lisibles.
Pas de migration SQL. Le binaire worker ne nécessite pas cette correction du maître.
