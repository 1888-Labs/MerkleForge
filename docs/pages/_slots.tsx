'use client'

import { useRouter } from 'waku'
import { LangSwitch } from '../components/Lang'
import { OwnerCredit, owner } from '../components/Owner'

/** Site-wide footer crediting 1888 Labs. The landing page renders its own. */
export function Footer() {
  const { path } = useRouter()
  if (path === '/') return null
  return (
    <div className="mf-site-footer">
      <OwnerCredit compact />
      <span>
        Built and maintained by <a href={owner.url}>{owner.name}</a> · MIT OR Apache-2.0
      </span>
    </div>
  )
}

/** Rust / TypeScript switch at the top of the docs sidebar. */
export function SidebarHeader() {
  return <LangSwitch />
}

// Unused slot, exported so Vocs doesn't warn about a missing export.
export const OutlineFooter = undefined
