# Revue adverse — boucle d'amélioration continue générique (2026-08-24)

**Croisement** : relecture par un moteur distinct de celui de l'auteur.
**Objet** : proposition utilisateur d'étendre « Maicie greffière du
catalogue » en boucle d'amélioration multi-projets (constats → priorisation
→ entrée au plan de travail). **Verdict : AMENDER.**

## Frontière déterministe/jugement (FR-022)

« Priorisées correctement » est le cheval de Troie : la correction d'une
priorité est un jugement. Mécanisable = tri lexicographique sur des faits
DÉCLARÉS ou observés uniquement : sévérité déclarée (enum fermée, jamais
inférée), récurrence si et seulement si `recurrence_of:<id>` est déclaré
(reconnaître « le même incident » est un jugement de similarité — interdit),
âge, lien à un gate raté (fait journalisé). Réservé à l'humain/référent :
attribuer la sévérité, fusionner/dédupliquer, arbitrer inter-projets,
décider qu'un constat devient une tâche. La capture de texte libre verbatim
est légale ; en tirer une décision ne l'est pas.

## Dérive Jira — point de bascule

Greffière = écrit des faits ; gestionnaire de projet = prescrit ce qui se
travaille. Dès que Maicie écrit dans tasks.md ou un plan, elle redevient le
moteur de workflow que la v3 a fui (ADR 003). Garde-fou : vue triée en
LECTURE (`registre list`) ; l'insertion au plan est un acte humain que
Maicie peut journaliser, jamais prendre.

## Multi-projets — source unique préservée

Ni écriture dans les artefacts de l'hôte (adaptateurs en écriture couplés à
des formats tiers : indéfendable), ni registre parallèle (interdit).
Résolution : Maicie apporte SON format de catalogue au projet hôte — un
fichier conventionné, versionné dans le repo hôte, déclaré en config. Un
catalogue par projet. L'intégration au backlog préexistant reste humaine.

## Constat imprévu : le catalogue actuel n'est pas machine-appendable

C'est de la prose de référent (entrées narratives, marqueurs composites).
La greffière mono-projet déjà décidée exige D'ABORD un format d'entrée
fermé + la migration du catalogue-prose ; la généricité en découle
gratuitement.

## Mode planifié par la porte de derrière

« Intègre la todo-liste » = l'état intermédiaire rejeté le 2026-08-23.
Sa voie de réhabilitation reste inchangée : adossé aux routines (bloc F),
jamais seul — et même alors, décision humaine journalisée.

## Noyau minimal (XIX)

1. `constat add` : schéma fermé (id, date, mission source, sévérité déclarée,
   recurrence_of optionnel, texte verbatim) appendu au catalogue du projet.
2. Transitions d'état automatiques à la clôture d'objectif (guichet 015).
3. Vue triée déterministe en lecture seule.

Gadgets : écriture dans les plans/issues de l'hôte, adaptateurs Jira/GitHub,
dédup automatique, score composite, comparaison inter-projets, tout état
« planifié » avant les routines.

## Conditions non négociables

1. Maicie TRIE, ne juge jamais — priorité = pure fonction de champs déclarés.
2. Jamais d'écriture dans les artefacts de planification de l'hôte.
3. La greffière mono-projet livre et prouve le format d'abord ; le volet
   « todo » attend les routines du bloc F.
