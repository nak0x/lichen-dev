# syntax=docker/dockerfile:1
# --- Étage de build (Nuxt/Nitro) --------------------------------------------
FROM node:22-slim AS builder
WORKDIR /app
# On copie les sources AVANT d'installer : le script postinstall du projet
# (`nuxt prepare`) en a besoin. On utilise `npm install` plutôt que `npm ci` :
# il réconcilie le lock quel que soit le npm de base, sans en exiger la synchro
# stricte. Le cache /root/.npm garde les paquets téléchargés entre deux builds.
COPY . .
RUN --mount=type=cache,target=/root/.npm npm install --no-audit --no-fund
RUN npm run build

# --- Image d'exécution (serveur Node autonome) ------------------------------
FROM node:22-slim AS runtime
WORKDIR /app
ENV NODE_ENV=production \
    HOST=0.0.0.0 \
    PORT=3000
# Nitro produit un serveur autonome : seul .output est nécessaire à l'exécution.
COPY --from=builder /app/.output ./.output
EXPOSE 3000
CMD ["node", ".output/server/index.mjs"]
