export type App = {
  id: string
  name: string
  description: string
  category: string
  details: string
}

export const apps: App[] = [
  {
    id: 'firefox',
    name: 'Firefox',
    description: 'Fast, private web browser',
    category: 'Browser',
    details: 'A fast and privacy-focused web browser from Mozilla.',
  },
  {
    id: 'vlc',
    name: 'VLC',
    description: 'Play almost any media format',
    category: 'Media',
    details:
      'A versatile media player that supports a wide range of audio and video formats.',
  },
  {
    id: 'gimp',
    name: 'GIMP',
    description: 'Powerful open-source image editor',
    category: 'Graphics',
    details:
      'A powerful open-source image editor for photo manipulation and graphic design.',
  },
]
