# Vérification 241

1. Charger spec.md, plan.md, reuse-audit.md puis tasks.md de cette session.
2. Exécuter les tests filtrés uniquement avec python3 scripts/build.py cargo ...
   et le cache réutilisé du checkout. Les noms et sorties seront dans validation.md.
3. Vérifier les pertes de réponse sur un magasin temporaire et de vrais sockets,
   sans reprendre une tâche métier. Relire la même clé et compter les tâches.
4. Vérifier la fin d'un tour, une réduction réelle de droits et une annulation.
5. Vérifier le répertoire local supplémentaire, le hors-Git autorisé et Docker
   absent/présent/inutilisable sans installation ou lancement de service.
6. Comparer les faits de première tentative, récupération, délais et actions
   humaines. Les lectures de production restent bornées et sans contenu métier.
7. Avant activation : vérifier images, retour, formats et registres ; après :
   processus chargés, lien natif et lectures réelles. T3 reste ouvert.
