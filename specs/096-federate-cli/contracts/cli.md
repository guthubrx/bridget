# Contrat CLI096

```
bridget federate ssh://cartae.app -p 2222
bridget federate status
bridget federate remove ssh://cartae.app -p 2222
bridget federate --help
```

Port22 par défaut ; portURL possible si non contradictoire avec -p. --label sélectionne explicitement une installation. Options de liaison095 restent disponibles et documentées. Une liaison existante compatible est réutilisée, jamais réinstallée pour obtenir un succès.

Si une destination DNS correspond à un IP enregistré, seule une correspondance complète et non ambiguë (port, utilisateur lorsqu'il est fourni) est admise. La configuration existante est validée avec les fonctions095 avant usage. L'URL ne remplace pas la cible enregistrée. Une erreur ou plusieurs candidats donne un refus actionnable.

Pour `remove`, une équivalence DNS seule exige une confirmation double-TTY affichant le label, l'hôte enregistré et le port ; hors TTY, demander `--label` explicite. Un label seul ou l'hôte exact enregistré est une sélection explicite. Si URL et label sont fournis, ils doivent concorder, jamais ignorer une URL divergente.

Bornes fermées : 128 installations, 64 adresses DNS, résolution5 secondes. Dépassement = refus global, pas liste tronquée qui masquerait une ambiguïté. Aucune résolution pour `status` ni pour une correspondance d'hôte exacte.

Statut sans destination : inventaire borné des installations, aucun SSH. Retrait sans destination/label interdit. Les codes non nuls natifs restent non nuls. Ni SSH-config exécutable arbitraire, ni eval d'argument, ni commande shell construite avec des valeurs utilisateur.

Le script095 autonome doit conserver ses modes run/install/status/remove/service-run. Le nouveau point d'entrée d'adaptation ne contourne pas ses gardes et n'est pas un outil MCP.

La façade URL/configuration/inventaire reste intégralement dans ce script et appelle directement les validations095. Rust ne parse aucun reçu ni configuration et ne crée aucun protocole de projection ; il matérialise la source embarquée et transmet les arguments/code retour.
