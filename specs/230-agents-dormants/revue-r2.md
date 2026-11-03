# Revue R2 et dernier contrôle

Revue native d199a6b6-1800-4d26-a350-ecbc40fbf4f8, Sonnet choisi par Bridget,
source ffa8fdf4, lecture seule sans test. Verdict : aucun P1, un P2 conditionnel
et trois questions. Résultat et reçu conservés dans le cache privé230.
Clôture propriétaire confirmée, cleanup_done=true, enfant arrêté. Vue conservée.

- Le P2 conditionnel porte sur l'autorité native de la mission déjà acceptée.
  Le prédicat `admitted_mission_exchange` exige la connexion interne exacte,
  le message canonique complet, mission_pending et les deux instances encore
  autorisées. Il ne couvre aucun message ultérieur arbitraire. Le test ancien
  native174 a été renforcé avec un enfant dormant : mission exacte acceptée,
  corps, destinataire, instance ou mandat modifiés refusés. PASS0,45s.
- Une nouvelle ronde est une nouvelle sollicitation : défaut confirmé et
  corrigé avant activation. Refus agent_dormant avant les preuves distantes
  puis au point d'admission. Un rejeu déjà admis rend son reçu avant la garde.
  Le test réel de continuation vérifie refus sans perte du résultat, réveil,
  admission unique puis sommeil et rejeu sans seconde ronde. PASS.
- Propriétaire d'un rappel : SelfTarget contrôle le propriétaire car il est
  la cible. Agent contrôle l'agent cible. Dormir n'interdit pas d'émettre une
  demande autorisée vers un autre agent actif. L'objection de lecture ne
  correspond pas à une garde globale sur l'expéditeur.
- La désactivation après une commande partie peut laisser des classements
  différents. C'est affiché local_only après confirmation, et expliqué dans
  le guide de l'adaptateur. UI Android non vérifiée : émulateur indisponible.

Dernier lot :19tests tracking230 PASS1,15s (incluant continuation et guidage),
plus le test d'autorité native renforcé, et2tests de liaison acquis précédemment.
Les tests Android existants ont39cas verts dans7suites. Assemblage Android0.8
après Tous acquis. Ces résultats sont ceux de l'auteur ; les revues restent
explicitement des lectures indépendantes, sans exécution.

Incident de clôture R1 : cause exacte du délai inconnue. Sonde native locale
mesurée9,145s, même lecture au maître0,017s. Contrôles de configuration du relais
locaux <1s, SSHisolé environ1s ; point lent exact non identifié. Le nouveau
guidage avant départ nomme l'étape (lecture des capacités ou raccordement),
la tâche et l'absence de transmission de CET appel. Aucun budget ni droit
n'a été élargi. Un test vérifie ce guidage ; aucune répétition de mission.

Dernier delta auto-audité après R2 : garde de nouvelle ronde, maintien du
rejeu antérieur et diagnostic préalable. Pas de troisième mission de revue
créée pour ces corrections ciblées couvertes par les tests.


## Vérification réelle après activationR3 : relais Mac

La commande agents depuis le Mac échoue encore sur la sonde d'identité
préalable à2secondes. La même commande au maître réussit. Le premier octet
sur le relais natif a été mesuré9,145s puis3,725s ; la seconde réponse sur la
même connexion prend0,007s. L'amorçage du flux est donc compris à tort dans
une sonde trop courte. Le point lent exact du listener n'est pas identifié.

Correction cibléeR4 : agents appelle directement son annuaire déjà négocié,
avec les mêmes contrôles de rôle, version et portée. Suppression de la sonde
en double, aucune suppression de validation et aucune liste vide sur erreur.
La sonde wait_v1 emploie désormais le budget client commun10s au lieu de3s,
avant toute commande151. Les délais de mission et droits sont inchangés.
Un serveur simulé retardé de3,2s reproduit cette amorce ; un pair muet reste
borné par10s. Reprise de la commande réelle du Mac requise après chargement.
Ce changement réel justifie l'assemblageR4 ; Android est inchangé.
