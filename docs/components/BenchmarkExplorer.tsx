'use client'

import * as React from 'react'
import { useConfig } from 'vocs'

type BenchmarkRow = {
  id: string
  tree_variant: string
  operation: string
  operation_key: string
  algorithm: string
  size: string
  size_value: number
  mean_ns: number
  lower_ns: number
  upper_ns: number
  elements_per_second: number | null
  report: string
}

type BenchmarkData = {
  benchmarks: BenchmarkRow[]
  generatedAt?: string
  commit?: string
}

type State =
  | { status: 'loading' }
  | { status: 'missing'; reason: string }
  | { status: 'ready'; data: BenchmarkData }

const ALL = 'All'

/** Join the site base path with a relative asset path. */
function withBase(basePath: string, file: string) {
  const base = basePath.endsWith('/') ? basePath : `${basePath}/`
  return `${base}${file.replace(/^\/+/, '')}`
}

function formatNs(ns: number) {
  if (!Number.isFinite(ns)) return '–'
  if (ns < 1_000) return `${ns.toFixed(2)} ns`
  if (ns < 1_000_000) return `${(ns / 1_000).toFixed(2)} µs`
  if (ns < 1_000_000_000) return `${(ns / 1_000_000).toFixed(2)} ms`
  return `${(ns / 1_000_000_000).toFixed(2)} s`
}

function formatRate(value: number | null) {
  if (value == null || !Number.isFinite(value)) return '–'
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(2)} M/s`
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)} K/s`
  return `${value.toFixed(0)} /s`
}

function unique(values: string[]) {
  return [ALL, ...Array.from(new Set(values)).sort((a, b) => a.localeCompare(b))]
}

/**
 * Live Criterion results. Fetches `<basePath>/benchmark-data.json`, which the
 * deploy build writes into the public directory, and links each row to its
 * Criterion HTML report under `<basePath>/<report>`.
 */
export function BenchmarkExplorer() {
  const { basePath } = useConfig()
  const [state, setState] = React.useState<State>({ status: 'loading' })
  const [variant, setVariant] = React.useState(ALL)
  const [operation, setOperation] = React.useState(ALL)
  const [algorithm, setAlgorithm] = React.useState(ALL)
  const [size, setSize] = React.useState(ALL)

  React.useEffect(() => {
    let cancelled = false
    fetch(withBase(basePath, 'benchmark-data.json'), { cache: 'no-cache' })
      .then(async (response) => {
        if (!response.ok) throw new Error(`HTTP ${response.status}`)
        const data = (await response.json()) as BenchmarkData
        if (!Array.isArray(data?.benchmarks) || data.benchmarks.length === 0)
          throw new Error('no benchmark rows')
        if (!cancelled) setState({ status: 'ready', data })
      })
      .catch((error: unknown) => {
        if (!cancelled)
          setState({
            status: 'missing',
            reason: error instanceof Error ? error.message : String(error),
          })
      })
    return () => {
      cancelled = true
    }
  }, [basePath])

  const rows = state.status === 'ready' ? state.data.benchmarks : []

  // Fastest mean per (variant, operation, size) group.
  const fastest = React.useMemo(() => {
    const best = new Map<string, number>()
    for (const row of rows) {
      const key = `${row.tree_variant}|${row.operation}|${row.size}`
      const current = best.get(key)
      if (current === undefined || row.mean_ns < current) best.set(key, row.mean_ns)
    }
    return best
  }, [rows])

  const filtered = React.useMemo(
    () =>
      rows
        .filter(
          (row) =>
            (variant === ALL || row.tree_variant === variant) &&
            (operation === ALL || row.operation === operation) &&
            (algorithm === ALL || row.algorithm === algorithm) &&
            (size === ALL || row.size === size),
        )
        .sort(
          (a, b) =>
            a.tree_variant.localeCompare(b.tree_variant) ||
            a.operation.localeCompare(b.operation) ||
            a.size_value - b.size_value ||
            a.mean_ns - b.mean_ns,
        ),
    [rows, variant, operation, algorithm, size],
  )

  if (state.status === 'loading')
    return <p className="mf-muted">Loading benchmark results…</p>

  if (state.status === 'missing')
    return (
      <div className="mf-card">
        <strong>No published benchmark data yet.</strong>
        <p className="mf-muted" style={{ marginTop: 8 }}>
          This page reads <code>benchmark-data.json</code>, which the site build generates from
          Criterion output ({state.reason}). Run the suite locally with{' '}
          <code>cargo bench --workspace</code> and open{' '}
          <code>target/criterion/report/index.html</code>, or see the summary tables below.
        </p>
      </div>
    )

  const { data } = state
  const sizes = Array.from(new Set(rows.map((row) => row.size))).sort(
    (a, b) => Number(a) - Number(b),
  )

  return (
    <div>
      <div className="mf-row" style={{ justifyContent: 'space-between' }}>
        <p className="mf-muted" style={{ margin: 0, fontSize: 14 }}>
          {rows.length} measurements
          {data.generatedAt && <> · generated {new Date(data.generatedAt).toUTCString()}</>}
          {data.commit && (
            <>
              {' '}
              · commit <code>{data.commit.slice(0, 10)}</code>
            </>
          )}
        </p>
        <a
          className="mf-button"
          href={withBase(basePath, 'reports/criterion/report/index.html')}
          rel="noreferrer"
          target="_blank"
        >
          Full Criterion report
        </a>
      </div>

      <div className="mf-filters">
        <Filter label="Tree" onChange={setVariant} options={unique(rows.map((r) => r.tree_variant))} value={variant} />
        <Filter label="Operation" onChange={setOperation} options={unique(rows.map((r) => r.operation))} value={operation} />
        <Filter label="Hash" onChange={setAlgorithm} options={unique(rows.map((r) => r.algorithm))} value={algorithm} />
        <Filter label="Size" onChange={setSize} options={[ALL, ...sizes]} value={size} />
      </div>

      <div className="mf-table-wrap">
        <table className="mf-table">
          <thead>
            <tr>
              <th>Tree</th>
              <th>Operation</th>
              <th>Hash</th>
              <th className="num">Size</th>
              <th className="num">Mean</th>
              <th className="num">95% CI</th>
              <th className="num">Throughput</th>
              <th>Report</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((row) => {
              const isFastest =
                fastest.get(`${row.tree_variant}|${row.operation}|${row.size}`) === row.mean_ns
              return (
                <tr data-fastest={isFastest} key={row.id}>
                  <td>{row.tree_variant}</td>
                  <td>{row.operation}</td>
                  <td>
                    {row.algorithm}
                    {isFastest && (
                      <span className="mf-pill" data-state="ok" style={{ marginLeft: 8 }}>
                        fastest
                      </span>
                    )}
                  </td>
                  <td className="num">{Number(row.size).toLocaleString('en-US')}</td>
                  <td className="num">{formatNs(row.mean_ns)}</td>
                  <td className="num">
                    {formatNs(row.lower_ns)} – {formatNs(row.upper_ns)}
                  </td>
                  <td className="num">{formatRate(row.elements_per_second)}</td>
                  <td>
                    <a href={withBase(basePath, row.report)} rel="noreferrer" target="_blank">
                      Criterion
                    </a>
                  </td>
                </tr>
              )
            })}
            {filtered.length === 0 && (
              <tr>
                <td className="mf-muted" colSpan={8}>
                  No measurements match these filters.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
      <p className="mf-muted" style={{ fontSize: 13, marginTop: 10 }}>
        Bold rows are the fastest hash for that tree, operation, and size. Lower is better.
      </p>
    </div>
  )
}

function Filter(props: {
  readonly label: string
  readonly options: string[]
  readonly value: string
  readonly onChange: (value: string) => void
}) {
  const { label, options, value, onChange } = props
  const id = `mf-filter-${label.toLowerCase()}`
  return (
    <div>
      <label className="mf-label" htmlFor={id}>
        {label}
      </label>
      <select className="mf-select" id={id} onChange={(e) => onChange(e.target.value)} value={value}>
        {options.map((option) => (
          <option key={option} value={option}>
            {option === ALL ? 'All' : option}
          </option>
        ))}
      </select>
    </div>
  )
}
