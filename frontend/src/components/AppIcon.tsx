import { useState } from 'react'
import { Package } from 'lucide-react'

type AppIconProps = {
  name: string
  icon?: string
  iconCandidates?: string[]
  imageClassName: string
  fallbackClassName: string
}

/**
 * Detect empty or practically invisible local images.
 *
 * A few non-transparent pixels can occur in broken/placeholder assets. The
 * threshold is intentionally tiny so legitimate icons with transparent
 * padding are retained.
 */
function isBlankLocalImage(image: HTMLImageElement): boolean {
  if (!image.currentSrc.startsWith('data:image/')) return false

  const canvas = document.createElement('canvas')
  canvas.width = 24
  canvas.height = 24

  const context = canvas.getContext('2d', { willReadFrequently: true })
  if (!context) return false

  try {
    context.drawImage(image, 0, 0, canvas.width, canvas.height)
    const pixels = context.getImageData(0, 0, canvas.width, canvas.height).data
    let visiblePixels = 0

    for (let alphaIndex = 3; alphaIndex < pixels.length; alphaIndex += 4) {
      if (pixels[alphaIndex] > 16) {
        visiblePixels += 1
        if (visiblePixels >= 4) return false
      }
    }

    return true
  } catch {
    // Cross-origin remote images may disallow pixel inspection.
    return false
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
  const candidates = Array.from(
    new Set([...(icon ? [icon] : []), ...(iconCandidates ?? [])]),
  )
  const candidateKey = JSON.stringify(candidates)
  const [failedCandidate, setFailedCandidate] = useState<{
    key: string
    index: number
  } | null>(null)

  // Reset the effective index when the package/candidate list changes.
  const candidateIndex = failedCandidate?.key === candidateKey ? failedCandidate.index : 0
  const activeIcon = candidates[candidateIndex]

  if (!activeIcon) {
    return <Package aria-hidden="true" className={fallbackClassName} />
  }

  const tryNextCandidate = () => {
    setFailedCandidate((current) => ({
      key: candidateKey,
      index: (current?.key === candidateKey ? current.index : 0) + 1,
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
          if (isBlankLocalImage(event.currentTarget)) tryNextCandidate()
        }}
        onError={tryNextCandidate}
      />
    </span>
  )
}

export default AppIcon
