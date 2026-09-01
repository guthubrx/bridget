#!/bin/sh
set -eu

# Le conteneur est l'environnement persistant du projet. Les agents y entrent
# ensuite via `docker exec`; il ne doit donc pas s'arrêter en l'absence de
# commande interactive. Aucun shell d'appelant ni secret n'est interprété.
trap 'exit 0' INT TERM
while :; do
  sleep 86400 &
  wait "$!"
done
