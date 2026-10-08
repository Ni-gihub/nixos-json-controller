import { useState } from 'react'
import { Package } from 'lucide-react'

type AppIconProps = {
  name: string
  icon?: string
  imageClassName: string
  fallbackClassName: string
}

function AppIcon({ name, icon, imageClassName, fallbackClassName }: AppIconProps) {
  const [failedSource, setFailedSource] = useState<{ icon: string; failed: boolean } | undefined>()
  const iconFailed = Boolean(icon && failedSource?.icon === icon && failedSource.failed)

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
      onError={() => setFailedSource({ icon, failed: true })}
    />
  )
}

export default AppIcon
