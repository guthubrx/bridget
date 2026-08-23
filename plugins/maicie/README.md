# Maicie — frontière de plugin

Ce répertoire est la frontière de propriété de Maicie v3.

Les futurs développements de Maicie — exécutable compagnon, configuration,
persistance privée, contrats et tests — devront vivre sous `plugins/maicie/`.
Maicie n'est pas du code chargé dans le démon Bridget : c'est un processus
distinct qui utilise uniquement les interfaces publiques de Bridget.

Ce fichier ne constitue pas une implémentation. Les sources et le manifeste
Cargo seront créés à l'exécution des tâches de
`specs/011-maicie-orchestration/tasks.md`.
