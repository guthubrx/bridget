# Recherche et décisions: Registre et identité des projets

## Décision 1 - Créer une identité explicite, ne pas promouvoir `domain`

**Décision**: `project_id` est une identité opaque durable. Le domaine Bridget
reste un libellé de classement dérivé ou surchargé.

**Rationale**: `derive_domain()` utilise le dernier composant de la racine Git.
Il peut collisionner et changer lors d'un déplacement. Le README précise déjà
que le domaine ne cloisonne pas.

**Alternative considérée**: rendre le domaine unique et sécuritaire.

**Rejet**: cela changerait rétroactivement le sens d'un contrat public et
laisserait l'identité dépendre d'un nom de dossier.

**Impact mainteneur**: deux concepts restent visibles, mais chacun a une
responsabilité simple et testable.

## Décision 2 - Maicie possède l'identité, Bridget possède la liaison

**Décision**: la séparation suit la frontière déjà retenue dans SPEC-064.
Maicie garde la vérité de mission; Bridget garde la vérité d'hôte et
d'exécution.

**Alternative considérée**: un fichier JSON commun lu et écrit par les deux.

**Rejet**: un fichier partagé créerait une autorité concurrente, des courses et
un couplage de cycle de vie.

**Impact mainteneur**: chaque panne partielle est représentée par une saga et
non masquée par une écriture supposée atomique entre deux bases.

## Décision 3 - Une commande, une saga idempotente

**Décision**: l'utilisateur agit une seule fois depuis la surface locale de
Maicie. Une outbox durable demande ensuite à Bridget de lier la racine.

**Alternative considérée**: deux commandes manuelles, une dans chaque outil.

**Rejet**: le risque de divergence est transféré à l'utilisateur et la reprise
après panne devient invérifiable.

**Impact mainteneur**: la saga réutilise les patterns d'outbox et de rejeu déjà
présents; aucun coordinateur générique n'est ajouté.

## Décision 4 - Compatibilité explicite des projets non enregistrés

**Décision**: aucun enregistrement automatique. Les commandes historiques
continuent avec `project_id=None` et apparaissent `unregistered`.

**Alternative considérée**: scanner les dépôts et créer automatiquement les
projets.

**Rejet**: un scan inventerait des identités, confondrait dépôts et worktrees et
pourrait enrôler des chemins non voulus.

**Impact mainteneur**: le déploiement est réversible et le diagnostic distingue
clairement ancien et nouveau modèle.

## Décision 5 - Canonicaliser sur l'hôte qui possède le chemin

**Décision**: Maicie valide la forme, Bridget canonicalise et décide la liaison
car elle possède l'hôte réel.

**Alternative considérée**: faire confiance au chemin envoyé par Maicie.

**Rejet**: liens symboliques, chemins disparus et différences d'hôte ne peuvent
être tranchés correctement par le plan de mission.

## Baseline et validation externe

- La constitution locale impose un registre explicite, des frontières
  d'autorité et une progression réversible:
  `/home/moi/.speckit/constitution.md`.
- Les recommandations d'orchestration insistent sur mandat, corrélation et
  bornes explicites:
  `/home/moi/.speckit/ref/agent-orchestration.md`.
- OpenAI décrit l'isolation comme une politique explicite de filesystem,
  réseau et approbation, pas comme une propriété déduite du nom d'un projet:
  https://openai.com/index/running-codex-safely/
- Claude Code documente les environnements de développement comme une
  configuration déclarée et versionnable:
  https://code.claude.com/docs/en/devcontainer
- Cursor associe ses agents de fond à une machine isolée et à un dépôt cloné,
  ce qui confirme la nécessité d'une identité et d'une liaison explicites:
  https://docs.cursor.com/background-agent

## Red flags retenus

- Ne pas présenter l'enregistrement comme une isolation de sécurité.
- Ne pas créer de registre global partagé en écriture.
- Ne pas auto-migrer les dépôts historiques.
- Ne pas utiliser le nom du dossier comme identité.
- Ne pas coupler la réussite métier à la disponibilité instantanée de Bridget.

## Complément de décision RC8 - 2026-08-30

- Les racines admises ne sont plus une hypothèse: Bridget charge une politique
  hôte fermée depuis un chemin absolu explicite et ferme les mutations si elle
  manque ou si ses permissions sont invalides.
- Le transport de mutation est une variante locale Maicie vers Bridget,
  versionnée, négociée et authentifiée par rôle/capability/UID. Le flux
  `ServiceRequest` existant garde sa direction Bridget vers Maicie.
- ProjectReference est une donnée durable de l'exécution, pas une propriété de
  l'UI: elle traverse ordres, leases, état désiré, snapshots, projections et
  curseurs de reprise.
- `review_project` n'est jamais migré au démarrage. Une prévisualisation et une
  confirmation locale explicites sont requises.

## Complément après intégration de SPEC-068 - 2026-08-30

- `main` à `d589b24` contient désormais les liaisons parent-enfant, événements
  de liaison et incidents runtime délégués durables de SPEC-068. Ils deviennent
  des porteurs obligatoires de `ProjectReference`; aucune identité ne peut être
  redéduite de `cwd`, `domain`, `instance_id` ou `execution_id` au rejeu.
- L'audit du registre est une donnée durable distincte de l'historique de
  liaison. Il est écrit dans la même transaction que la mutation et son
  identifiant déterministe empêche un doublon lors du rejeu d'une commande.
