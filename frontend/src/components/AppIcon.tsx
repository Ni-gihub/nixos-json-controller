import { useEffect, useState } from 'react'
import { Package } from 'lucide-react'

type AppIconProps = {
  name: string
  icon?: string
  imageClassName: string
  fallbackClassName: string
}

function AppIcon({ name, icon, imageClassName, fallbackClassName }: AppIconProps) {
  const [failedIcon, setFailedIcon] = useState<string | undefined>()

  useEffect(() => {
    setFailedIcon(undefined)
  }, [icon])

  if (!icon || failedIcon === icon) {
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
