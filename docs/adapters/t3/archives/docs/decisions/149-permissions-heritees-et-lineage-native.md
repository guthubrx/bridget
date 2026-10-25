# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/149-permissions-heritees-et-lineage-native.md](../../../../../decisions/149-permissions-heritees-et-lineage-native.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

La session148 possède déjà le moteur natif, l'identité attestée par session et
les remises corrélées. Elle limite un parent externe à discovery. Elle réserve
development à un profil Codex sans réseau. L'utilisateur demande de vrais enfants
lecture/écriture en un appel, sans grant humain Bridget supplémentaire. Leur
activité doit apparaître dans Lineage T3, avec un journal durable consultable.

<a id="extrait-2"></a>

## Extrait 2

La voie Claude de même famille peut hériter des inputs effectivement sélectionnés
sans prétendre observer une fusion de settings. Launcher, profil, cwd, sources,
absences et revisions sont attestés puis figés. Le CLI effectivement résolu
et le PATH qui le sélectionne font partie des inputs vérifiés. Le launcher et
le binaire ont chacun leur chemin et leur digest.
Une chaîne opaque rend permission_source_unavailable pour son seul contexte.
Leur changement avant lancement ou reprise rend un refus nommé. Le cas réel
gclaude/profil GLM commun à T3 et
au registre natif possède une recette positive prévue. Les sources managed
opaques sans voie vérifiable restent des refus limités à leur contexte.

<a id="extrait-3"></a>

## Extrait 3

T3 étend l'attestation148 à la politique réellement transmise à son fournisseur.
Le daemon valide lui-même la preuve privée hors verrou et recontrôle le binding
vivant au puits. Il ne persiste ni ne transmet aucun credential. Les wrappers
standalone attestent leurs sources fournisseur propriétaires. Un fait manquant
produit un refus nommé; il ne se transforme pas en droits inventés.

<a id="extrait-4"></a>

## Extrait 4

T3 réutilise ses fils, nœuds, sous-agents et relations existants. Il persiste des
descendants de présentation origin bridget_native, sans session ni tour fournisseur
ni effet de lancement. Le marqueur bridgetTaskRef les exclut du réimport Bridget.
Lineage les ouvre; la colonne de gauche les masque; le composeur reste en lecture seule.

<a id="extrait-5"></a>

## Extrait 5

- Utiliser delegated_task.request T3 : cela crée une seconde exécution et une dépendance T3.
- Déduire les droits du mode UI : les overrides et changements de fournisseur peuvent diverger.
- Demander un nouveau grant Bridget : cela contredit l'héritage demandé.
- Appliquer un bypass global ou imposer discovery : cela élargit ou réduit les droits sans preuve.
- Inventer un confinement OS Claude ou un framework sandbox : aucune équivalence n'est prouvée.
- Copier les journaux ou créer un journal métier supplémentaire : les sources natives suffisent.
