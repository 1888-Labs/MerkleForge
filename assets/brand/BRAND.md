# MerkleForge brand kit

The mark is the **Forged block**: a solid rounded block with a Merkle root and
two branches cut out of it. It reads as a tree, an anvil-struck block, and a
sealed hash at once.

## Files

| Folder | What | Use it for |
|---|---|---|
| `svg/mark/`, `png/mark/` | Mark alone, black or white, transparent cut-outs (64–1024 px) | Avatars, docs headers, small spaces |
| `svg/lockup/`, `png/lockup/` | Mark + wordmark, horizontal and stacked | README, website header, slides |
| `svg/wordmark/`, `png/wordmark/` | Wordmark only | Where the mark already appears nearby |
| `svg/icon/`, `png/icon/` | App icon (dark/light, 1024) and avatar (512/1024) | GitHub org, crates.io, X, Discord |
| `favicon/` | `favicon.svg` (adapts to dark mode), `.ico`, 16/32/48 PNG, apple-touch, 192/512, `site.webmanifest` | Website / docs site |
| `svg/banner/`, `png/banner/` | GitHub social preview, README, X header, LinkedIn (dark + light, 1x/2x) | Social and repository cards |

All wordmark text is converted to outlines, so the SVGs need no fonts installed.

### Favicon snippet

```html
<link rel="icon" href="/favicon.svg" type="image/svg+xml">
<link rel="icon" href="/favicon.ico" sizes="any">
<link rel="apple-touch-icon" href="/apple-touch-icon.png">
<link rel="manifest" href="/site.webmanifest">
```

### README snippet

```html
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/brand/png/banner/merkleforge-readme-1280x320@2x.png">
  <img alt="MerkleForge" src="assets/brand/png/banner/merkleforge-readme-light-1280x320@2x.png">
</picture>
```

## Colour

Monochrome by design — black, white and a short grey ramp. No accent colour.

| Name | Hex | Role |
|---|---|---|
| Forge Black | `#000000` | Primary ground, mark on light |
| Ink | `#141414` | Raised surfaces, ghost mark in banners |
| Steel | `#262626` | Borders, dividers on dark |
| Graphite | `#6B6B6B` | Secondary text on light, large only on dark |
| Slate | `#5C5C5C` | "Forge" in the wordmark on light |
| Ash | `#A3A3A3` | "Forge" in the wordmark on dark, captions on dark |
| White | `#FFFFFF` | Mark on dark, primary text on dark |

## Type

- **Space Grotesk** — wordmark (Merkle 700 / Forge 400, tracking −3%), headings 500–700.
- **JetBrains Mono** — code, hashes, technical labels (400–500).

Both are free on Google Fonts (OFL).

## Usage

- **Clear space:** keep at least ¼ of the mark's height empty on every side.
- **Minimum size:** mark 16 px; horizontal lockup 24 px tall.
- Use the black mark on light grounds and the white mark on dark grounds.
- Don't recolour, rotate, outline, stretch, add shadows or gradients, or
  rearrange the mark and wordmark.
- Don't place the mark on busy photos; use a solid black or white panel.

Fonts: Space Grotesk and JetBrains Mono, SIL Open Font License.
