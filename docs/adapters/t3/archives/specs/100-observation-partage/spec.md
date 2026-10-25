# Archives techniques de l’adaptateur T3

Document d’origine : [specs/100-observation-partage/spec.md](../../../../../../specs/100-observation-partage/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Bridget est le produit de communication inter-agents/inter-fournisseurs ; T3 est
un adaptateur, Maicie n'est pas requis. L'utilisateur veut partager un passage
utile pour une relecture, être averti d'un fait précis et repérer les écritures
concurrentes. Aucun profil, formulaire, mandat, approbation métier, verrou de
fichier ou condition préalable supplémentaire au travail des agents.

<a id="extrait-2"></a>

## Extrait 2

- FR01 : lecture et partage bornés, disponibles en CLI et MCP, sans dépendance T3/Maicie.
- FR02 : provenance agent/séquences, ordre conservé, Unicode valide, lacunes et limites explicites ; aucun résumé inventé.
- FR03 : partage par la messagerie existante, compatible avec demande de réponse (`reply`) ; aucun nouveau suivi métier.
- FR04 : catalogue fermé et expliqué : fin de tour, permission requise, écriture observée, collision observée ; filtres agent et fichier pertinents combinables.
- FR05 : création, liste, suppression, mode ponctuel et expiration ; seuls les faits reçus par le daemon après l'abonnement sont concernés, sans rejeu du journal. Un fait déjà produit mais encore en transit peut être reçu après l'abonnement ; aucune synchronisation des horloges source n'est promise.
- FR06 : propriétaire dérivé de l'identité autorisée ; aucune création ou suppression pour un tiers par simple déclaration.
- FR07 : notifications ordinaires, non bloquantes pour l'agent observé, sans boucle de notifications ni attente active d'un LLM.
- FR08 : fin de tour ne signifie ni mission terminée ni succès ; silence et déconnexion ne sont pas une preuve de fin.
- FR09 : collisions fondées uniquement sur écritures structurées attribuées, même hôte et même chemin absolu normalisé, auteurs distincts et fenêtre de 30 secondes.
- FR10 : lectures et outils non écrivains exclus ; couverture limitée aux signaux des intégrations, absence de surveillance universelle du disque ou d'interprétation de shell libre.
- FR11 : mémoire, taille des extraits, abonnements et débit bornés ; saturation visible et aucun ralentissement non borné de la messagerie.
- FR12 : documenter la portée locale des observations, les intégrations couvertes, les pertes possibles et la durée de vie des abonnements.
- FR13 : tests isolés seulement ; préserver modifications 099 et demandes/réponses existantes. L'interdiction initiale de commit/déploiement est levée par la demande explicite du 2026-09-16 : fusion locale puis adoption sauvegardée et vérifiée, sans arrêter les conversations fournisseur.

<a id="extrait-3"></a>

## Extrait 3

Les collisions se consultent par le même mécanisme d'abonnement, sans alerte
globale imposée. Le mode shell libre et les adaptateurs ne remontant pas les
chemins ne sont pas couverts ; aucune prétention de couverture T3 complète.
Les filtres portent sur les faits du catalogue, pas sur une condition libre
comme « le code est bon », pas de SQL ou de script utilisateur exécuté.
