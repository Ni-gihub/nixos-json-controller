import { useState } from 'react'
import { Package } from 'lucide-react'

type AppIconProps = {
  name: string
  icon?: string
  imageClassName: string
  fallbackClassName: string
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
      onError={() => setFailedIcon(icon)}
    />
  )
}

export default AppIcon
