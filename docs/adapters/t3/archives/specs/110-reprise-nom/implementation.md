# Archives techniques de l’adaptateur T3

Document d’origine : [specs/110-reprise-nom/implementation.md](../../../../../../specs/110-reprise-nom/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Journal du pont T3, toutes les 330 secondes depuis au moins 12:11 :
`nom du fil import:claudeAgent:da47eac1… refusé : NameConflict ; nouvel essai dans 330s`.
Trois fils concernés. Le nom `claude-horizon` était détenu par le profil `8204e45e`, mis à jour le
14 septembre et absent de l'annuaire ; `wiki` par `f82380fc` du 16 septembre. Le fil vivant portait
son titre automatique, « Animations Mixamo et compositions de mouvements ».
Mesure : 268 profils pour 16 agents connectés, 253 dormants depuis plus de deux jours.

<a id="extrait-2"></a>

## Extrait 2

- Binaire reconstruit par le script d'entretien, cible `target/release/bridget`, soit le fichier
  pointé par le lien de la ligne de commande et par les deux services launchd.
- `com.bridget.daemon` puis `com.bridget.t3` relancés par `launchctl kickstart -k`.
- Le pont a republié les titres à la reconnexion. Journal : aucun `NameConflict` après la relance,
  contre trois toutes les 330 secondes avant.
- Annuaire vérifié : `claude-horizon`, `wiki`, `horizon-3D`, `opus-horizon`, `horizon-cursor` et
  `cursor-listen` portent désormais le nom affiché dans T3. Quinze agents reconnectés.
- SC-004 satisfait.
