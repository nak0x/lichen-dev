# wilder-web — client fédéré des observations LICHEN

Un client web (Nuxt 4) pour le serveur ActivityPub [`wilder`](../wilder). Il lit et
affiche, via **ActivityPub**, les observations naturalistes qu'une école publie sur
le réseau — et montre chaque objet aussi bien *lisible* que *brut*.

Même parti pris que le [site LICHEN](../../lichen-website) : pas de module UI, pas de
police distante, pas de traceur, page légère. Le fond animé est dessiné dans le
navigateur, la carte est schématique — **aucune requête vers un tiers**.

## Ce qu'il fait

| Page | Rôle |
|------|------|
| `/` | Le fil : les observations de l'acteur d'accueil (son *outbox*), avec le JSON brut sous chaque carte |
| `/acteur?h=@nom@domaine` | Fiche d'un acteur (local **ou distant**) : profil, observations, abonnés — résolu par WebFinger |
| `/carte` | Les relevés géolocalisés, projetés dans un SVG (champ `location` → `Place`) |
| `/instance` | Le NodeInfo de l'instance : logiciel, protocoles, nombre d'acteurs |
| `/a-propos` | Le protocole, et comment ce client est branché |

## Architecture

Le navigateur ne parle **jamais** directement au serveur ActivityPub. Chaque requête
passe par les routes Nitro (`server/api/*`), qui interrogent l'instance côté serveur :

- pas de blocage **CORS** (le serveur fédéré n'a pas à autoriser d'origine web) ;
- l'en-tête `Accept: application/activity+json` est envoyé côté serveur, sans quoi la
  négociation de contenu échoue.

```
navigateur ──▶ /api/actor, /api/collection, /api/nodeinfo (Nitro) ──▶ serveur wilder
```

## Démarrer

Prérequis : le serveur `wilder` doit tourner (voir [`../wilder`](../wilder) ; par défaut
sur `http://127.0.0.1:8080`).

```bash
npm install
npm run dev              # http://localhost:3000
# ou, en production :
npm run build && npm run start
```

## Configuration

Copiez `.env.example` en `.env` :

| Variable | Défaut | Rôle |
|----------|--------|------|
| `WILDER_URL` | `http://127.0.0.1:8080` | base du serveur ActivityPub interrogé (côté serveur) |
| `NUXT_PUBLIC_HOME_HANDLE` | `beaver-class@127.0.0.1:8080` | acteur affiché par défaut sur `/` |
| `NUXT_PUBLIC_INSTANCE_NAME` | `wilder` | nom d'affichage de l'instance |

## Limites assumées

- **Lecture seule.** `wilder` n'expose pas d'API client-vers-serveur (C2S) ; ce client
  parcourt et affiche, il ne publie pas.
- Le contenu des notes distantes est réduit à du **texte** (pas de `v-html`) : pas de
  vecteur XSS, au prix du formatage riche.
- Le paramètre `url` des routes `/api/*` ne suit que http(s). En production, il faudrait
  une **liste blanche** de domaines fédérés (anti-SSRF).
- Les collections ne sont pas paginées au-delà de la première page.
