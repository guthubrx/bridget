# Contrat CLI : renommage d’agent

## Commande

```text
bridget rename <nouveau-nom>
```

## Préconditions

- La commande est exécutée depuis un agent lancé via Bridget et dispose de son fichier d’état local.
- Le wrapper de l’agent est encore connecté au démon Bridget.

## Résultat

- Succès : la commande affiche le nouveau nom confirmé et se termine avec le code `0`.
- Échec : la commande écrit une cause exploitable sur la sortie d’erreur, conserve le nom courant et se termine avec un code non nul.

## Garanties

- La commande ne déconnecte pas l’agent.
- Le nouveau nom devient l’adresse de routage immédiatement après le succès.
- La prochaine reprise de cet agent utilise le nom confirmé.
- Les commandes d’envoi et de réponse exécutées après le succès utilisent la nouvelle identité.
