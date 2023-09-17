#!/bin/bash

cleanup() {
  echo 'Shutting down...'
  docker stop nextcloud-aio-mastercontainer && \
  docker rm nextcloud-aio-mastercontainer && \
  docker volume rm nextcloud_aio_mastercontainer
}
trap cleanup EXIT

docker run \
--init \
--sig-proxy=false \
--name nextcloud-aio-mastercontainer \
--volume nextcloud_aio_mastercontainer:/mnt/docker-aio-config \
--volume /var/run/docker.sock:/var/run/docker.sock:ro \
--volume "${PWD}/ncp.php:/var/www/docker-aio/php/public/ncp.php" \
nextcloud/all-in-one:latest

