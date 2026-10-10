import { useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Package } from 'lucide-react'
import { findExternalAppIcon } from './external-app-icon'
import {
  collectIconCandidates,
  getActiveIconCandidate,
  isVisuallyBlankPixels,
  nextCandidateIndex,
} from './app-icon-logic'

function reportIconDiagnostic(message: string) {
  if (import.meta.env.VITE_ICON_DEBUG !== '1') return
  void invoke('log_icon_debug', { message }).catch(() => {
    // The component can also be rendered outside Tauri during frontend development.
  })
}

function describeIconSource(source: string): string {
  if (source.startsWith('data:image/png;base64,')) return 'PNG (' + source.length + ' chars)'
  if (source.startsWith('data:image/svg+xml;base64,') || source.startsWith('data:image/svg+xml;charset=utf-8,')) {
    return 'SVG (' + source.length + ' chars)'
  }
  if (source.startsWith('data:image/jpeg;base64,')) return 'JPEG (' + source.length + ' chars)'
  if (source.startsWith('data:')) return 'data URL (' + source.length + ' chars)'
  return source.startsWith('https://') || source.startsWith('http://') ? 'remote URL' : 'unknown source'
}

type AppIconProps = {
  name: string
  packageId?: string
  icon?: string
  iconCandidates?: string[]
  imageClassName: string
  fallbackClassName: string
}

/**
 * Detect local images that will be effectively invisible on the light icon surface.
 * External SVGs are sanitized and converted to data URLs before reaching the image element.
 */
export function isVisuallyBlankLocalImage(image: HTMLImageElement): boolean {
  const source = image.currentSrc || image.src
  if (!source.startsWith('data:image/')) return true

  const canvas = document.createElement('canvas')
  canvas.width = 24
  canvas.height = 24
  const context = canvas.getContext('2d', { willReadFrequently: true })
  if (!context) return true

  try {
    context.drawImage(image, 0, 0, canvas.width, canvas.height)
    return isVisuallyBlankPixels(context.getImageData(0, 0, canvas.width, canvas.height).data)
  } catch {
    // A candidate that cannot be inspected should not hide the generic icon behind it.
    return true
  }
}

/** Renders a local icon first and resolves a branded external icon only as a fallback. */
function AppIcon({
  name,
  packageId = name,
  icon,
  iconCandidates,
  imageClassName,
  fallbackClassName,
}: AppIconProps) {
  const candidates = collectIconCandidates(icon, iconCandidates)
  const candidateKey = JSON.stringify(candidates)
  const [failedCandidate, setFailedCandidate] = useState<{ key: string; index: number } | null>(null)
  const [externalResult, setExternalResult] = useState<{ key: string; src: string | null; done: boolean } | null>(null)
  const containerRef = useRef<HTMLSpanElement>(null)

  // Reset the effective index when the package/candidate list changes.
  const candidateIndex = failedCandidate?.key === candidateKey ? failedCandidate.index : 0
  const activeIcon = getActiveIconCandidate(candidates, candidateIndex)
  const lookupKey = packageId + '|' + name
  const externalSrc = externalResult?.key === lookupKey ? externalResult.src : null
  const externalComplete = externalResult?.key === lookupKey && externalResult.done
  const displayedIcon = activeIcon ?? externalSrc

  useEffect(() => {
    if (activeIcon || externalComplete) return
    const container = containerRef.current
    if (!container) return

    let cancelled = false
    let started = false
    const startLookup = () => {
      if (started) return
      started = true
      void findExternalAppIcon(packageId, name).then((src) => {
        if (!cancelled) setExternalResult({ key: lookupKey, src, done: true })
      })
    }

    // Do not spend requests on the whole catalog at once; only resolve icons near the viewport.
    if (typeof IntersectionObserver === 'undefined') {
      startLookup()
    } else {
      const observer = new IntersectionObserver((entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
          observer.disconnect()
          startLookup()
        }
      }, { rootMargin: '120px' })
      observer.observe(container)
      return () => {
        cancelled = true
        observer.disconnect()
      }
    }

    return () => {
      cancelled = true
    }
  }, [activeIcon, externalComplete, lookupKey, name, packageId])

  useEffect(() => {
    if (!activeIcon && !externalSrc) {
      reportIconDiagnostic(name + ': no local icon candidate; showing generic fallback while checking external source')
    }
  }, [activeIcon, externalSrc, name])

  const tryNextCandidate = () => {
    setFailedCandidate((current) => ({
      key: candidateKey,
      index: nextCandidateIndex(current?.key === candidateKey ? current.index : 0, candidates.length),
    }))
  }

  return (
    <span ref={containerRef} className="relative inline-grid place-items-center">
      {!displayedIcon ? (
        <Package aria-hidden="true" className={fallbackClassName} />
      ) : (
        <>
          {/* Keep a visible generic icon underneath images that fail or are nearly blank. */}
          <Package
            aria-hidden="true"
            className={'absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 ' + fallbackClassName}
          />
          <img
            src={displayedIcon}
            alt=""
            aria-label={name + ' icon'}
            className={imageClassName}
            loading="lazy"
            onLoad={(event) => {
              const blank = isVisuallyBlankLocalImage(event.currentTarget)
              reportIconDiagnostic(
                name + ': loaded candidate ' + (activeIcon ? candidateIndex + 1 : 'external') +
                '; source=' + describeIconSource(displayedIcon) + '; visuallyBlank=' + blank,
              )
              if (blank) {
                if (activeIcon) tryNextCandidate()
                else setExternalResult({ key: lookupKey, src: null, done: true })
              }
            }}
            onError={() => {
              reportIconDiagnostic(
                name + ': failed candidate ' + (activeIcon ? candidateIndex + 1 : 'external') +
                '; source=' + describeIconSource(displayedIcon),
              )
              if (activeIcon) tryNextCandidate()
              else setExternalResult({ key: lookupKey, src: null, done: true })
            }}
          />
        </>
      )}
    </span>
  )
}

export default AppIcon
