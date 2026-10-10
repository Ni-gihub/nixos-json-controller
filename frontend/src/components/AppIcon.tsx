import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Package } from 'lucide-react'
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
  if (source.startsWith('data:image/png;base64,')) return `PNG (${source.length} chars)`
  if (source.startsWith('data:image/svg+xml;base64,')) return `SVG (${source.length} chars)`
  if (source.startsWith('data:image/jpeg;base64,')) return `JPEG (${source.length} chars)`
  if (source.startsWith('data:')) return `data URL (${source.length} chars)`
  return source.startsWith('https://') || source.startsWith('http://')
    ? 'remote URL'
    : 'unknown source'
}

type AppIconProps = {
  name: string
  icon?: string
  iconCandidates?: string[]
  imageClassName: string
  fallbackClassName: string
}

/**
 * Detect local images that will be effectively invisible on the light icon surface.
 *
 * Alpha alone is not sufficient: a mostly-empty image, or an opaque white
 * placeholder, can load successfully while looking blank in the App Store.
 * Remote images are not inspected because canvas access may be blocked by CORS.
 */
export function isVisuallyBlankLocalImage(image: HTMLImageElement): boolean {
  // Catalog icons must be local data URLs. An unexpected source is never trusted
  // because it can be opaque to pixel inspection while covering the fallback.
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
    // Fail closed: if WebKit cannot inspect the image (for example, an SVG with
    // external references), try another candidate instead of keeping a blank overlay.
    return true
  }
}

/** Renders an app icon, trying another source when a local image is blank. */
function AppIcon({
  name,
  icon,
  iconCandidates,
  imageClassName,
  fallbackClassName,
}: AppIconProps) {
  const candidates = collectIconCandidates(icon, iconCandidates)
  const candidateKey = JSON.stringify(candidates)
  const [failedCandidate, setFailedCandidate] = useState<{
    key: string
    index: number
  } | null>(null)

  // Reset the effective index when the package/candidate list changes.
  const candidateIndex = failedCandidate?.key === candidateKey ? failedCandidate.index : 0
  const activeIcon = getActiveIconCandidate(candidates, candidateIndex)

  useEffect(() => {
    if (!activeIcon) {
      reportIconDiagnostic(`${name}: no icon candidates; rendering generic Package icon`)
    }
  }, [activeIcon, candidateKey, name])

  if (!activeIcon) {
    return <Package aria-hidden="true" className={fallbackClassName} />
  }

  const tryNextCandidate = () => {
    setFailedCandidate((current) => ({
      key: candidateKey,
      index: nextCandidateIndex(
        current?.key === candidateKey ? current.index : 0,
        candidates.length,
      ),
    }))
  }

  return (
    <span className="relative inline-grid place-items-center">
      {/* Keep a visible generic icon underneath transparent images, including remote URLs. */}
      <Package
        aria-hidden="true"
        className={`absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 ${fallbackClassName}`}
      />
      <img
        src={activeIcon}
        alt=""
        aria-label={`${name} icon`}
        className={imageClassName}
        loading="lazy"
        onLoad={(event) => {
          const blank = isVisuallyBlankLocalImage(event.currentTarget)
          reportIconDiagnostic(
            `${name}: loaded candidate ${candidateIndex + 1}/${candidates.length}; source=${describeIconSource(activeIcon)}; visuallyBlank=${blank}`,
          )
          if (blank) tryNextCandidate()
        }}
        onError={() => {
          reportIconDiagnostic(
            `${name}: failed candidate ${candidateIndex + 1}/${candidates.length}; source=${describeIconSource(activeIcon)}`,
          )
          tryNextCandidate()
        }}
      />
    </span>
  )
}

export default AppIcon
