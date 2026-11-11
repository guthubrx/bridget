# Faits et correction 276

Deux demandes bridget_auto_delegate ont eu leur réponse perdue. Les reçus natifs
du propriétaire attestent les tâches f94d5042 et fc06d175, toutes deux admises.
Leurs identifiants complets et enfants ont été rendus au responsable par le
parcours normal, sans rejouer d’admission. Le premier appel était déjà admis ;
le second a été admis par le rejeu strict du propriétaire. Ce n’est pas une preuve
que toute recherche par rejeu serait sans effet.

GuichetLookup consulte seulement le registre historique du guichet. Une clé
absente y rend le même code idempotency_expired qu’une protection expirée. Le MCP
en déduisait applied:false sans résultat métier. Cela décrit à tort une absence
d’application et masque la séparation avec les admissions natives.

La projection sans payload métier rend maintenant applied:null, scope explicite
guichet_request et native_admission_checked:false. Cela couvre aussi queued,
outcome_unknown et une coupure après dépôt. Les résultats terminaux qui doivent
avoir un payload continuent de refuser son absence. Les résultats métier prouvés
gardent leur booléen. Aucun format wire, stockage, droit ou accès de registre ne
change. Aucun rapprochement natif n’utilise un issuer_scope de guichet déclaratif.

Le texte précise le parcours : task_status avec un task_id connu ; sinon rejeu
natif strict uniquement si le mandat reste valide, car un appel jamais reçu peut
alors être admis. Pour un contrôle sans effet ou un brief périmé, faire identifier
le reçu par un responsable autorisé sans rejouer. Ne pas changer de clé après
une issue inconnue. Le catalogue et la référence native expliquent cette portée.

La remise feea6010 a une entrée dispatching côté maître, la même instance
récipiendaire que la tâche fc06d175 et une ligne de spool SQLite côté wrapper.
Elle est conservée et différée pendant le tour courant. Le journal montre le
même message de mission encore actif. Aucun ACK ni lecture n’en est déduit.
Le code ne perd pas ce corps : il attend de pouvoir l’injecter. Aucune mission
ni remise métier n’est rejouée pour ce contrôle.

Les deux tests276 échouent sur le runtime274, avec applied:false au lieu de null,
puis passent après correction. La table couvre sept états sans payload et garde
les quatre refus de terminal sans résultat. Le test Unix exerce le maître ancien
avec idempotency_expired, le scope exact et l’absence de toute commande native
ou mutation supplémentaire. Les tests voisins sont réutilisés et exécutés.

Limite d’activation : les anciennes façades MCP266 déjà montées dans T3 gardent
leur code de projection. Le choix explicite de garder T3 ouvert est préservé.
Le correctif bénéficie aux façades chargées depuis la nouvelle image ; l’ancien
format encore chargé n’est pas présenté comme corrigé. Aucune preuve de session
n’est copiée pour essayer de monter une autre façade depuis un shell.

La revue R1 demande quatre précisions de guidage. La coupure après dépôt
réutilise désormais le constructeur commun de réponse ; le détail transport
reste séparé. Chaque code connu a une prochaine action spécifique, puis les
conditions où demander la décision du propriétaire. Le conseil natif conditionnel
reste réservé à la clé absente/expirée et exige d’abord d’identifier l’outil
d’origine. Les états terminaux techniques sont présentés sans preuve d’effet
métier ; une issue future non reconnue n’atteste pas non plus la fin. La valeur
future_issue du test est synthétique. Les tests de mutation vérifient scope,
absence de lecture native et guidage, sur file d’attente et perte après dépôt.

Dans la fenêtre 20:27–20:37, cinq remises de ce groupe sont retrouvées accusées,
dont une émission à 20:31:43. Aucun corps perdu n’est établi pour cette fenêtre.
La référence du cas vers 22:40 manque : les faits fournis ne permettent pas de
l’identifier avec certitude ; ne pas lui attribuer la cause de feea6010.

Qualification après R1 : les trois tests276 (dont les booléens positifs/négatifs
attestés), les deux tests de mutation modifiés et Clippy strict workspace/all-targets
passent sur Mac et Debian 12. Les trois autres tests voisins avaient passé avant
ces ajustements et leurs chemins ne changent pas. Huit tests distincts sont donc
couverts sur chaque plateforme. Les deux builds release réussissent ; Linux
exige glibc 2.34, sous le plafond 2.36. La recette cumulative est vérifiée sans
fuzz ; ses cinq fichiers du delta correspondent à l’export compilé. La revue
R2 porte sur les conditions de guidage R1 et conserve son identifiant natif.

R2 confirme les corrections des points 1, 2 et 4. Sa dernière remarque sur la
description guichet est traitée en retirant son conseil de rejeu natif ; seul
le périmètre de lecture et l’absence de preuve restent dans cette description.
Le candidat non activé r1 est remplacé par r2, sans réutiliser son répertoire
exécutable immuable. Le reste de la logique et les garanties wire ne changent pas.
