# Recherche 034 — Orchestration par graphe de contrôle

**Statut** : recherche consolidée au 26 août 2026

**Base documentaire** : `b0a8cea00bddaa50ee243042f90eb141ba50e467`

**Objectif Maicie** : `1be4ff94-1f68-4b6f-a403-ebab106cbcde`

**Délégation** : `c11dbcb1-4307-4325-af35-a7ea7f56dbd5`

**Message d'origine** : `b0545066-4482-411c-931f-cb7b14059d01`

## But et méthode

Cette recherche répond à une question opérationnelle : expliciter le graphe
de contrôle déjà suivi par Bridget et Maicie apporte-t-il une protection que
des refus machine placés aux bonnes frontières n'apporteraient pas déjà ?

Elle ne constitue pas un panorama de produits. Elle confronte les huit défauts
d'orchestration observés en douze heures à des sources primaires, en cherchant
en priorité les échecs, les coûts de maintenance et les contradictions.

Légende employée dans tout le document :

- **[MESURÉ]** : observation locale ou mesure publiée ;
- **[LU]** : propriété explicitement documentée par une source primaire ;
- **[PRÉDIT]** : conséquence proposée pour Bridget et Maicie.

Les affirmations de vendeurs ou d'opérateurs sont signalées comme telles. Une
source de 2025 est marquée **SOURCE 2025**. L'absence de donnée publiée est
déclarée au lieu d'être comblée par une hypothèse.

### Corpus local versionné

Les faits locaux ne sont pas reconstitués depuis des messages. Ils sont lus
dans `docs/catalogue-du-du.md` au SHA
`b0a8cea00bddaa50ee243042f90eb141ba50e467` :

| Ligne | Identifiant stable | Rôle dans la démarche |
|---:|---|---|
| 278 | `doctrine/murs-sur-le-progres-rattrapage-confronte-sur-la-visibilite` | diagnostic, réfutation du « manque d'observation », visibilité/progrès et traitement des débordements |
| 280 | `doctrine/cinq-refus-machine-et-etats-de-revue-chez-maicie` | cinq refus, bornes et états de revue manquants |
| 284 | `etude/control-graph-gouverner-les-engagements-pas-la-pensee` | synthèse externe et corrections de doctrine |
| 285 | `constat/le-fond-de-panier-est-fait-de-victoires-non-soldees` | mesure postérieure : sept résolutions non soldées sur douze anciennes entrées |

Le catalogue local de l'agent distant, déclaré à
`/home/moi/.cache/bridget/catalogue.jsonl`, était vide au moment de la
première lecture. Cette absence a été déclarée au lieu d'être comblée. La
source versionnée ci-dessus a ensuite été publiée sur `main` et relue dans le
worktree dédié.

## Trois sens du mot « graphe »

La discussion publique mélange trois objets différents :

1. **Graphe de contrôle** : des nœuds qui exécutent une action, des arêtes qui
   déterminent la suite et un état transporté. C'est le seul objet étudié ici.
2. **Graphe de connaissances** : des relations entre entités utilisées pour la
   recherche d'information. Les résultats sur ce sujet polluent fortement les
   recherches, mais ils sont hors périmètre.
3. **Graphe de boucles** : composition de plusieurs boucles autonomes, de leurs
   erreurs et de leurs améliorations. Le domaine est encore peu stabilisé. Il
   est hors périmètre immédiat, mais constitue un sujet possible à douze mois.

Le tweet déclencheur de Peter Steinberger disait : « Are we still talking
loops or did we shift to graphs yet? ». Son identifiant X correspond au
18 juillet 2026 à 00:34:54 UTC, donc au 17 juillet dans des fuseaux américains.
Il ne définit ni produit, ni méthode, ni résultat : c'est une provocation, pas
une source technique. [Tweet, 17/18 juillet 2026](https://x.com/steipete/status/2078277297791189132)

## Verdict de recherche

**Le mouvement des boucles vers les graphes est réel, mais le mot « graphe »
mélange une notation, un moteur durable et une discipline de contrôle. La
notation seule ne produit pas de gain.**

- **[LU]** Des graphes de contrôle fonctionnent réellement en production.
  OpenAI Symphony transforme un gestionnaire de tickets en plan de contrôle ;
  LangGraph est utilisé chez Lyft et LATAM ; Temporal fournit la durabilité
  d'un workflow écrit en code. Microsoft Agent Framework, Google ADK et
  CrewAI exposent aussi des graphes explicites.
- **[LU]** Un graphe n'a pas besoin d'une syntaxe graphique. Microsoft expose
  une API fonctionnelle et une API graphe au-dessus du même modèle
  d'exécution, avec les mêmes résultats observables. Symphony est une
  procédure versionnée (`SPEC.md` et `WORKFLOW.md`) exécutée par un
  ordonnanceur et des scripts déterministes.
- **[MESURÉ local]** Bridget et Maicie possèdent déjà un graphe implicite.
  Les incidents sont des arêtes manquantes ou non opposables, pas une absence
  de dessin.
- **[PRÉDIT]** La cible adaptée est un **macrographe exécutable et petit** dans
  le guichet central existant : événements durables, état versionné, refus sur
  les frontières irréversibles et réconciliation des omissions. Le
  raisonnement interne de l'agent reste une boucle souple orientée objectif.

La thèse initiale est donc confirmée pour moitié : nommer ou visualiser le
graphe ne protège rien. Elle est trop forte lorsqu'elle affirme qu'il n'existe
aucun gain avant les refus : un journal structuré rend déjà détectables une
livraison non routée, un verdict non inscrit ou une tête périmée. Cette
visibilité ne devient toutefois protectrice qu'avec une réconciliation ou un
refus machine.

## A. Qui le fait réellement, et sous quelle forme ?

### OpenAI Symphony : la procédure écrite devenue contrôle opératoire

**[LU]** Symphony utilise Linear comme plan de contrôle : statuts de tickets,
dépendances, espaces de travail isolés, reprises, réconciliation, CI et passage
à la revue. L'ordonnanceur garde les décisions déterministes, tandis que la
politique et le comportement de l'agent restent versionnés dans des fichiers
écrits.

**[MESURÉ — opérateur, non audité]** OpenAI rapporte jusqu'à 500 % de pull
requests fusionnées en plus sur certaines équipes pendant les trois premières
semaines, sans groupe témoin. Le même retour situe à trois à cinq le nombre de
sessions qu'un humain gère confortablement avant que la productivité baisse.
[OpenAI, 27 avril 2026](https://openai.com/index/open-source-codex-orchestration-symphony/) ;
[spécification Symphony, consultée le 26 août 2026](https://github.com/openai/symphony/blob/main/SPEC.md)

La contradiction centrale vient de la même source : les premières versions
traitaient les agents comme des nœuds rigides d'une machine d'état et cette
forme s'est révélée trop limitée. OpenAI est passé à des objectifs larges.
Symphony conserve donc un graphe dur **autour** du travail, pas à l'intérieur
du raisonnement.

### LangGraph : un graphe réel, rendu plus dynamique par la production

**[LU]** LangGraph fournit nœuds, arêtes, cycles, checkpoints, interruptions,
historique d'état et reprise. Son retour 2026 indique que certains systèmes de
recherche ont commencé avec un workflow prédéfini puis déplacé leur centre
vers une boucle plus agentique ; les graphes de production sont souvent
cycliques et leurs transitions dynamiques.

[LangChain, 22 juillet 2026](https://www.langchain.com/blog/3-years-of-graph-engineering-with-langgraph) ;
[persistance LangGraph, consultée le 26 août 2026](https://docs.langchain.com/oss/python/langgraph/persistence)

### Microsoft, Temporal et AWS : le runtime vaut plus que le dessin

- **[LU]** Microsoft Agent Framework donne les mêmes résultats observables
  avec l'API fonctionnelle (`if`, boucles, `gather`) et l'API graphe. Le choix
  de notation n'est donc pas la source de la durabilité.
  [Microsoft Agent Framework, documentation mise à jour le 25 août 2026](https://learn.microsoft.com/en-us/agent-framework/concepts/workflows/)
- **[LU]** Temporal est un moteur de workflow durable, pas une syntaxe de
  graphe agentique. Son historique permet de reconstruire l'état après panne.
  [Temporal Event History, consultée le 26 août 2026](https://docs.temporal.io/encyclopedia/event-history)
- **[LU]** L'échantillon AWS Collaborative AI-DLC est presque isomorphe au
  cycle étudié : intention, exigences, unités en DAG, agents parallèles,
  capteurs déterministes, revue LLM, barrières humaines et merge possédé par
  le moteur. Sa version 2.0.0 date du 6 août 2026, mais le dépôt se déclare
  encore *early preview* et à durcir avant production. C'est une procédure
  documentée, pas un déploiement client prouvé.
  [AWS AI-DLC](https://github.com/aws-samples/sample-collaborative-ai-dlc/) ;
  [architecture](https://aws-samples.github.io/sample-collaborative-ai-dlc/concepts/architecture/) ;
  [exécution](https://aws-samples.github.io/sample-collaborative-ai-dlc/concepts/execution/)

**Conclusion A** : l'affirmation « procédure écrite plus scripts
déterministes » est étayée. Le graphe peut être exprimé en Markdown et en code
fonctionnel. Sa valeur vient des transitions opposables, de l'état durable et
de la reprise, pas de la syntaxe.

## B. Échecs et signaux d'alerte

### Camisole cognitive

**[LU]** Symphony fournit le contre-exemple recherché : OpenAI a essayé de
faire des agents des nœuds rigides et a abandonné cette granularité. Un graphe
est adapté aux engagements vérifiables ; il devient une camisole lorsqu'il
prescrit le chemin cognitif complet.

### Évolution des graphes persistés

**[LU]** LangGraph considère chaque modification d'un graphe persistant comme
un problème de compatibilité. Des fils suspendus peuvent devenir non
reprenables. Un nœud interrompu redémarre depuis son début, ce qui oblige à
rendre les effets de bord idempotents.

[Compatibilité LangGraph, consultée le 26 août 2026](https://docs.langchain.com/oss/python/langgraph/backward-compatibility) ;
[interruptions LangGraph](https://docs.langchain.com/oss/python/langgraph/interrupts)

**[LU]** Microsoft exige la même topologie et les mêmes identités d'exécuteurs
pour réhydrater un checkpoint. Changer un nom ou un branchement peut donc
invalider un travail en vol.
[Checkpoints Microsoft Agent Framework, consultés le 26 août 2026](https://learn.microsoft.com/en-us/agent-framework/workflows/checkpoints)

### État lisible mais faux

**[LU]** AutoGen GraphFlow reste explicitement expérimental. Une issue du
20 septembre 2025 décrit une restauration où la file prête devient vide alors
que du travail reste, menant à un faux état `complete`.
[AutoGen GraphFlow](https://microsoft.github.io/autogen/dev/user-guide/agentchat-user-guide/graph-flow.html) ;
[issue AutoGen 7043 — SOURCE 2025](https://github.com/microsoft/autogen/issues/7043)

**[LU]** Google ADK 2.0 documente une reprise avec exécution d'outils *at
least once* : les achats ou autres effets de bord doivent être dédupliqués par
l'application. CrewAI qualifie ses checkpoints automatiques de *best effort* :
une écriture ratée peut seulement être journalisée pendant que l'exécution
continue.
[ADK resumability, consultée le 26 août 2026](https://adk.dev/runtime/resume/) ;
[CrewAI checkpointing](https://docs.crewai.com/v1.15.17/en/concepts/checkpointing)

### Coût des structures intermédiaires

**[MESURÉ — retour opérateur non audité]** LATAM rapporte environ 15 % de
latence et de tokens perdus dans une architecture où chaque spécialiste
restructurait sa sortie. Une simplification a conservé la qualité et réduit le
coût d'environ 15 %.
[LangChain/LATAM, 4 août 2026](https://www.langchain.com/blog/customer-experience-cx-agents-in-production-lessons-from-lyft-vodafone-and-latam-airlines)

### Échecs organisationnels

**[MESURÉ — SOURCE 2025]** MAST analyse 1 642 traces sur sept systèmes et
observe des taux d'échec de 41 % à 86,7 % sur des benchmarks qui ne sont pas
directement comparables. Dans son échantillon de modes d'échec, 41,8 % relèvent
du design système, 36,9 % de l'alignement inter-agents et 21,3 % de la
vérification. Ajouter une vérification de l'objectif améliore ChatDev de
15,6 %, sans rendre le système fiable.
[MAST, NeurIPS 2025](https://proceedings.neurips.cc/paper_files/paper/2025/file/b1041e52d3be19f0a9bc491657488e4a-Paper-Datasets_and_Benchmarks_Track.pdf)

### Défenses partielles

**[LU]** Les garde-fous de l'OpenAI Agents SDK n'opèrent pas tous aux mêmes
frontières : ceux des outils ne couvrent ni les handoffs, ni plusieurs outils
intégrés. Posséder des garde-fous ne signifie donc pas que chaque arête
critique est protégée.
[OpenAI Agents SDK, consulté le 26 août 2026](https://openai.github.io/openai-agents-python/guardrails/)

**Conclusion B** : un moteur de graphe ne supprime ni l'idempotence, ni la
compatibilité de schéma, ni la vérification métier. Il peut même rendre un
état faux plus convaincant. Le coût est proportionnel au nombre de transitions
persistées et aux workflows en vol.

## C. Où stocker l'état et comment traiter sa péremption ?

Les approches sérieuses séparent deux lectures :

- un journal durable répond à « que s'est-il passé ? » ;
- une projection rapide répond à « où en sommes-nous ? ».

**[LU]** Temporal illustre la frontière. Son historique est autoritatif pour
une exécution, tandis que son index global `Visibility` est mis à jour de façon
asynchrone, peut être périmé plusieurs secondes ou davantage et n'a pas de SLA
de fraîcheur. Temporal demande de lire l'exécution elle-même pour une décision
et de réserver l'index à la recherche et à l'observabilité.
[Temporal Visibility, consultée le 26 août 2026](https://docs.temporal.io/visibility)

**[LU]** AWS AI-DLC utilise DynamoDB comme vérité du processus, Neptune comme
index de traçabilité reconstruisible et S3 pour les corps d'artefacts. Le
travail n'est terminé qu'après relecture de la tête Git exacte ; un retour en
arrière supersède les artefacts sans les supprimer.

**[LU]** L'event sourcing intégral est un remède coûteux. Microsoft avertit
qu'il change stockage, concurrence, migrations et requêtes, contraint les
évolutions futures et n'est justifié que sélectivement.
[Azure Event Sourcing, 28 mars 2026](https://learn.microsoft.com/en-us/azure/architecture/patterns/event-sourcing)

**[PRÉDIT]** Pour Bridget/Maicie : journaliser seulement les transitions de
contrôle, maintenir une projection `current_status` avec révision monotone et
dernier identifiant d'événement, puis exiger `expected_revision` sur toute
décision issue d'une vue. Les pensées, plans intermédiaires et messages restent
hors de ce journal métier.

Une différence locale doit être assumée : « tout refus laisse une ligne
durable » est plus strict que Temporal. Temporal documente qu'une mise à jour
rejetée est retournée et comptée, mais que l'événement de rejet n'est jamais
écrit dans l'historique du workflow. La durabilité des refus reste donc une
propriété métier à construire, pas un comportement gratuit d'un moteur.
[Temporal, documentation consultée le 26 août 2026](https://github.com/temporalio/documentation/blob/main/docs/evaluate/temporal-cloud/actions.mdx)

## D. Accueillir l'imprévu sans élargir silencieusement le périmètre

**[LU]** Symphony permet à un agent qui découvre un problème hors périmètre de
créer un nouveau ticket, évalué et planifié plus tard. Sa spécification
publique n'impose cependant ni provenance complète, ni clé de déduplication,
ni politique d'admission.

**[MESURÉ — retour opérateur non audité]** Chez LATAM, 13 % des messages
étaient classés hors périmètre. L'examen a montré que 95 % correspondaient à de
vrais besoins de voyageurs. Ajouter un spécialiste a ramené le taux de 13 % à
1 %. Cela soutient une voie de capture et de triage, pas l'extension
automatique du travail courant.

**[PRÉDIT]** Une entrée latérale `finding_proposed` doit porter le parent
`objective_id/run_id/checkpoint`, une preuve ou un extrait, un type fermé et
une clé de déduplication. Elle ne modifie pas le périmètre et ne démarre aucun
travail. Une étape distincte décide `promote_to_objective | duplicate | reject
| defer` et conserve la causalité.

Je n'ai trouvé aucun standard public complet pour la provenance, la
déduplication et l'admission de ces découvertes. C'est un angle mort du
domaine, pas un détail déjà résolu par les frameworks.

## E. Un vérificateur doit-il être un nœud séparé ?

**Verdict : la séparation topologique seule n'a pas de gain causal démontré.**
Une seconde instance nommée `reviewer`, avec le même modèle, le même contexte
et les mêmes informations, ne crée pas d'indépendance statistique.

- **[MESURÉ]** L'auto-correction intrinsèque peut dégrader GPT-4 : sur GSM8K,
  95,5 % au premier jet, 91,5 % après une révision et 89,0 % après deux.
  [Huang et al., 3 octobre 2023, ICLR 2024](https://arxiv.org/abs/2310.01798)
- **[MESURÉ]** Un critique spécialisé apporte un signal différent : une équipe
  Human+CriticGPT est préférée à un humain seul dans plus de 60 % des
  comparaisons ; CriticGPT est préféré à ChatGPT dans 63 % des critiques de
  bugs naturels. OpenAI signale aussi ses hallucinations et ses limites.
  [OpenAI CriticGPT, 27 juin 2024](https://openai.com/index/finding-gpt4s-mistakes-with-gpt-4/)
- **[MESURÉ — SOURCE 2025]** MAST mesure +15,6 % quand la vérification porte
  explicitement l'objectif de haut niveau. Le gain vient de l'oracle ajouté,
  pas du nom du nœud.

**[PRÉDIT]** Un vérificateur séparé mérite d'être conservé seulement s'il
apporte au moins un signal orthogonal : tests exécutables, source externe,
contexte masqué au producteur, données spécialisées, arbitrage humain ou droit
d'abstention et d'escalade. `author != reviewer` reste une garde utile contre
le conflit d'intérêt ; ce n'est pas une preuve de qualité.

Hiérarchie pratique : oracle déterministe valide, puis humain assisté d'un
critique spécialisé, puis relecteur avec preuves ou contexte distincts, puis
autre instance du même modèle, puis auto-révision du producteur.

## Ce que la recherche n'a pas trouvé

- Aucun benchmark contrôlé montrant qu'une syntaxe graphe est meilleure que du
  code fonctionnel sur le même runtime.
- Aucun framework généraliste qui implémente nativement mandat, preuve,
  jugement, version revue, merge et clôture.
- Aucune mesure publique comparative du coût de maintenance des graphes
  persistés, ni seuil universel de « trop de branches ».
- Aucun standard complet pour les découvertes latérales.
- Aucune preuve qu'un autre fournisseur ou modèle suffit à décorréler les
  erreurs de vérification.
- Aucune mesure permettant d'affirmer qu'un graphe sans refus ni
  réconciliation aurait empêché les huit incidents locaux.

## Conclusion

**[PRÉDIT]** Bridget n'a pas besoin d'« adopter les graphes ». Elle doit rendre
explicite et exécutable le graphe qu'elle suit déjà. Le progrès vient du
passage de promesses textuelles à quatre objets vérifiables : événement
durable, état versionné, refus typé et réconciliation. Le graphe gouverne les
engagements ; il ne gouverne pas la pensée.
