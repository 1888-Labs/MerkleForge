'use client'

import type { CSSProperties } from 'react'
import { useConfig } from 'vocs'

const swatches = [
  { name: 'Forge Black', hex: '#000000' },
  { name: 'Ink', hex: '#141414' },
  { name: 'Steel', hex: '#262626' },
  { name: 'Slate', hex: '#5C5C5C' },
  { name: 'Graphite', hex: '#6B6B6B' },
  { name: 'Ash', hex: '#A3A3A3' },
  { name: 'White', hex: '#FFFFFF' },
]

function useAsset() {
  const { basePath } = useConfig()
  const base = basePath.endsWith('/') ? basePath : `${basePath}/`
  return (file: string) => `${base}${file}`
}

/** The lockup and mark on black and white panels. */
export function BrandPreview() {
  const asset = useAsset()
  const panel = (background: string, border: string): CSSProperties => ({
    background,
    border: `1px solid ${border}`,
    borderRadius: 12,
    padding: 32,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 32,
    minHeight: 140,
  })
  return (
    <div className="mf-grid" data-cols="2" style={{ margin: '24px 0' }}>
      <div style={panel('#000000', '#262626')}>
        <img alt="MerkleForge mark, white" height={48} src={asset('merkleforge-mark-white.svg')} />
        <img alt="MerkleForge lockup, white" height={36} src={asset('merkleforge-lockup-white.svg')} />
      </div>
      <div style={panel('#FFFFFF', '#E5E5E5')}>
        <img alt="MerkleForge mark, black" height={48} src={asset('merkleforge-mark-black.svg')} />
        <img alt="MerkleForge lockup, black" height={36} src={asset('merkleforge-lockup-black.svg')} />
      </div>
    </div>
  )
}

/** The 1888 Labs logo on its light and dark panels. */
export function OwnerPreview() {
  const asset = useAsset()
  const panel = (background: string, border: string): CSSProperties => ({
    background,
    border: `1px solid ${border}`,
    borderRadius: 12,
    padding: 32,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: 140,
  })
  return (
    <div className="mf-grid" data-cols="2" style={{ margin: '24px 0' }}>
      <div style={panel('#0F1115', '#262626')}>
        <img alt="1888 Labs logo, on dark" height={56} src={asset('1888labs-logo-on-dark.svg')} />
      </div>
      <div style={panel('#F3F1EA', '#E5E5E5')}>
        <img alt="1888 Labs logo, on light" height={56} src={asset('1888labs-logo-on-light.svg')} />
      </div>
    </div>
  )
}

/** The monochrome palette as swatches. */
export function BrandSwatches() {
  return (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(120px, 1fr))',
        gap: 10,
        margin: '20px 0',
      }}
    >
      {swatches.map((swatch) => (
        <div className="mf-card" key={swatch.hex} style={{ padding: 0, overflow: 'hidden' }}>
          <div style={{ background: swatch.hex, height: 56, borderBottom: '1px solid #262626' }} />
          <div style={{ padding: '8px 10px' }}>
            <div style={{ fontSize: 13, fontWeight: 600 }}>{swatch.name}</div>
            <div className="mf-mono mf-muted">{swatch.hex}</div>
          </div>
        </div>
      ))}
    </div>
  )
}
