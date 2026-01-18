/**
 * Icon mapping utilities for ZManager
 *
 * Maps file extensions and types to their corresponding sprite symbol names.
 * Uses vite-plugin-svg-icons for sprite generation.
 * Priority: dev_icons > filetypes > ui (fallback)
 *
 * Symbol ID format: icon-{dir}-{name} (without .svg extension)
 * - dev_icons: icon-{folder}-{filename} e.g., "icon-typescript-typescript-original"
 * - filetypes: icon-{filename} e.g., "icon-document"
 * - ui: icon-{filename} e.g., "icon-ic_copy"
 */

import type { EntryMeta } from "../types";

/**
 * Dev icons mapping: extension/name → symbol name
 * Format: extension -> "folder-filename" (will become "icon-folder-filename")
 */
const DEV_ICON_MAP: Record<string, string> = {
  // TypeScript
  ts: "typescript-typescript-original",
  tsx: "typescript-typescript-original",
  // JavaScript
  js: "javascript-javascript-original",
  mjs: "javascript-javascript-original",
  cjs: "javascript-javascript-original",
  jsx: "react-react-original",
  // React
  // Rust
  rs: "rust-rust-original",
  // Python
  py: "python-python-original",
  pyw: "python-python-original",
  pyi: "python-python-original",
  // Java
  java: "java-java-original",
  jar: "java-java-original",
  // C/C++
  c: "c-c-original",
  h: "c-c-original",
  cpp: "cplusplus-cplusplus-original",
  cxx: "cplusplus-cplusplus-original",
  cc: "cplusplus-cplusplus-original",
  hpp: "cplusplus-cplusplus-original",
  hxx: "cplusplus-cplusplus-original",
  // C#
  cs: "csharp-csharp-original",
  csx: "csharp-csharp-original",
  // Go
  go: "go-go-original",
  mod: "go-go-original",
  // Ruby
  rb: "ruby-ruby-original",
  erb: "ruby-ruby-original",
  gemspec: "ruby-ruby-original",
  // PHP
  php: "php-php-original",
  phtml: "php-php-original",
  // Swift
  swift: "swift-swift-original",
  // Kotlin
  kt: "kotlin-kotlin-original",
  kts: "kotlin-kotlin-original",
  // Scala
  scala: "scala-scala-original",
  sc: "scala-scala-original",
  // Dart
  dart: "dart-dart-original",
  // Lua
  lua: "lua-lua-original",
  // Perl
  pl: "perl-perl-original",
  pm: "perl-perl-original",
  // Haskell
  hs: "haskell-haskell-original",
  lhs: "haskell-haskell-original",
  // Elixir
  ex: "elixir-elixir-original",
  exs: "elixir-elixir-original",
  // Erlang
  erl: "erlang-erlang-original",
  hrl: "erlang-erlang-original",
  // Clojure
  clj: "clojure-clojure-original",
  cljs: "clojure-clojure-original",
  cljc: "clojure-clojure-original",
  // F#
  fs: "fsharp-fsharp-original",
  fsx: "fsharp-fsharp-original",
  fsi: "fsharp-fsharp-original",
  // R
  r: "r-r-original",
  rmd: "r-r-original",
  // Julia
  jl: "julia-julia-original",
  // Nim
  nim: "nim-nim-original",
  // Zig
  zig: "zig-zig-original",
  // OCaml
  ml: "ocaml-ocaml-original",
  mli: "ocaml-ocaml-original",
  // HTML
  html: "html5-html5-original",
  htm: "html5-html5-original",
  xhtml: "html5-html5-original",
  // CSS
  css: "css3-css3-original",
  // Sass/SCSS
  scss: "sass-sass-original",
  sass: "sass-sass-original",
  // Less
  less: "less-less-plain-wordmark",
  // JSON
  json: "json-json-original",
  jsonc: "json-json-original",
  json5: "json-json-original",
  // YAML
  yaml: "yaml-yaml-original",
  yml: "yaml-yaml-original",
  // XML
  xml: "xml-xml-original",
  xsl: "xml-xml-original",
  xslt: "xml-xml-original",
  // Markdown
  md: "markdown-markdown-original",
  mdx: "markdown-markdown-original",
  markdown: "markdown-markdown-original",
  // SQL
  sql: "azuresqldatabase-azuresqldatabase-original",
  // Shell
  sh: "bash-bash-original",
  bash: "bash-bash-original",
  zsh: "bash-bash-original",
  fish: "bash-bash-original",
  // PowerShell
  ps1: "powershell-powershell-original",
  psm1: "powershell-powershell-original",
  psd1: "powershell-powershell-original",
  // Docker
  dockerfile: "docker-docker-original",
  // Git
  gitignore: "git-git-original",
  gitattributes: "git-git-original",
  gitmodules: "git-git-original",
  // GraphQL
  graphql: "graphql-graphql-plain",
  gql: "graphql-graphql-plain",
  // Vue
  vue: "vuejs-vuejs-original",
  // Svelte
  svelte: "svelte-svelte-original",
  // Angular
  angular: "angular-angular-original",
  // Terraform
  tf: "terraform-terraform-original",
  tfvars: "terraform-terraform-original",
  // Kubernetes
  // Gradle
  gradle: "gradle-gradle-original",
  // Maven
  pom: "maven-maven-original",
  // NPM
  // Yarn
  // Webpack
  // Vite
  // ESLint
  eslintrc: "eslint-eslint-original",
  // Jest
  // Mocha
  // Cypress
  // Storybook
  // Redux
  // MobX
  // Prisma
  prisma: "prisma-prisma-original",
  // Nginx
  nginx: "nginx-nginx-original",
  // Apache
  htaccess: "apache-apache-original",
  // Vim
  vim: "vim-vim-original",
  vimrc: "vim-vim-original",
  // Neovim
  nvim: "neovim-neovim-original",
  // VS Code
  code: "vscode-vscode-original",
  // Unity
  unity: "unity-unity-original",
  // Unreal
  // Blender
  blend: "blender-blender-original",
  // Godot
  gd: "godot-godot-original",
  godot: "godot-godot-original",
  // Arduino
  ino: "arduino-arduino-original",
  // LaTeX
  tex: "latex-latex-original",
  latex: "latex-latex-original",
  bib: "latex-latex-original",
};

/**
 * Special filenames that map to dev icons (symbol names)
 */
const SPECIAL_FILENAME_MAP: Record<string, string> = {
  // Package managers
  "package.json": "nodejs-nodejs-original",
  "package-lock.json": "npm-npm-original-wordmark",
  "yarn.lock": "yarn-yarn-original",
  "pnpm-lock.yaml": "pnpm-pnpm-original",
  "bun.lockb": "bun-bun-original",
  "bunfig.toml": "bun-bun-original",
  "deno.json": "denojs-denojs-original",
  "deno.jsonc": "denojs-denojs-original",
  "deno.lock": "denojs-denojs-original",
  // Config files
  "tsconfig.json": "typescript-typescript-original",
  "jsconfig.json": "javascript-javascript-original",
  ".eslintrc": "eslint-eslint-original",
  ".eslintrc.json": "eslint-eslint-original",
  ".eslintrc.js": "eslint-eslint-original",
  "eslint.config.js": "eslint-eslint-original",
  "eslint.config.mjs": "eslint-eslint-original",
  ".babelrc": "babel-babel-original",
  "babel.config.js": "babel-babel-original",
  "webpack.config.js": "webpack-webpack-original",
  "webpack.config.ts": "webpack-webpack-original",
  "vite.config.js": "vitejs-vitejs-original",
  "vite.config.ts": "vitejs-vitejs-original",
  "rollup.config.js": "rollup-rollup-original",
  "rollup.config.ts": "rollup-rollup-original",
  "tailwind.config.js": "tailwindcss-tailwindcss-original",
  "tailwind.config.ts": "tailwindcss-tailwindcss-original",
  "postcss.config.js": "postcss-postcss-original",
  "next.config.js": "nextjs-nextjs-original",
  "next.config.mjs": "nextjs-nextjs-original",
  "nuxt.config.js": "nuxtjs-nuxtjs-original",
  "nuxt.config.ts": "nuxtjs-nuxtjs-original",
  "svelte.config.js": "svelte-svelte-original",
  "astro.config.mjs": "astro-astro-original",
  "remix.config.js": "remix-remix-original",
  "angular.json": "angular-angular-original",
  "nest-cli.json": "nestjs-nestjs-original",
  // Docker
  Dockerfile: "docker-docker-original",
  "docker-compose.yml": "docker-docker-original",
  "docker-compose.yaml": "docker-docker-original",
  ".dockerignore": "docker-docker-original",
  // Git
  ".gitignore": "git-git-original",
  ".gitattributes": "git-git-original",
  ".gitmodules": "git-git-original",
  // CI/CD
  ".travis.yml": "travis-travis-original",
  ".gitlab-ci.yml": "gitlab-gitlab-original",
  Jenkinsfile: "jenkins-jenkins-original",
  "azure-pipelines.yml": "azure-azure-original",
  // Rust
  "Cargo.toml": "rust-rust-original",
  "Cargo.lock": "rust-rust-original",
  // Python
  "requirements.txt": "python-python-original",
  Pipfile: "python-python-original",
  "Pipfile.lock": "python-python-original",
  "pyproject.toml": "python-python-original",
  "setup.py": "python-python-original",
  // Ruby
  Gemfile: "ruby-ruby-original",
  "Gemfile.lock": "ruby-ruby-original",
  Rakefile: "ruby-ruby-original",
  // Go
  "go.mod": "go-go-original",
  "go.sum": "go-go-original",
  // Java/Gradle/Maven
  "build.gradle": "gradle-gradle-original",
  "build.gradle.kts": "gradle-gradle-original",
  "settings.gradle": "gradle-gradle-original",
  "settings.gradle.kts": "gradle-gradle-original",
  "pom.xml": "maven-maven-original",
  // .NET
  ".csproj": "csharp-csharp-original",
  ".sln": "visualstudio-visualstudio-original",
  "nuget.config": "nuget-nuget-original",
  // README/Docs
  "README.md": "markdown-markdown-original",
  "CHANGELOG.md": "markdown-markdown-original",
  "CONTRIBUTING.md": "markdown-markdown-original",
  LICENSE: "document",
  "LICENSE.md": "document",
  // Tauri
  "tauri.conf.json": "tauri-tauri-original",
  // Biome
  "biome.json": "biome-biome-original",
  "biome.jsonc": "biome-biome-original",
  // Vitest
  "vitest.config.ts": "vitest-vitest-original",
  "vitest.config.js": "vitest-vitest-original",
  // Jest
  "jest.config.js": "jest-jest-plain",
  "jest.config.ts": "jest-jest-plain",
  // Playwright
  "playwright.config.ts": "playwright-playwright-original",
  "playwright.config.js": "playwright-playwright-original",
};

/**
 * Filetypes icons mapping for common file types (symbol names without icon- prefix)
 * Falls back to these when dev_icons don't have a match
 */
const FILETYPE_ICON_MAP: Record<string, string> = {
  // Documents
  pdf: "pdf",
  doc: "document",
  docx: "document",
  odt: "document",
  rtf: "document",
  txt: "document",
  // Spreadsheets
  xls: "table",
  xlsx: "table",
  csv: "table",
  ods: "table",
  // Presentations
  ppt: "powerpoint",
  pptx: "powerpoint",
  odp: "powerpoint",
  // Images
  png: "image",
  jpg: "image",
  jpeg: "image",
  gif: "image",
  bmp: "image",
  webp: "image",
  svg: "svg",
  ico: "image",
  tiff: "image",
  tif: "image",
  psd: "adobe-photoshop",
  ai: "adobe-illustrator",
  // Audio
  mp3: "audio",
  wav: "audio",
  flac: "audio",
  ogg: "audio",
  m4a: "audio",
  aac: "audio",
  wma: "audio",
  // Video
  mp4: "video",
  mkv: "video",
  avi: "video",
  mov: "video",
  wmv: "video",
  webm: "video",
  flv: "video",
  m4v: "video",
  // Archives
  zip: "zip",
  rar: "zip",
  "7z": "zip",
  tar: "zip",
  gz: "zip",
  bz2: "zip",
  xz: "zip",
  // Executables
  exe: "exe",
  msi: "exe",
  bat: "command",
  cmd: "command",
  com: "exe",
  // System
  dll: "dll",
  sys: "settings",
  ini: "settings",
  cfg: "settings",
  conf: "settings",
  // Fonts
  ttf: "font",
  otf: "font",
  woff: "font",
  woff2: "font",
  eot: "font",
  // Database
  db: "database",
  sqlite: "database",
  sqlite3: "database",
  // Certificates
  pem: "certificate",
  crt: "certificate",
  cer: "certificate",
  key: "key",
  // Logs
  log: "log",
  // Lock files
  lock: "lock",
  // Notebooks
  ipynb: "jupyter",
};

/**
 * Folder icon mapping based on folder name (symbol names without icon- prefix)
 */
const FOLDER_ICON_MAP: Record<string, string> = {
  // Common folders
  src: "folder-src",
  source: "folder-src",
  lib: "folder-lib",
  libs: "folder-lib",
  dist: "folder-dist",
  build: "folder-dist",
  out: "folder-dist",
  output: "folder-dist",
  bin: "folder-dist",
  node_modules: "folder-node",
  vendor: "folder-lib",
  packages: "folder-packages",
  // Config
  config: "folder-config",
  configs: "folder-config",
  configuration: "folder-config",
  settings: "folder-config",
  // Tests
  test: "folder-test",
  tests: "folder-test",
  __tests__: "folder-test",
  spec: "folder-test",
  specs: "folder-test",
  // Documentation
  docs: "folder-docs",
  doc: "folder-docs",
  documentation: "folder-docs",
  // Assets
  assets: "folder-images",
  images: "folder-images",
  img: "folder-images",
  icons: "folder-images",
  fonts: "folder-font",
  media: "folder-video",
  audio: "folder-audio",
  video: "folder-video",
  // Styles
  styles: "folder-css",
  css: "folder-css",
  scss: "folder-sass",
  sass: "folder-sass",
  less: "folder-less",
  // Scripts
  scripts: "folder-scripts",
  // Components
  components: "folder-components",
  component: "folder-components",
  // Pages/Views
  pages: "folder-views",
  views: "folder-views",
  screens: "folder-views",
  // API
  api: "folder-api",
  apis: "folder-api",
  // Services
  services: "folder-server",
  service: "folder-server",
  // Hooks
  hooks: "folder-hook",
  hook: "folder-hook",
  // Utils
  utils: "folder-utils",
  util: "folder-utils",
  utilities: "folder-utils",
  helpers: "folder-helper",
  helper: "folder-helper",
  // Types
  types: "folder-typescript",
  typings: "folder-typescript",
  interfaces: "folder-interface",
  // Models
  models: "folder-database",
  model: "folder-database",
  entities: "folder-database",
  schemas: "folder-database",
  // Controllers
  controllers: "folder-controller",
  controller: "folder-controller",
  // Middleware
  middleware: "folder-middleware",
  middlewares: "folder-middleware",
  // Routes
  routes: "folder-routes",
  router: "folder-routes",
  routing: "folder-routes",
  // State/Store
  store: "folder-store",
  stores: "folder-store",
  state: "folder-store",
  redux: "folder-redux-reducer",
  // Context
  context: "folder-context",
  contexts: "folder-context",
  providers: "folder-context",
  // Public
  public: "folder-public",
  static: "folder-public",
  // Private
  private: "folder-private",
  // Git
  ".git": "folder-git",
  // GitHub
  ".github": "folder-github",
  // VSCode
  ".vscode": "folder-vscode",
  // IDE
  ".idea": "folder-intellij",
  // CI/CD
  ".circleci": "folder-circleci",
  // Docker
  docker: "folder-docker",
  // Kubernetes
  kubernetes: "folder-kubernetes",
  k8s: "folder-kubernetes",
  // Terraform
  terraform: "folder-terraform",
  // Ansible
  ansible: "folder-ansible",
  // AWS
  aws: "folder-aws",
  // Azure
  azure: "folder-azure-pipelines",
  // Logs
  logs: "folder-log",
  log: "folder-log",
  // Temp
  tmp: "folder-temp",
  temp: "folder-temp",
  cache: "folder-temp",
  ".cache": "folder-temp",
  // Backup
  backup: "folder-backup",
  backups: "folder-backup",
  // Archive
  archive: "folder-archive",
  archives: "folder-archive",
  // Download
  downloads: "folder-download",
  download: "folder-download",
  // Upload
  uploads: "folder-upload",
  upload: "folder-upload",
  // Localization
  i18n: "folder-i18n",
  l10n: "folder-i18n",
  locales: "folder-i18n",
  locale: "folder-i18n",
  translations: "folder-i18n",
  // Templates
  templates: "folder-template",
  template: "folder-template",
  // Layouts
  layouts: "folder-layout",
  layout: "folder-layout",
  // Plugins
  plugins: "folder-plugin",
  plugin: "folder-plugin",
  extensions: "folder-plugin",
  addons: "folder-plugin",
  // Modules
  modules: "folder-lib",
  module: "folder-lib",
  // Examples
  examples: "folder-examples",
  example: "folder-examples",
  samples: "folder-examples",
  sample: "folder-examples",
  demo: "folder-examples",
  demos: "folder-examples",
  // Fixtures
  fixtures: "folder-mock",
  fixture: "folder-mock",
  mocks: "folder-mock",
  mock: "folder-mock",
  __mocks__: "folder-mock",
  // Snapshots
  __snapshots__: "folder-test",
  snapshots: "folder-test",
  // Storybook
  ".storybook": "folder-storybook",
  stories: "folder-storybook",
  // Prisma
  prisma: "folder-prisma",
  // Migrations
  migrations: "folder-migrations",
  migration: "folder-migrations",
  // Seeds
  seeds: "folder-seeders",
  seeders: "folder-seeders",
  // Husky
  ".husky": "folder-husky",
  // Tauri
  "src-tauri": "folder-src-tauri",
  // Next.js
  ".next": "folder-next",
  // Nuxt
  ".nuxt": "folder-nuxt",
  // Svelte
  ".svelte-kit": "folder-svelte",
  // Android
  android: "folder-android",
  // iOS
  ios: "folder-ios",
  // React Native
  // Flutter
  flutter: "folder-flutter",
  // Supabase
  supabase: "folder-supabase",
  // Vercel
  ".vercel": "folder-vercel",
  // Netlify
  ".netlify": "folder-netlify",
  // Drizzle
  drizzle: "folder-drizzle",
};

export interface IconInfo {
  /** Symbol name for the sprite (use with SvgIcon name prop) */
  symbolName: string;
  /** Icon category: 'dev' | 'filetype' | 'ui' */
  category: "dev" | "filetype" | "ui";
  /** Whether this is a folder icon */
  isFolder: boolean;
  /** Whether this folder should show as open */
  isOpen?: boolean;
}

/**
 * Get the icon info for a file entry
 */
export function getIconForEntry(entry: EntryMeta, isOpen = false): IconInfo {
  const name = entry.name;
  const ext = entry.extension?.toLowerCase() ?? "";
  const nameLower = name.toLowerCase();

  // Folders
  if (entry.kind === "directory") {
    return getFolderIcon(nameLower, isOpen);
  }

  // Symlinks and Junctions use special UI icons
  if (entry.kind === "symlink") {
    return {
      symbolName: "ic_link",
      category: "ui",
      isFolder: false,
    };
  }

  if (entry.kind === "junction") {
    return {
      symbolName: "ic_folder_link",
      category: "ui",
      isFolder: false,
    };
  }

  // Check special filenames first (highest priority for files)
  if (SPECIAL_FILENAME_MAP[name]) {
    return {
      symbolName: SPECIAL_FILENAME_MAP[name],
      category: "dev",
      isFolder: false,
    };
  }

  // Check case-insensitive filename match
  const lowerName = name.toLowerCase();
  for (const [key, value] of Object.entries(SPECIAL_FILENAME_MAP)) {
    if (key.toLowerCase() === lowerName) {
      return {
        symbolName: value,
        category: "dev",
        isFolder: false,
      };
    }
  }

  // Check dev icons by extension
  if (ext && DEV_ICON_MAP[ext]) {
    return {
      symbolName: DEV_ICON_MAP[ext],
      category: "dev",
      isFolder: false,
    };
  }

  // Check filetypes icons
  if (ext && FILETYPE_ICON_MAP[ext]) {
    return {
      symbolName: FILETYPE_ICON_MAP[ext],
      category: "filetype",
      isFolder: false,
    };
  }

  // Default file icon
  return {
    symbolName: "document",
    category: "filetype",
    isFolder: false,
  };
}

/**
 * Get folder icon based on folder name
 */
function getFolderIcon(folderName: string, isOpen: boolean): IconInfo {
  const iconName = FOLDER_ICON_MAP[folderName];

  if (iconName) {
    // Use open variant if available
    const openIconName = isOpen ? `${iconName}-open` : iconName;
    return {
      symbolName: openIconName,
      category: "filetype",
      isFolder: true,
      isOpen,
    };
  }

  // Default folder icon
  return {
    symbolName: isOpen ? "folder-base-open" : "folder-base",
    category: "filetype",
    isFolder: true,
    isOpen,
  };
}

/**
 * Get a drive icon symbol name
 */
export function getDriveIconName(driveType: string): string {
  switch (driveType) {
    case "Removable":
      return "ic_usb_stick";
    case "Network":
      return "ic_globe";
    case "CdRom":
      return "ic_disc";
    case "RamDisk":
      return "ic_storage";
    default:
      return "ic_hard_drive";
  }
}

/**
 * Get UI icon symbol name
 */
export function getUiIconName(iconName: string): string {
  return `ic_${iconName}`;
}
