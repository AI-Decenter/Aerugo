/// <reference types="vite/client" />

interface ImportMetaEnv {
    readonly VITE_API_BASE_URL: string
    readonly VITE_DOMAIN: string
    readonly VITE_GEMINI_API_KEY: string
    readonly VITE_ALLOWED_HOSTS: string
}

interface ImportMeta {
    readonly env: ImportMetaEnv
}