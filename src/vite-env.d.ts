/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_GIT_COMMIT_HASH: string;
  // more env variables...
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}