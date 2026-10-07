export type AppCategory = 'ブラウザ' | '開発' | 'メディア' | 'グラフィックス'

export type AppTag = string

export type App = {
  id: string
  name: string
  description: string
  category: AppCategory
  tags: AppTag[]
  details: string
  featured?: boolean
  icon?: string
}

export const apps: App[] = [
  {
    id: 'firefox',
    name: 'Firefox',
    description: '高速でプライバシーを重視したWebブラウザ',
    category: 'ブラウザ',
    tags: ['オープンソース', 'プライバシー', 'Webブラウザ'],
    details: 'Mozillaが開発する、高速でプライバシーを重視したWebブラウザです。',
    featured: true,
  },
  {
    id: 'vlc',
    name: 'VLC',
    description: 'ほぼすべてのメディア形式を再生できるプレイヤー',
    category: 'メディア',
    tags: ['オープンソース', '動画', '音声'],
    details: '幅広い音声・動画形式に対応した、多機能なオープンソースメディアプレイヤーです。',
    featured: true,
  },
  {
    id: 'gimp',
    name: 'GIMP',
    description: '高機能なオープンソース画像編集ソフト',
    category: 'グラフィックス',
    tags: ['オープンソース', '画像編集', '写真'],
    details: '写真加工やグラフィックデザインに使える、高機能なオープンソース画像編集ソフトです。',
    featured: true,
  },
]
