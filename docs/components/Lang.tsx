'use client'

import * as React from 'react'

/**
 * Site-wide Rust / TypeScript switch.
 *
 * The chosen language lives on `<html data-mf-lang="rust|ts">`. An inline
 * script in `vocs.config.ts` sets it before first paint, from `?lang=` or the
 * reader's saved choice, so there is no flash of the wrong language and the
 * fully static build needs no hydration to show the right content.
 * `<Rust>` and `<TypeScript>` blocks are then shown or hidden purely by CSS
 * (see `_root.css`).
 */
export type Lang = 'rust' | 'ts'

const STORAGE_KEY = 'mf-lang'
const EVENT = 'mf-lang-change'

function current(): Lang {
  return document.documentElement.getAttribute('data-mf-lang') === 'ts' ? 'ts' : 'rust'
}

function subscribe(onChange: () => void) {
  // `storage` keeps several open tabs in step.
  const onStorage = (event: StorageEvent) => {
    if (event.key !== STORAGE_KEY) return
    document.documentElement.setAttribute('data-mf-lang', event.newValue === 'ts' ? 'ts' : 'rust')
    onChange()
  }
  window.addEventListener(EVENT, onChange)
  window.addEventListener('storage', onStorage)
  return () => {
    window.removeEventListener(EVENT, onChange)
    window.removeEventListener('storage', onStorage)
  }
}

export function setLang(lang: Lang) {
  document.documentElement.setAttribute('data-mf-lang', lang)
  try {
    localStorage.setItem(STORAGE_KEY, lang)
  } catch {}
  // Drop a `?lang=` that would contradict the choice on reload.
  const url = new URL(window.location.href)
  if (url.searchParams.has('lang')) {
    url.searchParams.delete('lang')
    window.history.replaceState(window.history.state, '', url)
  }
  window.dispatchEvent(new Event(EVENT))
}

export function useLang(): Lang {
  return React.useSyncExternalStore(subscribe, current, () => 'rust')
}

const OPTIONS: { value: Lang; label: string }[] = [
  { value: 'rust', label: 'Rust' },
  { value: 'ts', label: 'TypeScript' },
]

/** Segmented Rust / TypeScript control. */
export function LangSwitch() {
  const lang = useLang()
  return (
    <div className="mf-lang-switch" role="group" aria-label="Code language">
      {OPTIONS.map((option) => (
        <button
          key={option.value}
          type="button"
          aria-pressed={lang === option.value}
          data-active={lang === option.value || undefined}
          onClick={() => setLang(option.value)}
        >
          {option.label}
        </button>
      ))}
    </div>
  )
}

/**
 * In-page switch for pages with language-specific content. Hidden on wide
 * screens, where the sidebar shows the same control, unless `always` is set
 * (for pages with no sidebar, such as the minimal-layout Examples page).
 */
export function LangBar({ always }: { always?: boolean }) {
  return (
    <div className="mf-lang-bar" data-always={always || undefined}>
      <span>Showing code for</span>
      <LangSwitch />
    </div>
  )
}

type BlockProps = React.PropsWithChildren<{
  /** Render inline (inside a sentence) instead of as a block. */
  inline?: boolean
}>

function Block({ lang, children, inline }: BlockProps & { lang: Lang }) {
  const className = `mf-lang-${lang}`
  if (inline) return <span className={className}>{children}</span>
  // Same spacing hooks as Vocs' article container, so Markdown inside a
  // block is laid out exactly like Markdown outside it.
  return (
    <div className={`${className} vocs:space-y-6`} data-v-content>
      {children}
    </div>
  )
}

/** Content shown only when Rust is selected. */
export function Rust(props: BlockProps) {
  return <Block lang="rust" {...props} />
}

/** Content shown only when TypeScript is selected. */
export function TypeScript(props: BlockProps) {
  return <Block lang="ts" {...props} />
}
