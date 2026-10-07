export type AppCategory = 'Browser' | 'Development' | 'Media' | 'Graphics'

export type AppTag = string

export type App = {
  id: string
  name: string
  description: string
  category: AppCategory
  tags: AppTag[]
  details: string
  featured?: boolean
}

export const apps: App[] = [
  {
    id: 'firefox',
    name: 'Firefox',
    description: 'Fast, private web browser',
    category: 'Browser',
    tags: ['Open Source', 'Privacy', 'Web Browser'],
    details: 'A fast and privacy-focused web browser from Mozilla.',
    featured: true,
  },
  {
    id: 'vlc',
    name: 'VLC',
    description: 'Play almost any media format',
    category: 'Media',
    tags: ['Open Source', 'Video', 'Audio'],
    details:
      'A versatile media player that supports a wide range of audio and video formats.',
    featured: true,
  },
  {
    id: 'gimp',
    name: 'GIMP',
    description: 'Powerful open-source image editor',
    category: 'Graphics',
    tags: ['Open Source', 'Image Editing', 'Photography'],
    details:
      'A powerful open-source image editor for photo manipulation and graphic design.',
    featured: true,
  },
]
