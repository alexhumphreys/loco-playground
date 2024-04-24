default: docker-compose-loco-restart

install-cargo-tools:
  cargo install loco-cli
  cargo install sea-orm-cli

docker_compose_files_arg := "-f ./docker-compose/docker-compose.loco-base.yaml -f ./docker-compose/zitadel.docker-compose.yaml"

docker-compose-loco-build:
	docker compose {{docker_compose_files_arg}} build

docker-compose-loco-up:
	docker compose {{docker_compose_files_arg}} up

docker-compose-loco-down:
	docker compose {{docker_compose_files_arg}} down

docker-compose-loco-restart: docker-compose-loco-down docker-compose-loco-build docker-compose-loco-up

