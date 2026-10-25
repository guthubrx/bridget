# Archives techniques de l’adaptateur T3

Document d’origine : [specs/089-communication-core/research.md](../../../../../../specs/089-communication-core/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

1. **Tout convertir en A2A :** ajoute une nouvelle surface réseau, une correspondance d'états et de nouvelles questions d'autorisation sans résoudre le découplage local. Une passerelle pourra être étudiée pour un consommateur A2A réel, après le noyau.
2. **Installer LangChain/LangGraph comme bus :** ce sont des abstractions applicatives différentes ; notre dette est dans les dépendances et coutures de communication existantes. Les idées de reprise sont utiles, la dépendance n'est pas justifiée.
3. **Réécrire le daemon :** perd les preuves accumulées sur les courses ACK/ledger/replies/crash. Extraction graduelle préférable, avec réduction mesurée.
4. **Revenir à tmux pour simplifier :** contraire au besoin confirmé ; les pilotes natifs restent. L'ancien comportement peut servir de comparaison historique, jamais de dépendance requise.
5. **Forker T3 maintenant :** une UI ne prouve pas la stabilité du cœur. La preuve T3 devra ultérieurement lire un état unique, sans lancer une seconde flotte ni posséder une deuxième base de missions.
6. **Supprimer toutes les protections :** confond sandbox de rendu web et contrôle d'accès aux messages/processus. On enlève le premier du produit extrait, on maintient le second.
