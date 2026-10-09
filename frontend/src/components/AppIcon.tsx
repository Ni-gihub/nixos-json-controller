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
 * Detect local images that will be effectively invisible on the light icon surface.
 *
 * Alpha alone is not sufficient: a mostly-empty image, or an opaque white
 * placeholder, can load successfully while looking blank in the App Store.
 * Remote images are not inspected because canvas access may be blocked by CORS.
 */
function isVisuallyBlankLocalImage(image: HTMLImageElement): boolean {
  if (!image.currentSrc.startsWith('data:image/')) return false

  const canvas = document.createElement('canvas')
  canvas.width = 24
  canvas.height = 24

  const context = canvas.getContext('2d', { willReadFrequently: true })
  if (!context) return false

  try {
    context.drawImage(image, 0, 0, canvas.width, canvas.height)
    const pixels = context.getImageData(0, 0, canvas.width, canvas.height).data

    // Approximate the light muted background behind App Store icons.
    const background = [247, 248, 250] as const
    const minimumContrast = 22
    const minimumVisiblePixels = 8
    let contrastingPixels = 0

    for (let offset = 0; offset < pixels.length; offset += 4) {
      const alpha = pixels[offset + 3] / 255
      if (alpha < 0.08) continue

      const red = pixels[offset] * alpha + background[0] * (1 - alpha)
      const green = pixels[offset + 1] * alpha + background[1] * (1 - alpha)
      const blue = pixels[offset + 2] * alpha + background[2] * (1 - alpha)
      const contrast = Math.max(
        Math.abs(red - background[0]),
        Math.abs(green - background[1]),
        Math.abs(blue - background[2]),
      )

      if (contrast >= minimumContrast) {
        contrastingPixels += 1
        if (contrastingPixels >= minimumVisiblePixels) return false
      }
    }

    return true
  } catch {
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
          if (isVisuallyBlankLocalImage(event.currentTarget)) tryNextCandidate()
        }}
        onError={tryNextCandidate}
      />
    </span>
  )
}

export default AppIcon
