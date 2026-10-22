# Sécurité et confidentialité

[English](SECURITY.md) · [Français](SECURITY.fr.md) · [Audit](docs/security-audit.fr.md)

## Signaler en privé

Ne publiez pas de clé, conversation privée, base active ou détail permettant
d’accéder à un compte dans une issue publique.

Utilisez le signalement privé GitHub si l’action **Security → Report a
vulnerability** existe pour ce dépôt. Sinon, demandez au mainteneur un canal
privé sans joindre les données sensibles. Cette page ne prétend pas que cette
option GitHub est activée.

Donnez les versions, l’impact, le fichier concerné et une reproduction avec
des données fictives. N’envoyez pas une vraie clé pour prouver sa validité.

## Règles du dépôt

Conservez hors Git les clés fournisseur/SSH, cookies, URL authentifiées, profils
de compte, bases actives et exports bruts de conversations. Utilisez des
domaines et IP réservés à la documentation. Un nom générique de compte n’est
pas un secret.

Le `.gitignore` protège les fichiers non suivis. Il ne retire ni les fichiers
déjà commités, ni leur historique, ni les tags, actifs de release, forks ou clones.

`scripts/check-secrets.sh` lance Gitleaks sur l’historique récupéré, avec
sortie masquée. Les empreintes immuables de `.gitleaksignore` correspondent
aux faux positifs historiques examinés. Ne masquez pas un dossier entier,
tous les tests, une règle complète ou une alerte nouvelle inexpliquée.

## Si une vraie clé est trouvée

1. Révoquer ou renouveler la clé chez son émetteur. Ne pas essayer la clé suspecte.
2. La retirer des fichiers courants et mettre une valeur fictive.
3. Recenser en privé les références Git, actifs de release et autres copies.
4. Organiser séparément une éventuelle réécriture de l’historique : les SHA
   changent et les autres copies doivent être resynchronisées.
5. Refaire le scan et garder un compte rendu masqué.

Effacer une ligne ou rendre le dépôt privé ne révoque pas une clé. Un scan
sans alerte ne garantit pas l’absence de toute donnée sensible.

Bridget contrôle les clients coopératifs. Il n’isole pas un processus hostile
qui contrôle le même compte système et peut lire ses fichiers privés.
