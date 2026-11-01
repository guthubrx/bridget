# 223 — Publication Android cohérente
Statut : Livré. Correction demandée le 1 novembre 2026.

L'utilisateur doit pouvoir mettre à jour Bridget 0.7.0 vers 0.7.1 sans désinstaller
l'application. Le paquet publié doit avoir la version annoncée et le certificat
existant. Les contrôles Android de version, taille, empreinte et signature restent
inchangés. Les données présentes sont conservées.

La release v0.7.1 contenait l'APK 0.7.0/7000 : le refus est reproduit.
Résultat requis : APK 0.7.1/7001 signé, publié et installé par le parcours réel
sur l'émulateur Android 15. Le téléphone physique de François n'est pas connecté.
La publication doit refuser une autre version ou un autre certificat.
