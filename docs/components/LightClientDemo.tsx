'use client'

import * as React from 'react'
import { type DemoResult, runProof } from './merkle'

const defaultRows = [
  'tx:alice->bob:100',
  'tx:bob->carol:50',
  'tx:carol->dave:25',
  'tx:dave->erin:10',
  'tx:erin->frank:5',
].join('\n')

function short(hex: string) {
  return hex.length > 20 ? `${hex.slice(0, 10)}…${hex.slice(-10)}` : hex
}

/**
 * Interactive light-client demo. Mirrors `BinaryMerkleTree<Sha256>` in the
 * browser with WebCrypto. All hashing runs in effects, so server rendering
 * never touches `crypto.subtle`.
 */
export function LightClientDemo() {
  const [rowsText, setRowsText] = React.useState(defaultRows)
  const rows = React.useMemo(
    () =>
      rowsText
        .split('\n')
        .map((row) => row.trim())
        .filter((row) => row.length > 0),
    [rowsText],
  )
  const [index, setIndex] = React.useState(0)
  const safeIndex = Math.min(index, Math.max(rows.length - 1, 0))
  const [claimed, setClaimed] = React.useState<string>(rows[0] ?? '')
  const [result, setResult] = React.useState<DemoResult | null>(null)
  const [error, setError] = React.useState<string | null>(null)

  // Reset the claimed value whenever the selected row changes.
  // biome-ignore lint/correctness/useExhaustiveDependencies: only on selection change
  React.useEffect(() => {
    setClaimed(rows[safeIndex] ?? '')
  }, [safeIndex, rows[safeIndex]])

  React.useEffect(() => {
    let cancelled = false
    if (rows.length === 0) {
      setResult(null)
      setError('Add at least one row to build a tree.')
      return
    }
    if (typeof crypto === 'undefined' || !crypto.subtle) {
      setError('This browser does not expose WebCrypto (crypto.subtle). Use HTTPS or localhost.')
      return
    }
    runProof(rows, safeIndex, claimed)
      .then((next) => {
        if (cancelled) return
        setResult(next)
        setError(null)
      })
      .catch((err: unknown) => {
        if (cancelled) return
        setError(err instanceof Error ? err.message : String(err))
      })
    return () => {
      cancelled = true
    }
  }, [rows, safeIndex, claimed])

  const tampered = claimed !== (rows[safeIndex] ?? '')

  return (
    <div className="mf-demo">
      <div className="mf-stack">
        <div className="mf-card mf-stack">
          <div className="mf-eyebrow">1 · Server builds the tree</div>
          <div>
            <label className="mf-label" htmlFor="mf-demo-rows">
              Rows (one leaf per line)
            </label>
            <textarea
              className="mf-textarea"
              id="mf-demo-rows"
              onChange={(event) => setRowsText(event.target.value)}
              spellCheck={false}
              value={rowsText}
            />
          </div>
          <div>
            <label className="mf-label" htmlFor="mf-demo-index">
              Row to prove
            </label>
            <select
              className="mf-select"
              id="mf-demo-index"
              onChange={(event) => setIndex(Number(event.target.value))}
              value={safeIndex}
            >
              {rows.map((row, i) => (
                <option key={`${i}-${row}`} value={i}>
                  LeafIndex({i}) · {row}
                </option>
              ))}
            </select>
          </div>
        </div>

        <div className="mf-card mf-stack">
          <div className="mf-eyebrow">2 · Client receives a claim</div>
          <div>
            <label className="mf-label" htmlFor="mf-demo-claim">
              Claimed value (edit it to tamper)
            </label>
            <input
              className="mf-input"
              id="mf-demo-claim"
              onChange={(event) => setClaimed(event.target.value)}
              spellCheck={false}
              value={claimed}
            />
          </div>
          <div className="mf-row">
            <button
              className="mf-button"
              disabled={!tampered}
              onClick={() => setClaimed(rows[safeIndex] ?? '')}
              type="button"
            >
              Reset to real value
            </button>
            <button
              className="mf-button"
              onClick={() => setClaimed(`${rows[safeIndex] ?? ''}0`)}
              type="button"
            >
              Tamper
            </button>
          </div>
        </div>
      </div>

      <div className="mf-card mf-stack">
        <div className="mf-row" style={{ justifyContent: 'space-between' }}>
          <div className="mf-eyebrow">3 · Stateless verification</div>
          {result && !error && (
            <span className="mf-pill" data-state={result.verified ? 'ok' : 'bad'}>
              {result.verified ? 'Verified' : 'Rejected'}
            </span>
          )}
        </div>

        {error && <p className="mf-muted">{error}</p>}

        {result && !error && (
          <>
            <dl className="mf-kv">
              <dt>Trusted root</dt>
              <dd className="mf-mono">{result.root}</dd>
              <dt>Leaf count</dt>
              <dd className="mf-mono">{result.leafCount}</dd>
              <dt>Claimed leaf hash</dt>
              <dd className="mf-mono">{result.leafHash}</dd>
              <dt>Recomputed root</dt>
              <dd className="mf-mono">{result.recomputedRoot}</dd>
            </dl>

            <div>
              <div className="mf-label">
                Proof path for LeafIndex({result.leafIndex}) · {result.proof.length} sibling
                {result.proof.length === 1 ? '' : 's'}
              </div>
              {result.proof.length === 0 ? (
                <p className="mf-muted">Single-leaf tree: the leaf hash is the root.</p>
              ) : (
                <ol className="mf-proof-list">
                  {result.proof.map((node, level) => (
                    <li key={`${level}-${node.hash}`}>
                      <span className="mf-muted mf-mono">L{level}</span>
                      <span className="mf-mono">{node.side}</span>
                      <span className="mf-mono" title={node.hash}>
                        {short(node.hash)}
                      </span>
                    </li>
                  ))}
                </ol>
              )}
            </div>

            <p className="mf-muted" style={{ fontSize: 14, margin: 0 }}>
              {result.verified
                ? 'The client rebuilt the root from the claimed value and the sibling hashes alone. It matches the trusted root.'
                : 'The recomputed root differs from the trusted root, so the claimed value is not the leaf at this index.'}
            </p>
          </>
        )}
      </div>
    </div>
  )
}
