# Tableaux dans la CLI Bridget

Utilisez bridget --table specialists list ou bridget specialists list --table.
Même option pour agents, task list, control status, control delegations,
requests, inbox list, queue, ledger, journal, events, wake list, accounts list,
updates status, delegate --catalogue, clients check, formats policy,
mobile devices, federate status et les autres réponses finies de la CLI.

Les colonnes suivent la famille. Les spécialistes montrent le nom humain, le
rôle, les projets, l'hôte et l'état observé. Un nom humain n'est pas un identifiant
d'envoi. Le JSON existant garde tous les détails et références.

Sans --table, les sorties restent inchangées. --json garde son format.
Combiner --json et --table est refusé avant l'opération. Les valeurs d'option
restent des valeurs, même lorsqu'elles valent --table. Arguments fournisseur
et ceux après -- non interceptés.

Le terminal fixe la largeur ; COLUMNS sert de repli, puis120 colonnes.
Les cellules longues se replient. NO_COLOR=1 retire la couleur, également
absente dans les pipes et avec TERM=dumb. Les états restent lisibles.

Aucune requête, connexion ou vérification d'accès supplémentaire. Une auth non
vérifiée reste non vérifiée. Installé ne signifie pas chargé. Confirmations,
erreurs et codes de retour restent propres à la commande. Les flux continus,
protocoles MCP, appairages/QR et contenus bruts refusent --table avant démarrage.

Pour send/reply, placer --table avant le premier mot du corps, ou avant la commande. Un texte après -- reste intact. who/discover gardent leurs gardes, pieds de contrôle et avertissements natifs.
