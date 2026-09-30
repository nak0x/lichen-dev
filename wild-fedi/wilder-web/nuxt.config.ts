// Client web du serveur ActivityPub « wilder ». Même parti pris que le site
// LICHEN : pas de module UI, pas de police distante, pas d'analytics. La page
// reste légère parce que c'est une valeur du projet, pas un détail.
//
// Le navigateur ne parle jamais directement au serveur ActivityPub : tout passe
// par les routes Nitro (`server/api/*`). Cela évite CORS et laisse le serveur
// envoyer le bon en-tête `Accept: application/activity+json`, sans lequel la
// négociation de contenu échoue côté fédiverse.
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: false },
  telemetry: false,

  css: ['~/assets/css/main.css'],

  runtimeConfig: {
    // Base du serveur wilder (côté serveur uniquement). Surchargeable via WILDER_URL.
    wilderUrl: process.env.WILDER_URL || 'http://127.0.0.1:8080',
    public: {
      // L'acteur « d'accueil » affiché par défaut, au format @nom@domaine.
      homeHandle: process.env.NUXT_PUBLIC_HOME_HANDLE || 'beaver-class@127.0.0.1:8080',
      instanceName: process.env.NUXT_PUBLIC_INSTANCE_NAME || 'wilder'
    }
  },

  app: {
    head: {
      htmlAttrs: { lang: 'fr' },
      meta: [
        { charset: 'utf-8' },
        { name: 'viewport', content: 'width=device-width, initial-scale=1' },
        { name: 'color-scheme', content: 'light dark' },
        { name: 'robots', content: 'noindex' }, // un client de démonstration ne s'indexe pas
        {
          name: 'description',
          content:
            "Client fédéré pour le réseau LICHEN : parcourir les observations naturalistes partagées entre écoles via ActivityPub."
        }
      ],
      link: [{ rel: 'icon', href: '/favicon.svg', type: 'image/svg+xml' }]
    }
  },

  features: {
    // Le CSS est injecté dans le HTML : une requête de moins.
    inlineStyles: true
  },

  experimental: {
    payloadExtraction: false
  }
})
