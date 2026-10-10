const ICONIFY_API = 'https://api.iconify.design'
const CACHE_PREFIX = 'nxc-external-app-icon-v1:'
const SUCCESS_TTL_MS = 90 * 24 * 60 * 60 * 1000
const MISS_TTL_MS = 12 * 60 * 60 * 1000
const REQUEST_TIMEOUT_MS = 1800
const MAX_CACHE_ENTRIES = 120

type CacheEntry = {
  createdAt: number
  expiresAt: number
  src: string | null
}

type LookupResponse =
  | { kind: 'found'; src: string }
  | { kind: 'miss' }
  | { kind: 'error' }

const memoryCache = new Map<string, CacheEntry>()
const pendingLookups = new Map<string, Promise<string | null>>()
const allowedPrefixes = new Set(['logos', 'simple-icons'])
const allowedElements = new Set([
  'svg', 'g', 'path', 'circle', 'ellipse', 'rect', 'line', 'polyline',
  'polygon', 'defs', 'lineargradient', 'radialgradient', 'stop', 'clippath', 'mask',
])
const allowedAttributes = new Set([
  'xmlns', 'viewBox', 'width', 'height', 'fill', 'fill-rule', 'clip-rule',
  'stroke', 'stroke-width', 'stroke-linecap', 'stroke-linejoin', 'stroke-miterlimit',
  'stroke-dasharray', 'stroke-dashoffset', 'opacity', 'fill-opacity', 'stroke-opacity',
  'transform', 'd', 'cx', 'cy', 'r', 'rx', 'ry', 'x', 'y', 'x1', 'y1', 'x2', 'y2',
  'points', 'offset', 'stop-color', 'stop-opacity', 'gradientUnits', 'gradientTransform',
  'clipPathUnits', 'maskUnits', 'maskContentUnits', 'preserveAspectRatio', 'id',
])

function normalize(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9]/g, '')
}

/** Package IDs and visible names produce strict aliases; no fuzzy logo match is accepted. */
export function normalizeAppIconAliases(packageId: string, name: string): string[] {
  const leaf = packageId.split('.').at(-1) ?? packageId
  return Array.from(new Set([normalize(leaf), normalize(name)]))
    .filter((alias) => alias.length > 0 && alias.length <= 80)
    .slice(0, 4)
}

/** Only accept an exact name from a brand-logo collection to avoid unrelated fuzzy matches. */
export function selectExternalIconId(icons: readonly string[], aliases: readonly string[]): string | null {
  const expected = new Set(aliases.map(normalize))
  const matches = icons.flatMap((iconId, index) => {
    const separator = iconId.indexOf(':')
    if (separator <= 0) return []
    const prefix = iconId.slice(0, separator).toLowerCase()
    const iconName = iconId.slice(separator + 1)
    if (!allowedPrefixes.has(prefix) || !expected.has(normalize(iconName))) return []
    return [{ iconId, prefix, index }]
  })

  matches.sort((left, right) => {
    const sourcePriority = (left.prefix === 'logos' ? 0 : 1) - (right.prefix === 'logos' ? 0 : 1)
    return sourcePriority || left.index - right.index
  })
  return matches[0]?.iconId ?? null
}

function persistentKey(key: string): string {
  return CACHE_PREFIX + encodeURIComponent(key)
}

function readCache(key: string): CacheEntry | null {
  const now = Date.now()
  const memory = memoryCache.get(key)
  if (memory) {
    if (memory.expiresAt > now) return memory
    memoryCache.delete(key)
  }

  try {
    const storageKey = persistentKey(key)
    const raw = window.localStorage.getItem(storageKey)
    if (!raw) return null
    const entry = JSON.parse(raw) as CacheEntry
    if (
      typeof entry.expiresAt !== 'number' ||
      typeof entry.createdAt !== 'number' ||
      !(entry.src === null || typeof entry.src === 'string') ||
      entry.expiresAt <= now
    ) {
      window.localStorage.removeItem(storageKey)
      return null
    }
    memoryCache.set(key, entry)
    return entry
  } catch {
    // Storage may be unavailable or full; in-memory lookup still works.
    return null
  }
}

function writeCache(key: string, src: string | null): void {
  const now = Date.now()
  const entry: CacheEntry = {
    createdAt: now,
    expiresAt: now + (src ? SUCCESS_TTL_MS : MISS_TTL_MS),
    src,
  }
  memoryCache.set(key, entry)

  try {
    const storageKey = persistentKey(key)
    window.localStorage.setItem(storageKey, JSON.stringify(entry))

    const keys: string[] = []
    for (let index = 0; index < window.localStorage.length; index += 1) {
      const existing = window.localStorage.key(index)
      if (existing?.startsWith(CACHE_PREFIX)) keys.push(existing)
    }

    if (keys.length > MAX_CACHE_ENTRIES) {
      const oldestFirst = keys
        .filter((existing) => existing !== storageKey)
        .map((existing) => {
          try {
            const stored = JSON.parse(window.localStorage.getItem(existing) ?? 'null') as CacheEntry | null
            return { key: existing, createdAt: stored?.createdAt ?? 0 }
          } catch {
            return { key: existing, createdAt: 0 }
          }
        })
        .sort((left, right) => left.createdAt - right.createdAt)
      while (keys.length > MAX_CACHE_ENTRIES && oldestFirst.length > 0) {
        const oldest = oldestFirst.shift()
        if (!oldest) break
        window.localStorage.removeItem(oldest.key)
        keys.pop()
      }
    }
  } catch {
    // Cache failures must never prevent an app result from rendering.
  }
}

async function requestText(url: string): Promise<{ kind: 'ok'; text: string } | { kind: 'miss' | 'error' }> {
  const controller = new AbortController()
  const timeout = window.setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS)
  try {
    const response = await fetch(url, {
      signal: controller.signal,
      headers: { Accept: 'image/svg+xml, application/json;q=0.9, */*;q=0.1' },
    })
    if (response.status === 404) return { kind: 'miss' }
    if (!response.ok) return { kind: 'error' }
    return { kind: 'ok', text: await response.text() }
  } catch {
    return { kind: 'error' }
  } finally {
    window.clearTimeout(timeout)
  }
}

function sanitizeSvgToDataUrl(svgText: string): string | null {
  if (svgText.length === 0 || svgText.length > 80_000) return null

  const document = new DOMParser().parseFromString(svgText, 'image/svg+xml')
  const root = document.documentElement
  if (root.localName.toLowerCase() !== 'svg' || document.querySelector('parsererror')) return null

  const clean = (element: Element): boolean => {
    if (!allowedElements.has(element.localName.toLowerCase())) return false

    for (const attribute of Array.from(element.attributes)) {
      const value = attribute.value
      if (
        !allowedAttributes.has(attribute.name) ||
        /^on/i.test(attribute.name) ||
        /javascript:|https?:|data:/i.test(value) ||
        (/url\(/i.test(value) && !/url\(\s*#[^)]+\s*\)/i.test(value))
      ) {
        element.removeAttribute(attribute.name)
      }
    }

    for (const child of Array.from(element.children)) {
      if (!clean(child)) child.remove()
    }
    for (const child of Array.from(element.childNodes)) {
      if (child.nodeType !== Node.ELEMENT_NODE) child.remove()
    }
    return true
  }

  if (!clean(root)) return null
  root.setAttribute('xmlns', 'http://www.w3.org/2000/svg')
  const serialized = new XMLSerializer().serializeToString(root)
  if (serialized.length > 80_000) return null
  return 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(serialized)
}

async function lookupIcon(terms: string[], aliases: string[]): Promise<LookupResponse> {
  for (const term of terms) {
    const queryUrl = ICONIFY_API + '/search?query=' + encodeURIComponent(term) + '&limit=20'
    const search = await requestText(queryUrl)
    if (search.kind === 'error') return { kind: 'error' }
    if (search.kind === 'miss') continue

    let iconIds: string[]
    try {
      const payload: unknown = JSON.parse(search.text)
      if (!payload || typeof payload !== 'object' || !('icons' in payload)) return { kind: 'error' }
      const value = (payload as { icons: unknown }).icons
      if (!Array.isArray(value)) return { kind: 'error' }
      iconIds = value.filter((value): value is string => typeof value === 'string')
    } catch {
      return { kind: 'error' }
    }

    const iconId = selectExternalIconId(iconIds, aliases)
    if (!iconId) continue
    const parts = iconId.split(':')
    const iconUrl = ICONIFY_API + '/' + parts[0] + '/' + encodeURIComponent(parts[1]) + '.svg'
    const iconResponse = await requestText(iconUrl)
    if (iconResponse.kind !== 'ok') return { kind: 'error' }

    const src = sanitizeSvgToDataUrl(iconResponse.text)
    if (!src) return { kind: 'error' }
    return { kind: 'found', src }
  }

  return { kind: 'miss' }
}

let activeLookups = 0
const lookupQueue: Array<() => void> = []
const MAX_CONCURRENT_LOOKUPS = 2

function runLimited<T>(operation: () => Promise<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    const run = () => {
      activeLookups += 1
      void operation()
        .then(resolve, reject)
        .finally(() => {
          activeLookups -= 1
          lookupQueue.shift()?.()
        })
    }

    if (activeLookups < MAX_CONCURRENT_LOOKUPS) run()
    else lookupQueue.push(run)
  })
}

/**
 * Resolve missing app icons from Iconify's brand collections.
 * Local AppStream/theme icons stay first priority; this function only runs as fallback.
 */
export function findExternalAppIcon(packageId: string, name: string): Promise<string | null> {
  const aliases = normalizeAppIconAliases(packageId, name)
  if (aliases.length === 0) return Promise.resolve(null)

  const key = aliases.join('|')
  const cached = readCache(key)
  if (cached) return Promise.resolve(cached.src)
  if (typeof navigator !== 'undefined' && !navigator.onLine) return Promise.resolve(null)

  const pending = pendingLookups.get(key)
  if (pending) return pending

  const leaf = packageId.split('.').at(-1) ?? packageId
  const terms = Array.from(new Set([leaf, name]
    .map((term) => term.trim().toLowerCase().replace(/[^a-z0-9 _.-]/g, ''))
    .filter((term) => term.length > 0 && term.length <= 80)))
    .slice(0, 2)

  const task = runLimited(() => lookupIcon(terms, aliases))
    .then((result) => {
      if (result.kind === 'found') {
        writeCache(key, result.src)
        return result.src
      }
      if (result.kind === 'miss') writeCache(key, null)
      return null
    })
    .catch(() => null)
    .finally(() => pendingLookups.delete(key))

  pendingLookups.set(key, task)
  return task
}
