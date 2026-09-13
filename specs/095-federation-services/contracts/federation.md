# Contrat CLI 095

`scripts/federate-ssh.sh run` conserve les options 089 et son exécution au premier plan.
`install` reçoit les mêmes paramètres de liaison et installe explicitement un service utilisateur autonome ; `status --label NAME` rapporte son état natif ; `remove --label NAME` retire uniquement ce service et ses fichiers possédés, jamais les données Bridget ni les clés SSH.
Plateforme détectée, pas de lancement d'un daemon supplémentaire. Même interface macOS/Linux. Le service transporte exclusivement la socket maître configurée.
Direction fixe : le service est installé sur la machine MAÎTRE et exécute `ssh -R remote_socket:socket`. Pour US2, installation sur le Mac, endpoint client sur Cartae. systemd est testé séparément pour un maître Linux ; aucun service inverse ni unité cliente vide n'est installé en production sur Cartae.
L'implémentation peut définir un emplacement privé déterministe documenté pour ses copies. Pas de réutilisation implicite d'un fichier historique, de shell eval, de root ni de fallback d'autorité.
Un échec de bind ou de clé d'hôte doit rester visible. Une socket étrangère ne peut jamais être retirée. Les fichiers de service préexistants incompatibles sont refusés, pas écrasés.
Première installation : préflight distant strict avant publication, même une socket périmée préexistante est refusée. Une installation reconnue reprend uniquement sa configuration privée. Le retrait désactive/arrête d'abord le service et atteste cet arrêt, puis nettoie uniquement la socket périmée configurée par la même sonde bornée (ECONNREFUSED et inode inchangé), sans rebind. Si arrêt/nettoyage non confirmé, conserver les fichiers pour une nouvelle tentative ; ne pas annoncer un retrait réussi. Cela permet remove→install sans effacement manuel aveugle.
