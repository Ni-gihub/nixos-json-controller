import { useState } from 'react'
import { Package } from 'lucide-react'

type AppIconProps = {
  name: string
  icon?: string
  imageClassName: string
  fallbackClassName: string
}

/**
 * Some local AppStream images can decode successfully while containing only
 * fully transparent pixels. Browsers do not fire onError for those files, so
 * check local data URLs and let the generic package icon take over.
 *
 * Remote images are not inspected because canvas access can be blocked by CORS.
 */
function isFullyTransparentLocalImage(image: HTMLImageElement): boolean {
  if (!image.currentSrc.startsWith('data:image/')) return false

  const canvas = document.createElement('canvas')
  canvas.width = 24
  canvas.height = 24

  const context = canvas.getContext('2d', { willReadFrequently: true })
  if (!context) return false

  try {
    context.drawImage(image, 0, 0, canvas.width, canvas.height)
    const pixels = context.getImageData(0, 0, canvas.width, canvas.height).data

    for (let alphaIndex = 3; alphaIndex < pixels.length; alphaIndex += 4) {
      if (pixels[alphaIndex] !== 0) return false
    }

    return true
  } catch {
    // Do not reject an image just because the browser disallows pixel inspection.
    return false
  }
}

function AppIcon({ name, icon, imageClassName, fallbackClassName }: AppIconProps) {
  const [failedIcon, setFailedIcon] = useState<string | undefined>()
  const iconFailed = Boolean(icon && failedIcon === icon)

  if (!icon || iconFailed) {
    return <Package aria-hidden="true" className={fallbackClassName} />
  }

  return (
    <img
      src={icon}
      alt=""
      aria-label={`${name} icon`}
      className={imageClassName}
      loading="lazy"
      onLoad={(event) => {
        if (isFullyTransparentLocalImage(event.currentTarget)) {
          setFailedIcon(icon)
        }
      }}
      onError={() => setFailedIcon(icon)}
    />
  )
}

export default AppIcon
