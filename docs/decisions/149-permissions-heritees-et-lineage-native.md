# Décision149 — droits hérités et projection native dans Lineage

Date :2026-10-10. Statut : proposé, en attente du gate GLM du plan commun.

## Contexte

La session148 possède déjà le moteur natif, l'identité attestée par session et
les remises corrélées. Elle limite un parent externe à discovery. Elle réserve
development à un profil Codex sans réseau. L'utilisateur demande de vrais enfants
lecture/écriture en un appel, sans grant humain Bridget supplémentaire. Leur
activité doit apparaître dans Lineage T3, avec un journal durable consultable.

## Décision

Bridget reste seul moteur. Le parent de connexion fournit une provenance de
politique effective. Un appel sans posture hérite. Discovery explicite réduit
les droits. L'identité et les permissions ne sont pas des arguments d'agent.
Les révocations et l'opt-out MCP existants restent prioritaires. Aucun nouveau
feature toggle ou grant par mission n'est introduit.

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

T3 étend l'attestation148 à la politique réellement transmise à son fournisseur.
Le daemon valide lui-même la preuve privée hors verrou et recontrôle le binding
vivant au puits. Il ne persiste ni ne transmet aucun credential. Les wrappers
standalone attestent leurs sources fournisseur propriétaires. Un fait manquant
produit un refus nommé; il ne se transforme pas en droits inventés.

Chaque tâche fige la politique effective et la définition enfant à admission.
Le retry et la reprise gardent le même enfant, modèle, mission et politique.
Un changement ultérieur de mode UI n'élargit pas les tâches déjà admises.
Le moteur natif continue sans T3 après admission. L'annulation explicite agit
toujours sur la tâche native et sa descendance.

Le mappage inter-fournisseurs distingue la frontière logique de projet et le
confinement OS réel. Full-access attesté peut autoriser le mode bypass Claude
dans la seule mission enfant. Un sandbox OS workspace-write Codex n'est pas
reproduit par de simples permissions d'outils Claude. Sans capacité équivalente
prouvée, ce mappage est refusé. Aucun framework de sandbox nouveau n'est ajouté.
Les permissions non couvertes restent refusées sans dialogue humain Bridget.

La saga native ajoute parent_task_id et root owner stables. Un snapshot indexé,
borné et cohérent expose les tâches. Une séquence transactionnelle sert uniquement
à l'invalidation. Les journaux existants restent source unique. La garde humaine147
contrôle liste, détail, journal et annulation. Aucun polling des corps n'est nécessaire.

T3 réutilise ses fils, nœuds, sous-agents et relations existants. Il persiste des
descendants de présentation origin bridget_native, sans session ni tour fournisseur
ni effet de lancement. Le marqueur bridgetTaskRef les exclut du réimport Bridget.
Lineage les ouvre; la colonne de gauche les masque; le composeur reste en lecture seule.

## Alternatives écartées

- Utiliser delegated_task.request T3 : cela crée une seconde exécution et une dépendance T3.
- Déduire les droits du mode UI : les overrides et changements de fournisseur peuvent diverger.
- Demander un nouveau grant Bridget : cela contredit l'héritage demandé.
- Appliquer un bypass global ou imposer discovery : cela élargit ou réduit les droits sans preuve.
- Inventer un confinement OS Claude ou un framework sandbox : aucune équivalence n'est prouvée.
- Copier les journaux ou créer un journal métier supplémentaire : les sources natives suffisent.

## Conséquences et validation requise

La provenance est plus précise. Les policies sont figées et les descendants
peuvent fonctionner hors T3. La présentation ne contrôle pas les modèles.
L'attestation standalone Claude PTY doit prouver le mode courant et les inputs
réels revisionnés. La recette positive de premier parent GLM doit fermer ce gate.

GLM détient tests et relectures. Le gate doit couvrir héritage lecture/écriture,
full-access Codex→GLM, permissions refusées sans dialogue, politiques opaques,
standalone, rejeu/reprise, résultat unique, projection atomique, journal après
nettoyage, annulation et absence de boucle de pont. Aucun restart ou état de
production ne fait partie de cette validation.

Contrats canoniques :
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/contracts/permissions.md`.
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/contracts/lineage.md`.
