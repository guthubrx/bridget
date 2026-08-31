# Checklist de qualité - SPEC-080 Centre de contrôle Bridget

## Qualité de la spécification

- [x] Le problème utilisateur, la priorité serveur et la portée du centre de contrôle sont explicitement définis.
- [x] Les rôles Mac, serveur, projet et agent sont distingués sans chevauchement d'autorité.
- [x] Les fonctionnalités demandées sont décrites par résultats utilisateurs et non par une dépendance ou un framework imposé.
- [x] Les six user stories possèdent des scénarios d'acceptation observables.
- [x] Les cas de conflit de génération, serveur absent, capacité retirée et données d'usage incomplètes sont couverts.
- [x] Les exigences fonctionnelles sont numérotées, testables et sans marqueur à clarifier.
- [x] Les exigences non fonctionnelles couvrent latence, atomicité, confidentialité, accessibilité, déconnexion et complexité.
- [x] Les limites de sécurité interdisent explicitement le shell arbitraire, les secrets, les approbations distantes et les mutations agent.
- [x] Les coûts sont définis comme estimations API avec provenance et non comme facturation.
- [x] Les mises à jour sont explicitement informatives et ne déclenchent pas une maintenance distante.
- [x] Les critères de succès sont mesurables et reliés aux user stories.
- [x] Les tests requis couvrent daemon, transport, interface, usage, intégration et vérification manuelle.

## Résultat

Spécification complète et prête pour le plan technique. Les détails de schéma, de protocole et de placement des fichiers seront déterminés par l'audit de réutilisation et consignés dans `plan.md`.
