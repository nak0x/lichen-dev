# Wild-Fedi — orchestration de la pile fédérée (serveur wilder + client web).
# Usage :
#   make stack       lance toute la pile (back + front)
#   make services    lance uniquement le serveur ActivityPub
#   make ui          lance uniquement le client web
#   make down        arrête tout ; make logs suit les journaux

COMPOSE ?= docker compose

.DEFAULT_GOAL := help
.PHONY: stack services ui build down logs ps restart clean help

stack: ## Lance toute la pile (serveur + client)
	$(COMPOSE) up -d --build
	@echo ""
	@echo "  Serveur ActivityPub : http://localhost:8080"
	@echo "  Client web          : http://localhost:3000"

services: ## Lance uniquement le back (serveur ActivityPub wilder)
	$(COMPOSE) up -d --build wilder
	@echo ""
	@echo "  Serveur ActivityPub : http://localhost:8080"

ui: ## Lance uniquement le front (client web, sans ses dépendances)
	$(COMPOSE) up -d --build --no-deps wilder-web
	@echo ""
	@echo "  Client web : http://localhost:3000"
	@echo "  (les données n'apparaîtront qu'avec le back : make services)"

build: ## Construit les images sans démarrer les conteneurs
	$(COMPOSE) build

down: ## Arrête et supprime les conteneurs de la pile
	$(COMPOSE) down

restart: ## Redémarre la pile
	$(COMPOSE) restart

logs: ## Suit les journaux de la pile
	$(COMPOSE) logs -f

ps: ## Affiche l'état des conteneurs
	$(COMPOSE) ps

clean: ## Arrête tout et supprime images locales, volumes et orphelins
	$(COMPOSE) down --rmi local --volumes --remove-orphans

help: ## Affiche cette aide
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN{FS=":.*?## "}{printf "  \033[36m%-10s\033[0m %s\n", $$1, $$2}'
