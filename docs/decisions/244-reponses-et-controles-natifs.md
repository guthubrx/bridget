#244 — Réponses ordinaires et contrôles natifs

Date :8novembre2026. Statut : retenu, activation en cours.

Une mission distante et un message ordinaire adressé à son enfant sont deux
échanges distincts. Le résultat de mission garde son emplacement réservé et
ses contrôles worker. Le message ordinaire garde son identité, sa corrélation,
les gardes de communication et la durée de vie déjà accordée à cet enfant.
Il ne prolonge ni sa garde ni ses droits.

Le wrapper conserve sa réponse ordinaire avant l’envoi dans une boîte SQLite
privée, distincte des reçus d’injection. La clé est dérivée de l’instance,
de l’identifiant de la demande et de son expéditeur. Le maître réutilise son
store d’idempotence, sous une portée interne dérivée de la connexion primaire
vivante. Il vérifie la demande suivie et ses deux extrémités. Aucun faux
client négocié, droit humain ou fait de permission n’est créé. Les missions
courantes et anciennes ne peuvent pas passer par cette entrée.

L’ACK du maître signifie dépôt durable de la réponse. L’ACK du destinataire
reste distinct. Une coupure avant le premier ACK garde la boîte ; la reprise
conserve clé et enveloppe. Un refus nommé bloque cette réponse en conservant
sa preuve. Une réponse dépassant l’horizon de7jours ne repart plus. Les
anciens maîtres n’annoncent pas native_reply_outbox_v1 : aucun rejeu de boîte
ne leur est envoyé. Le retour doit être qualifié sur ces fichiers et sur les
files d’entrée244 ; un ancien binaire243 seul ne suffit pas à promettre la
reprise des nouvelles files.

Les contrôles quotidiens sont des agents natifs sans client externe. Leurs
clés et identités sont écrites avant chaque ordre. Une issue inconnue reste
attachée à cet ordre. Les résultats, lectures, gardes, fils et réponses sont
vérifiés dans les stores natifs, en lecture seule. Les agents discovery ne
reçoivent aucun droit d’écrire ou de communiquer qui leur était interdit.
Le parent détaché crée le fil ; son enfant lit et confirme sa page.

La réparation de service ne vise qu’une unité Bridget déclarée dans la
configuration, déjà arrêtée en échec et sans PID. Aucun travail fournisseur
n’est tué pour ce contrôle. Les permissions et les comptes ne sont jamais
réparés par élargissement automatique. Le client reste ouvert pendant cette
livraison, conformément à la demande de François.
