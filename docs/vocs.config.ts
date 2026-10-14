import { readFileSync } from 'node:fs'
import { defineConfig } from 'vocs/config'

/**
 * Base path the site is served from. GitHub Pages serves the site at
 * https://1888-labs.github.io/MerkleForge/, so the default is `/MerkleForge`.
 * Override with e.g. `VOCS_BASE_PATH=/ npm run dev`.
 */
const basePath = (process.env.VOCS_BASE_PATH ?? '/MerkleForge').replace(/\/+$/, '') || '/'

/** Prefix a public asset path with the base path (Vocs does not do this for logo/icon URLs). */
const asset = (file: string) => (basePath === '/' ? `/${file}` : `${basePath}/${file}`)

/** Crate version, read from the workspace manifest so releases only bump `Cargo.toml`. */
const version = (() => {
  const manifest = readFileSync(new URL('../Cargo.toml', import.meta.url), 'utf8')
  const match = manifest.match(/^\[workspace\.package\][^[]*?^version\s*=\s*"([^"]+)"/m)
  if (!match) throw new Error('could not read [workspace.package] version from Cargo.toml')
  return match[1]
})()
const repo = 'https://github.com/1888-Labs/MerkleForge'

export default defineConfig({
  title: 'MerkleForge',
  titleTemplate: '%s · MerkleForge',
  description:
    'Authenticated data structures for Rust: Binary, Sparse, and Ethereum-compatible Patricia Merkle trees behind one API, with pluggable hashes and stateless proofs.',
  basePath,
  baseUrl: 'https://1888-labs.github.io',
  // Pages live in `docs/pages`, static files in `docs/public`.
  srcDir: '.',
  // Plain static HTML for GitHub Pages. Output: `docs/dist/public`.
  renderStrategy: 'full-static',
  // Monochrome brand: no accent colour. Black on light, white on dark.
  accentColor: 'light-dark(#000000, #ffffff)',
  colorScheme: 'light dark',
  logoUrl: {
    light: asset('merkleforge-lockup-black.svg'),
    dark: asset('merkleforge-lockup-white.svg'),
  },
  iconUrl: asset('favicon.svg'),
  ogImageUrl: `https://1888-labs.github.io${asset('og.png')}`,
  codeHighlight: {
    themes: {
      light: 'github-light',
      dark: 'github-dark-default',
    },
  },
  head: {
    // `baseUrl` is only needed for canonical/OG URLs; skip the <base> tag so
    // local previews keep resolving assets from localhost.
    base: false,
    link: [
      { rel: 'icon', href: asset('favicon.ico'), sizes: 'any' },
      { rel: 'apple-touch-icon', href: asset('apple-touch-icon.png') },
      { rel: 'manifest', href: asset('site.webmanifest') },
      { rel: 'preconnect', href: 'https://fonts.googleapis.com' },
      { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: 'anonymous' },
      {
        rel: 'stylesheet',
        href: 'https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500&family=Space+Grotesk:wght@400;500;600;700&display=swap',
      },
    ],
    meta: {
      themeColor: '#000000',
    },
    script: [
      {
        // Dark is the primary theme: use it unless the reader picked a theme
        // with the toggle. Runs before paint, after Vocs' own theme script.
        innerHTML:
          "(function(){try{if(!localStorage.getItem('vocs-theme')){var e=document.documentElement;e.setAttribute('data-vocs-theme','dark');e.style.colorScheme='dark'}}catch(e){}})()",
      },
    ],
  },
  editLink: {
    link: `${repo}/edit/main/docs/pages/:path`,
    text: 'Suggest changes to this page',
  },
  socials: [{ icon: 'github', link: repo }],
  topNav: [
    { text: 'Docs', link: '/docs/getting-started', match: '/docs' },
    { text: 'Examples', link: '/examples', match: '/examples' },
    { text: 'Demo', link: '/demo', match: '/demo' },
    { text: 'Benchmarks', link: '/benchmarks', match: '/benchmarks' },
    {
      text: `v${version}`,
      items: [
        { text: 'Changelog', link: `${repo}/blob/main/CHANGELOG.md` },
        { text: 'Contributing', link: '/docs/community/contributing' },
        { text: 'crates.io', link: 'https://crates.io/crates/merkle-variants' },
      ],
    },
  ],
  sidebar: {
    '/docs/': [
      {
        text: 'Introduction',
        items: [
          { text: 'Why MerkleForge', link: '/docs' },
          { text: 'Getting started', link: '/docs/getting-started' },
          { text: 'Mental model', link: '/docs/mental-model' },
        ],
      },
      {
        text: 'Guides',
        items: [
          { text: 'Choosing a tree', link: '/docs/guides/choosing-a-tree' },
          { text: 'Binary Merkle tree', link: '/docs/guides/binary-merkle-tree' },
          { text: 'Sparse Merkle tree', link: '/docs/guides/sparse-merkle-tree' },
          { text: 'Merkle Patricia trie', link: '/docs/guides/merkle-patricia-trie' },
          { text: 'Hash functions', link: '/docs/guides/hash-functions' },
          { text: 'Proofs & stateless verification', link: '/docs/guides/proofs' },
          { text: 'Serialization', link: '/docs/guides/serialization' },
          { text: 'Metadata', link: '/docs/guides/metadata' },
          { text: 'Errors', link: '/docs/guides/errors' },
          { text: 'no_std', link: '/docs/guides/no-std' },
        ],
      },
      {
        text: 'Performance',
        items: [
          { text: 'Benchmarks', link: '/docs/performance/benchmarks' },
          { text: 'Profiling & energy', link: '/docs/performance/profiling' },
        ],
      },
      {
        text: 'Reference',
        items: [
          { text: 'API overview', link: '/docs/reference/api' },
          { text: 'Changelog', link: '/docs/reference/changelog' },
          { text: 'Security', link: '/docs/reference/security' },
          { text: 'Brand assets', link: '/docs/reference/brand' },
        ],
      },
      {
        text: 'Community',
        items: [
          { text: 'Contributing', link: '/docs/community/contributing' },
          { text: 'Code of Conduct', link: '/docs/community/code-of-conduct' },
          { text: 'Projects using MerkleForge', link: '/docs/community/projects' },
        ],
      },
    ],
  },
})
