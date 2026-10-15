'use client'

import * as React from 'react'

const CRATES = ['merkle-core', 'merkleforge-hash', 'merkle-variants'] as const

type Crate = (typeof CRATES)[number]

type Counts = { downloads: number; recent: number }

type State =
  | { status: 'loading' }
  | { status: 'ready'; counts: Record<Crate, Counts> }
  | { status: 'error' }

const format = new Intl.NumberFormat('en-US')

/**
 * Live crates.io download counts for every published MerkleForge crate.
 * Fetched in the browser (crates.io sends `Access-Control-Allow-Origin: *`),
 * so the numbers stay current without rebuilding the static site.
 */
export function CrateDownloads() {
  const [state, setState] = React.useState<State>({ status: 'loading' })

  React.useEffect(() => {
    let cancelled = false
    Promise.all(
      CRATES.map(async (name) => {
        const response = await fetch(`https://crates.io/api/v1/crates/${name}`)
        if (!response.ok) throw new Error(`HTTP ${response.status}`)
        const { crate } = await response.json()
        return [name, { downloads: crate.downloads, recent: crate.recent_downloads ?? 0 }] as const
      }),
    )
      .then((entries) => {
        if (!cancelled)
          setState({ status: 'ready', counts: Object.fromEntries(entries) as Record<Crate, Counts> })
      })
      .catch(() => {
        if (!cancelled) setState({ status: 'error' })
      })
    return () => {
      cancelled = true
    }
  }, [])

  const show = (pick: (counts: Counts) => number, crates: readonly Crate[] = CRATES) => {
    if (state.status === 'loading') return '…'
    if (state.status === 'error') return '—'
    return format.format(crates.reduce((sum, name) => sum + pick(state.counts[name]), 0))
  }

  return (
    <div className="mf-grid">
      <div className="mf-card mf-stat" data-variant="total">
        <div className="mf-eyebrow">Total downloads</div>
        <div className="mf-stat-value">{show((c) => c.downloads)}</div>
        <div className="mf-stat-note">{show((c) => c.recent)} in the last 90 days</div>
      </div>
      {CRATES.map((name) => (
        <a
          key={name}
          className="mf-card mf-stat"
          href={`https://crates.io/crates/${name}`}
          target="_blank"
          rel="noreferrer"
        >
          <div className="mf-eyebrow">{name}</div>
          <div className="mf-stat-value">{show((c) => c.downloads, [name])}</div>
          <div className="mf-stat-note">{show((c) => c.recent, [name])} in the last 90 days</div>
        </a>
      ))}
    </div>
  )
}
