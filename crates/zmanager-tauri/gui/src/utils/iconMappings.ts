/**
 * Icon mapping utilities for ZManager
 *
 * Maps file extensions and types to their corresponding icon paths.
 * Priority: dev_icons > filetypes > ui (fallback)
 */

import type { EntryMeta } from "../types";

// Base paths for icon categories
const ICONS_BASE = "/icons";
const DEV_ICONS = `${ICONS_BASE}/dev_icons`;
const FILETYPES = `${ICONS_BASE}/filetypes`;
const UI = `${ICONS_BASE}/ui`;

/**
 * Dev icons mapping: extension/name → dev_icon folder/file
 * These are high-quality branded icons for developer tools and languages
 */
const DEV_ICON_MAP: Record<string, string> = {
  // TypeScript
  ts: "typescript/typescript-original.svg",
  tsx: "typescript/typescript-original.svg",
  // JavaScript
  js: "javascript/javascript-original.svg",
  mjs: "javascript/javascript-original.svg",
  cjs: "javascript/javascript-original.svg",
  jsx: "react/react-original.svg",
  // React
  // Rust
  rs: "rust/rust-original.svg",
  // Python
  py: "python/python-original.svg",
  pyw: "python/python-original.svg",
  pyi: "python/python-original.svg",
  // Java
  java: "java/java-original.svg",
  jar: "java/java-original.svg",
  // C/C++
  c: "c/c-original.svg",
  h: "c/c-original.svg",
  cpp: "cplusplus/cplusplus-original.svg",
  cxx: "cplusplus/cplusplus-original.svg",
  cc: "cplusplus/cplusplus-original.svg",
  hpp: "cplusplus/cplusplus-original.svg",
  hxx: "cplusplus/cplusplus-original.svg",
  // C#
  cs: "csharp/csharp-original.svg",
  csx: "csharp/csharp-original.svg",
  // Go
  go: "go/go-original.svg",
  mod: "go/go-original.svg",
  // Ruby
  rb: "ruby/ruby-original.svg",
  erb: "ruby/ruby-original.svg",
  gemspec: "ruby/ruby-original.svg",
  // PHP
  php: "php/php-original.svg",
  phtml: "php/php-original.svg",
  // Swift
  swift: "swift/swift-original.svg",
  // Kotlin
  kt: "kotlin/kotlin-original.svg",
  kts: "kotlin/kotlin-original.svg",
  // Scala
  scala: "scala/scala-original.svg",
  sc: "scala/scala-original.svg",
  // Dart
  dart: "dart/dart-original.svg",
  // Lua
  lua: "lua/lua-original.svg",
  // Perl
  pl: "perl/perl-original.svg",
  pm: "perl/perl-original.svg",
  // Haskell
  hs: "haskell/haskell-original.svg",
  lhs: "haskell/haskell-original.svg",
  // Elixir
  ex: "elixir/elixir-original.svg",
  exs: "elixir/elixir-original.svg",
  // Erlang
  erl: "erlang/erlang-original.svg",
  hrl: "erlang/erlang-original.svg",
  // Clojure
  clj: "clojure/clojure-original.svg",
  cljs: "clojure/clojure-original.svg",
  cljc: "clojure/clojure-original.svg",
  // F#
  fs: "fsharp/fsharp-original.svg",
  fsx: "fsharp/fsharp-original.svg",
  fsi: "fsharp/fsharp-original.svg",
  // R
  r: "r/r-original.svg",
  rmd: "r/r-original.svg",
  // Julia
  jl: "julia/julia-original.svg",
  // Nim
  nim: "nim/nim-original.svg",
  // Zig
  zig: "zig/zig-original.svg",
  // OCaml
  ml: "ocaml/ocaml-original.svg",
  mli: "ocaml/ocaml-original.svg",
  // HTML
  html: "html5/html5-original.svg",
  htm: "html5/html5-original.svg",
  xhtml: "html5/html5-original.svg",
  // CSS
  css: "css3/css3-original.svg",
  // Sass/SCSS
  scss: "sass/sass-original.svg",
  sass: "sass/sass-original.svg",
  // Less
  less: "less/less-plain-wordmark.svg",
  // JSON
  json: "json/json-original.svg",
  jsonc: "json/json-original.svg",
  json5: "json/json-original.svg",
  // YAML
  yaml: "yaml/yaml-original.svg",
  yml: "yaml/yaml-original.svg",
  // XML
  xml: "xml/xml-original.svg",
  xsl: "xml/xml-original.svg",
  xslt: "xml/xml-original.svg",
  // Markdown
  md: "markdown/markdown-original.svg",
  mdx: "markdown/markdown-original.svg",
  markdown: "markdown/markdown-original.svg",
  // SQL
  sql: "azuresqldatabase/azuresqldatabase-original.svg",
  // Shell
  sh: "bash/bash-original.svg",
  bash: "bash/bash-original.svg",
  zsh: "bash/bash-original.svg",
  fish: "bash/bash-original.svg",
  // PowerShell
  ps1: "powershell/powershell-original.svg",
  psm1: "powershell/powershell-original.svg",
  psd1: "powershell/powershell-original.svg",
  // Docker
  dockerfile: "docker/docker-original.svg",
  // Git
  gitignore: "git/git-original.svg",
  gitattributes: "git/git-original.svg",
  gitmodules: "git/git-original.svg",
  // GraphQL
  graphql: "graphql/graphql-plain.svg",
  gql: "graphql/graphql-plain.svg",
  // Vue
  vue: "vuejs/vuejs-original.svg",
  // Svelte
  svelte: "svelte/svelte-original.svg",
  // Angular
  angular: "angular/angular-original.svg",
  // Terraform
  tf: "terraform/terraform-original.svg",
  tfvars: "terraform/terraform-original.svg",
  // Kubernetes
  // Gradle
  gradle: "gradle/gradle-original.svg",
  // Maven
  pom: "maven/maven-original.svg",
  // NPM
  // Yarn
  // Webpack
  // Vite
  // ESLint
  eslintrc: "eslint/eslint-original.svg",
  // Jest
  // Mocha
  // Cypress
  // Storybook
  // Redux
  // MobX
  // Prisma
  prisma: "prisma/prisma-original.svg",
  // Nginx
  nginx: "nginx/nginx-original.svg",
  // Apache
  htaccess: "apache/apache-original.svg",
  // Vim
  vim: "vim/vim-original.svg",
  vimrc: "vim/vim-original.svg",
  // Neovim
  nvim: "neovim/neovim-original.svg",
  // VS Code
  code: "vscode/vscode-original.svg",
  // Unity
  unity: "unity/unity-original.svg",
  // Unreal
  // Blender
  blend: "blender/blender-original.svg",
  // Godot
  gd: "godot/godot-original.svg",
  godot: "godot/godot-original.svg",
  // Arduino
  ino: "arduino/arduino-original.svg",
  // LaTeX
  tex: "latex/latex-original.svg",
  latex: "latex/latex-original.svg",
  bib: "latex/latex-original.svg",
};

/**
 * Special filenames that map to dev icons
 */
const SPECIAL_FILENAME_MAP: Record<string, string> = {
  // Package managers
  "package.json": "nodejs/nodejs-original.svg",
  "package-lock.json": "npm/npm-original-wordmark.svg",
  "yarn.lock": "yarn/yarn-original.svg",
  "pnpm-lock.yaml": "pnpm/pnpm-original.svg",
  "bun.lockb": "bun/bun-original.svg",
  "bunfig.toml": "bun/bun-original.svg",
  "deno.json": "denojs/denojs-original.svg",
  "deno.jsonc": "denojs/denojs-original.svg",
  "deno.lock": "denojs/denojs-original.svg",
  // Config files
  "tsconfig.json": "typescript/typescript-original.svg",
  "jsconfig.json": "javascript/javascript-original.svg",
  ".eslintrc": "eslint/eslint-original.svg",
  ".eslintrc.json": "eslint/eslint-original.svg",
  ".eslintrc.js": "eslint/eslint-original.svg",
  "eslint.config.js": "eslint/eslint-original.svg",
  "eslint.config.mjs": "eslint/eslint-original.svg",
  ".babelrc": "babel/babel-original.svg",
  "babel.config.js": "babel/babel-original.svg",
  "webpack.config.js": "webpack/webpack-original.svg",
  "webpack.config.ts": "webpack/webpack-original.svg",
  "vite.config.js": "vitejs/vitejs-original.svg",
  "vite.config.ts": "vitejs/vitejs-original.svg",
  "rollup.config.js": "rollup/rollup-original.svg",
  "rollup.config.ts": "rollup/rollup-original.svg",
  "tailwind.config.js": "tailwindcss/tailwindcss-original.svg",
  "tailwind.config.ts": "tailwindcss/tailwindcss-original.svg",
  "postcss.config.js": "postcss/postcss-original.svg",
  "next.config.js": "nextjs/nextjs-original.svg",
  "next.config.mjs": "nextjs/nextjs-original.svg",
  "nuxt.config.js": "nuxtjs/nuxtjs-original.svg",
  "nuxt.config.ts": "nuxtjs/nuxtjs-original.svg",
  "svelte.config.js": "svelte/svelte-original.svg",
  "astro.config.mjs": "astro/astro-original.svg",
  "remix.config.js": "remix/remix-original.svg",
  "angular.json": "angular/angular-original.svg",
  "nest-cli.json": "nestjs/nestjs-original.svg",
  // Docker
  Dockerfile: "docker/docker-original.svg",
  "docker-compose.yml": "docker/docker-original.svg",
  "docker-compose.yaml": "docker/docker-original.svg",
  ".dockerignore": "docker/docker-original.svg",
  // Git
  ".gitignore": "git/git-original.svg",
  ".gitattributes": "git/git-original.svg",
  ".gitmodules": "git/git-original.svg",
  // CI/CD
  ".travis.yml": "travis/travis-original.svg",
  ".gitlab-ci.yml": "gitlab/gitlab-original.svg",
  Jenkinsfile: "jenkins/jenkins-original.svg",
  "azure-pipelines.yml": "azure/azure-original.svg",
  // Rust
  "Cargo.toml": "rust/rust-original.svg",
  "Cargo.lock": "rust/rust-original.svg",
  // Python
  "requirements.txt": "python/python-original.svg",
  Pipfile: "python/python-original.svg",
  "Pipfile.lock": "python/python-original.svg",
  "pyproject.toml": "python/python-original.svg",
  "setup.py": "python/python-original.svg",
  // Ruby
  Gemfile: "ruby/ruby-original.svg",
  "Gemfile.lock": "ruby/ruby-original.svg",
  Rakefile: "ruby/ruby-original.svg",
  // Go
  "go.mod": "go/go-original.svg",
  "go.sum": "go/go-original.svg",
  // Java/Gradle/Maven
  "build.gradle": "gradle/gradle-original.svg",
  "build.gradle.kts": "gradle/gradle-original.svg",
  "settings.gradle": "gradle/gradle-original.svg",
  "settings.gradle.kts": "gradle/gradle-original.svg",
  "pom.xml": "maven/maven-original.svg",
  // .NET
  ".csproj": "csharp/csharp-original.svg",
  ".sln": "visualstudio/visualstudio-original.svg",
  "nuget.config": "nuget/nuget-original.svg",
  // README/Docs
  "README.md": "markdown/markdown-original.svg",
  "CHANGELOG.md": "markdown/markdown-original.svg",
  "CONTRIBUTING.md": "markdown/markdown-original.svg",
  LICENSE: "document.svg",
  "LICENSE.md": "document.svg",
  // Tauri
  "tauri.conf.json": "tauri/tauri-original.svg",
  // Biome
  "biome.json": "biome/biome-original.svg",
  "biome.jsonc": "biome/biome-original.svg",
  // Vitest
  "vitest.config.ts": "vitest/vitest-original.svg",
  "vitest.config.js": "vitest/vitest-original.svg",
  // Jest
  "jest.config.js": "jest/jest-plain.svg",
  "jest.config.ts": "jest/jest-plain.svg",
  // Playwright
  "playwright.config.ts": "playwright/playwright-original.svg",
  "playwright.config.js": "playwright/playwright-original.svg",
};

/**
 * Filetypes icons mapping for common file types
 * Falls back to these when dev_icons don't have a match
 */
const FILETYPE_ICON_MAP: Record<string, string> = {
  // Documents
  pdf: "pdf.svg",
  doc: "document.svg",
  docx: "document.svg",
  odt: "document.svg",
  rtf: "document.svg",
  txt: "document.svg",
  // Spreadsheets
  xls: "table.svg",
  xlsx: "table.svg",
  csv: "table.svg",
  ods: "table.svg",
  // Presentations
  ppt: "powerpoint.svg",
  pptx: "powerpoint.svg",
  odp: "powerpoint.svg",
  // Images
  png: "image.svg",
  jpg: "image.svg",
  jpeg: "image.svg",
  gif: "image.svg",
  bmp: "image.svg",
  webp: "image.svg",
  svg: "svg.svg",
  ico: "image.svg",
  tiff: "image.svg",
  tif: "image.svg",
  psd: "adobe-photoshop.svg",
  ai: "adobe-illustrator.svg",
  // Audio
  mp3: "audio.svg",
  wav: "audio.svg",
  flac: "audio.svg",
  ogg: "audio.svg",
  m4a: "audio.svg",
  aac: "audio.svg",
  wma: "audio.svg",
  // Video
  mp4: "video.svg",
  mkv: "video.svg",
  avi: "video.svg",
  mov: "video.svg",
  wmv: "video.svg",
  webm: "video.svg",
  flv: "video.svg",
  m4v: "video.svg",
  // Archives
  zip: "folder-zip.svg",
  rar: "folder-zip.svg",
  "7z": "folder-zip.svg",
  tar: "folder-zip.svg",
  gz: "folder-zip.svg",
  bz2: "folder-zip.svg",
  xz: "folder-zip.svg",
  // Executables
  exe: "exe.svg",
  msi: "exe.svg",
  bat: "command.svg",
  cmd: "command.svg",
  com: "exe.svg",
  // System
  dll: "dll.svg",
  sys: "settings.svg",
  ini: "settings.svg",
  cfg: "settings.svg",
  conf: "settings.svg",
  // Fonts
  ttf: "font.svg",
  otf: "font.svg",
  woff: "font.svg",
  woff2: "font.svg",
  eot: "font.svg",
  // Database
  db: "database.svg",
  sqlite: "database.svg",
  sqlite3: "database.svg",
  // Certificates
  pem: "certificate.svg",
  crt: "certificate.svg",
  cer: "certificate.svg",
  key: "key.svg",
  // Logs
  log: "log.svg",
  // Lock files
  lock: "lock.svg",
  // Notebooks
  ipynb: "jupyter.svg",
};

/**
 * Folder icon mapping based on folder name
 */
const FOLDER_ICON_MAP: Record<string, string> = {
  // Common folders
  src: "folder-src.svg",
  source: "folder-src.svg",
  lib: "folder-lib.svg",
  libs: "folder-lib.svg",
  dist: "folder-dist.svg",
  build: "folder-dist.svg",
  out: "folder-dist.svg",
  output: "folder-dist.svg",
  bin: "folder-dist.svg",
  node_modules: "folder-node.svg",
  vendor: "folder-lib.svg",
  packages: "folder-packages.svg",
  // Config
  config: "folder-config.svg",
  configs: "folder-config.svg",
  configuration: "folder-config.svg",
  settings: "folder-config.svg",
  // Tests
  test: "folder-test.svg",
  tests: "folder-test.svg",
  __tests__: "folder-test.svg",
  spec: "folder-test.svg",
  specs: "folder-test.svg",
  // Documentation
  docs: "folder-docs.svg",
  doc: "folder-docs.svg",
  documentation: "folder-docs.svg",
  // Assets
  assets: "folder-images.svg",
  images: "folder-images.svg",
  img: "folder-images.svg",
  icons: "folder-images.svg",
  fonts: "folder-font.svg",
  media: "folder-video.svg",
  audio: "folder-audio.svg",
  video: "folder-video.svg",
  // Styles
  styles: "folder-css.svg",
  css: "folder-css.svg",
  scss: "folder-sass.svg",
  sass: "folder-sass.svg",
  less: "folder-less.svg",
  // Scripts
  scripts: "folder-scripts.svg",
  // Components
  components: "folder-components.svg",
  component: "folder-components.svg",
  // Pages/Views
  pages: "folder-views.svg",
  views: "folder-views.svg",
  screens: "folder-views.svg",
  // API
  api: "folder-api.svg",
  apis: "folder-api.svg",
  // Services
  services: "folder-server.svg",
  service: "folder-server.svg",
  // Hooks
  hooks: "folder-hook.svg",
  hook: "folder-hook.svg",
  // Utils
  utils: "folder-utils.svg",
  util: "folder-utils.svg",
  utilities: "folder-utils.svg",
  helpers: "folder-helper.svg",
  helper: "folder-helper.svg",
  // Types
  types: "folder-typescript.svg",
  typings: "folder-typescript.svg",
  interfaces: "folder-interface.svg",
  // Models
  models: "folder-database.svg",
  model: "folder-database.svg",
  entities: "folder-database.svg",
  schemas: "folder-database.svg",
  // Controllers
  controllers: "folder-controller.svg",
  controller: "folder-controller.svg",
  // Middleware
  middleware: "folder-middleware.svg",
  middlewares: "folder-middleware.svg",
  // Routes
  routes: "folder-routes.svg",
  router: "folder-routes.svg",
  routing: "folder-routes.svg",
  // State/Store
  store: "folder-store.svg",
  stores: "folder-store.svg",
  state: "folder-store.svg",
  redux: "folder-redux-reducer.svg",
  // Context
  context: "folder-context.svg",
  contexts: "folder-context.svg",
  providers: "folder-context.svg",
  // Public
  public: "folder-public.svg",
  static: "folder-public.svg",
  // Private
  private: "folder-private.svg",
  // Git
  ".git": "folder-git.svg",
  // GitHub
  ".github": "folder-github.svg",
  // VSCode
  ".vscode": "folder-vscode.svg",
  // IDE
  ".idea": "folder-intellij.svg",
  // CI/CD
  ".circleci": "folder-circleci.svg",
  // Docker
  docker: "folder-docker.svg",
  // Kubernetes
  kubernetes: "folder-kubernetes.svg",
  k8s: "folder-kubernetes.svg",
  // Terraform
  terraform: "folder-terraform.svg",
  // Ansible
  ansible: "folder-ansible.svg",
  // AWS
  aws: "folder-aws.svg",
  // Azure
  azure: "folder-azure-pipelines.svg",
  // Logs
  logs: "folder-log.svg",
  log: "folder-log.svg",
  // Temp
  tmp: "folder-temp.svg",
  temp: "folder-temp.svg",
  cache: "folder-temp.svg",
  ".cache": "folder-temp.svg",
  // Backup
  backup: "folder-backup.svg",
  backups: "folder-backup.svg",
  // Archive
  archive: "folder-archive.svg",
  archives: "folder-archive.svg",
  // Download
  downloads: "folder-download.svg",
  download: "folder-download.svg",
  // Upload
  uploads: "folder-upload.svg",
  upload: "folder-upload.svg",
  // Localization
  i18n: "folder-i18n.svg",
  l10n: "folder-i18n.svg",
  locales: "folder-i18n.svg",
  locale: "folder-i18n.svg",
  translations: "folder-i18n.svg",
  // Templates
  templates: "folder-template.svg",
  template: "folder-template.svg",
  // Layouts
  layouts: "folder-layout.svg",
  layout: "folder-layout.svg",
  // Plugins
  plugins: "folder-plugin.svg",
  plugin: "folder-plugin.svg",
  extensions: "folder-plugin.svg",
  addons: "folder-plugin.svg",
  // Modules
  modules: "folder-lib.svg",
  module: "folder-lib.svg",
  // Examples
  examples: "folder-examples.svg",
  example: "folder-examples.svg",
  samples: "folder-examples.svg",
  sample: "folder-examples.svg",
  demo: "folder-examples.svg",
  demos: "folder-examples.svg",
  // Fixtures
  fixtures: "folder-mock.svg",
  fixture: "folder-mock.svg",
  mocks: "folder-mock.svg",
  mock: "folder-mock.svg",
  __mocks__: "folder-mock.svg",
  // Snapshots
  __snapshots__: "folder-test.svg",
  snapshots: "folder-test.svg",
  // Storybook
  ".storybook": "folder-storybook.svg",
  stories: "folder-storybook.svg",
  // Prisma
  prisma: "folder-prisma.svg",
  // Migrations
  migrations: "folder-migrations.svg",
  migration: "folder-migrations.svg",
  // Seeds
  seeds: "folder-seeders.svg",
  seeders: "folder-seeders.svg",
  // Husky
  ".husky": "folder-husky.svg",
  // Tauri
  "src-tauri": "folder-src-tauri.svg",
  // Next.js
  ".next": "folder-next.svg",
  // Nuxt
  ".nuxt": "folder-nuxt.svg",
  // Svelte
  ".svelte-kit": "folder-svelte.svg",
  // Android
  android: "folder-android.svg",
  // iOS
  ios: "folder-ios.svg",
  // React Native
  // Flutter
  flutter: "folder-flutter.svg",
  // Supabase
  supabase: "folder-supabase.svg",
  // Vercel
  ".vercel": "folder-vercel.svg",
  // Netlify
  ".netlify": "folder-netlify.svg",
  // Drizzle
  drizzle: "folder-drizzle.svg",
};

export interface IconInfo {
  /** Full path to the icon SVG */
  path: string;
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
      path: `${UI}/ic_link.svg`,
      category: "ui",
      isFolder: false,
    };
  }

  if (entry.kind === "junction") {
    return {
      path: `${UI}/ic_folder_link.svg`,
      category: "ui",
      isFolder: false,
    };
  }

  // Check special filenames first (highest priority for files)
  if (SPECIAL_FILENAME_MAP[name]) {
    return {
      path: `${DEV_ICONS}/${SPECIAL_FILENAME_MAP[name]}`,
      category: "dev",
      isFolder: false,
    };
  }

  // Check case-insensitive filename match
  const lowerName = name.toLowerCase();
  for (const [key, value] of Object.entries(SPECIAL_FILENAME_MAP)) {
    if (key.toLowerCase() === lowerName) {
      return {
        path: `${DEV_ICONS}/${value}`,
        category: "dev",
        isFolder: false,
      };
    }
  }

  // Check dev icons by extension
  if (ext && DEV_ICON_MAP[ext]) {
    return {
      path: `${DEV_ICONS}/${DEV_ICON_MAP[ext]}`,
      category: "dev",
      isFolder: false,
    };
  }

  // Check filetypes icons
  if (ext && FILETYPE_ICON_MAP[ext]) {
    return {
      path: `${FILETYPES}/${FILETYPE_ICON_MAP[ext]}`,
      category: "filetype",
      isFolder: false,
    };
  }

  // Default file icon
  return {
    path: `${FILETYPES}/document.svg`,
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
    const openIconName = isOpen ? iconName.replace(".svg", "-open.svg") : iconName;
    return {
      path: `${FILETYPES}/${openIconName}`,
      category: "filetype",
      isFolder: true,
      isOpen,
    };
  }

  // Default folder icon
  return {
    path: isOpen ? `${FILETYPES}/folder-base-open.svg` : `${FILETYPES}/folder-base.svg`,
    category: "filetype",
    isFolder: true,
    isOpen,
  };
}

/**
 * Get a drive icon path
 */
export function getDriveIconPath(driveType: string): string {
  switch (driveType) {
    case "Removable":
      return `${UI}/ic_usb_stick.svg`;
    case "Network":
      return `${UI}/ic_globe.svg`;
    case "CdRom":
      return `${UI}/ic_disc.svg`;
    case "RamDisk":
      return `${UI}/ic_storage.svg`;
    default:
      return `${UI}/ic_hard_drive.svg`;
  }
}

/**
 * Get UI icon path by name
 */
export function getUiIconPath(iconName: string): string {
  return `${UI}/ic_${iconName}.svg`;
}
