'use client'

import { useConfig } from 'vocs'

export const owner = {
  name: '1888 Labs',
  url: 'https://github.com/1888-Labs',
  tagline: 'Open infrastructure for verifiable, distributed, and intelligent systems.',
}

function useAsset() {
  const { basePath } = useConfig()
  const base = basePath.endsWith('/') ? basePath : `${basePath}/`
  return (file: string) => `${base}${file}`
}

/** The 1888 Labs logo, swapped for the current light/dark theme. */
export function OwnerLogo(props: { height?: number }) {
  const { height = 28 } = props
  const asset = useAsset()
  return (
    <>
      <img
        alt={owner.name}
        className="mf-owner-logo vocs:dark:hidden"
        height={height}
        src={asset('1888labs-logo-on-light.svg')}
      />
      <img
        alt={owner.name}
        className="mf-owner-logo vocs:hidden vocs:dark:block"
        height={height}
        src={asset('1888labs-logo-on-dark.svg')}
      />
    </>
  )
}

/** "A project by 1888 Labs" credit with the org logo. */
export function OwnerCredit(props: { compact?: boolean }) {
  const { compact = false } = props
  return (
    <a
      className={compact ? 'mf-owner mf-owner-compact' : 'mf-owner'}
      href={owner.url}
      rel="noopener"
      target="_blank"
    >
      {!compact && <span className="mf-owner-label">A project by</span>}
      <OwnerLogo height={compact ? 20 : 28} />
    </a>
  )
}
