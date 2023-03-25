
PYTHON=python3

down:
	docker-compose down
stop:
	make down
up:
	docker-compose up -d
restart:
	make down && make up
start:
	make start
enter:
	docker exec -it subspace bash
build:
	cargo build --release

dev:
	./target/release/node-subspace  --dev