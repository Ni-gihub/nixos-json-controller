import type { App } from "./apps";

type FallbackMetadata = Pick<App, "name" | "description" | "category" | "tags">;

/**
 * Minimal metadata for the updater's curated package map.
 * These entries keep the recommendation UI useful if local Nix evaluation fails;
 * package availability is still verified by Nix when the user searches or installs.
 */
const FALLBACK_METADATA: Record<string, FallbackMetadata> = {
  firefox: {
    name: "Firefox",
    description: "プライバシー機能を備えたWebブラウザー",
    category: "ブラウザ",
    tags: ["ブラウザ", "Web"],
  },
  chromium: {
    name: "Chromium",
    description: "オープンソースのWebブラウザー",
    category: "ブラウザ",
    tags: ["ブラウザ", "Web"],
  },
  epiphany: {
    name: "GNOME Web",
    description: "GNOMEデスクトップ向けのWebブラウザー",
    category: "ブラウザ",
    tags: ["ブラウザ", "GNOME"],
  },
  "kdePackages.konqueror": {
    name: "Konqueror",
    description: "Webブラウザー兼ファイルマネージャー",
    category: "ブラウザ",
    tags: ["ブラウザ", "KDE"],
  },
  thunderbird: {
    name: "Thunderbird",
    description: "メール、カレンダー、連絡先を管理するアプリ",
    category: "コミュニケーション",
    tags: ["メール", "Mozilla"],
  },
  gimp: {
    name: "GIMP",
    description: "写真や画像を編集するためのグラフィックスアプリ",
    category: "グラフィックス",
    tags: ["画像編集", "オープンソース"],
  },
  inkscape: {
    name: "Inkscape",
    description: "ベクター画像を作成・編集するアプリ",
    category: "グラフィックス",
    tags: ["ベクター画像", "オープンソース"],
  },
  blender: {
    name: "Blender",
    description: "3Dモデリング、アニメーション、レンダリングツール",
    category: "グラフィックス",
    tags: ["3D", "制作"],
  },
  krita: {
    name: "Krita",
    description: "デジタルペイントやイラスト制作のためのアプリ",
    category: "グラフィックス",
    tags: ["ペイント", "イラスト"],
  },
  "obs-studio": {
    name: "OBS Studio",
    description: "画面録画とライブ配信のためのアプリ",
    category: "メディア",
    tags: ["配信", "録画"],
  },
  vlc: {
    name: "VLC media player",
    description: "さまざまな形式の音声・動画を再生するメディアプレーヤー",
    category: "メディア",
    tags: ["動画", "音楽"],
  },
  mpv: {
    name: "mpv",
    description: "軽量で幅広い形式に対応したメディアプレーヤー",
    category: "メディア",
    tags: ["動画", "音楽"],
  },
  audacity: {
    name: "Audacity",
    description: "音声の録音と編集を行うアプリ",
    category: "メディア",
    tags: ["音声編集", "録音"],
  },
  "kdePackages.kdenlive": {
    name: "Kdenlive",
    description: "動画編集アプリ",
    category: "メディア",
    tags: ["動画編集", "KDE"],
  },
  discord: {
    name: "Discord",
    description: "音声・テキストチャットのコミュニケーションアプリ",
    category: "コミュニケーション",
    tags: ["チャット", "音声"],
  },
  "telegram-desktop": {
    name: "Telegram Desktop",
    description: "メッセージングアプリ",
    category: "コミュニケーション",
    tags: ["チャット", "メッセージ"],
  },
  "signal-desktop": {
    name: "Signal",
    description: "プライバシー重視のメッセージングアプリ",
    category: "コミュニケーション",
    tags: ["チャット", "プライバシー"],
  },
  qbittorrent: {
    name: "qBittorrent",
    description: "BitTorrentダウンロードクライアント",
    category: "ネットワーク",
    tags: ["ダウンロード", "ネットワーク"],
  },
  steam: {
    name: "Steam",
    description: "PCゲームの管理・起動プラットフォーム",
    category: "ゲーム",
    tags: ["ゲーム", "PC"],
  },
  lutris: {
    name: "Lutris",
    description: "ゲームのインストールと管理を支援するアプリ",
    category: "ゲーム",
    tags: ["ゲーム", "管理"],
  },
  vscode: {
    name: "Visual Studio Code",
    description: "拡張機能に対応したソースコードエディター",
    category: "開発",
    tags: ["エディター", "開発"],
  },
  vscodium: {
    name: "VSCodium",
    description: "コミュニティ版のVisual Studio Codeビルド",
    category: "開発",
    tags: ["エディター", "開発"],
  },
  kitty: {
    name: "Kitty",
    description: "GPUアクセラレーション対応のターミナルエミュレーター",
    category: "ターミナル",
    tags: ["ターミナル", "開発"],
  },
  alacritty: {
    name: "Alacritty",
    description: "GPUアクセラレーション対応のターミナルエミュレーター",
    category: "ターミナル",
    tags: ["ターミナル", "開発"],
  },
  wezterm: {
    name: "WezTerm",
    description: "カスタマイズ可能なターミナルエミュレーター",
    category: "ターミナル",
    tags: ["ターミナル", "開発"],
  },
  ghostty: {
    name: "Ghostty",
    description: "ネイティブ機能を重視したターミナルエミュレーター",
    category: "ターミナル",
    tags: ["ターミナル", "開発"],
  },
  keepassxc: {
    name: "KeePassXC",
    description: "ローカルでパスワードを管理するアプリ",
    category: "ユーティリティ",
    tags: ["パスワード", "セキュリティ"],
  },
  obsidian: {
    name: "Obsidian",
    description: "ローカルのノートをリンクして管理するアプリ",
    category: "ユーティリティ",
    tags: ["ノート", "知識管理"],
  },
  "kdePackages.okular": {
    name: "Okular",
    description: "PDFなどの文書を閲覧するアプリ",
    category: "ドキュメント",
    tags: ["PDF", "KDE"],
  },
  evince: {
    name: "Evince",
    description: "PDFなどの文書を閲覧するアプリ",
    category: "ドキュメント",
    tags: ["PDF", "GNOME"],
  },
  gparted: {
    name: "GParted",
    description: "ディスクのパーティションを管理するツール",
    category: "システム",
    tags: ["ディスク", "管理"],
  },
  filezilla: {
    name: "FileZilla",
    description: "FTPなどでファイルを転送するクライアント",
    category: "ネットワーク",
    tags: ["ファイル転送", "FTP"],
  },
  pavucontrol: {
    name: "PulseAudio Volume Control",
    description: "音声デバイスと音量を管理するアプリ",
    category: "ユーティリティ",
    tags: ["音声", "設定"],
  },
  "gnome-calculator": {
    name: "GNOME Calculator",
    description: "日常計算や科学計算のための電卓",
    category: "ユーティリティ",
    tags: ["電卓", "GNOME"],
  },
  darktable: {
    name: "darktable",
    description: "RAW写真の現像と画像管理を行うアプリ",
    category: "グラフィックス",
    tags: ["写真", "現像"],
  },
  rawtherapee: {
    name: "RawTherapee",
    description: "RAW写真を現像するアプリ",
    category: "グラフィックス",
    tags: ["写真", "現像"],
  },
  remmina: {
    name: "Remmina",
    description: "リモートデスクトップ接続クライアント",
    category: "ネットワーク",
    tags: ["リモート", "接続"],
  },
  "virt-manager": {
    name: "Virtual Machine Manager",
    description: "仮想マシンを作成・管理するアプリ",
    category: "システム",
    tags: ["仮想化", "管理"],
  },,
  nautilus: {
    name: "Files",
    description: "GNOMEデスクトップのファイルマネージャー",
    category: "ファイル管理",
    tags: ["ファイル管理", "GNOME"],
  },
  thunar: {
    name: "Thunar",
    description: "軽量なファイルマネージャー",
    category: "ファイル管理",
    tags: ["ファイル管理", "軽量"],
  },
  "kdePackages.dolphin": {
    name: "Dolphin",
    description: "KDEデスクトップのファイルマネージャー",
    category: "ファイル管理",
    tags: ["ファイル管理", "KDE"],
  },
  "kdePackages.ark": {
    name: "Ark",
    description: "圧縮ファイルを作成・展開するアーカイブ管理アプリ",
    category: "ファイル管理",
    tags: ["アーカイブ", "KDE"],
  },
  "file-roller": {
    name: "File Roller",
    description: "圧縮ファイルを作成・展開するアーカイブ管理アプリ",
    category: "ファイル管理",
    tags: ["アーカイブ", "GNOME"],
  },
  "libreoffice-qt": {
    name: "LibreOffice",
    description: "文書、表計算、プレゼンテーションを扱うオフィススイート",
    category: "オフィス",
    tags: ["オフィス", "文書作成"],
  },
  calibre: {
    name: "Calibre",
    description: "電子書籍を整理・変換・閲覧するアプリ",
    category: "ドキュメント・ノート",
    tags: ["電子書籍", "文書"],
  },
  zathura: {
    name: "Zathura",
    description: "キーボード操作を重視した文書ビューアー",
    category: "ドキュメント・ノート",
    tags: ["PDF", "文書"],
  },
  evolution: {
    name: "Evolution",
    description: "メール、カレンダー、連絡先を管理するアプリ",
    category: "コミュニケーション",
    tags: ["メール", "カレンダー"],
  },
  geary: {
    name: "Geary",
    description: "会話形式でメールを管理するメールクライアント",
    category: "コミュニケーション",
    tags: ["メール", "GNOME"],
  },
  handbrake: {
    name: "HandBrake",
    description: "動画ファイルを変換するツール",
    category: "メディア",
    tags: ["動画変換", "エンコード"],
  },
  kodi: {
    name: "Kodi",
    description: "動画や音楽を整理・再生するメディアセンター",
    category: "メディア",
    tags: ["動画", "音楽"],
  },
  rhythmbox: {
    name: "Rhythmbox",
    description: "音楽ライブラリを整理して再生するアプリ",
    category: "メディア",
    tags: ["音楽", "GNOME"],
  },
  strawberry: {
    name: "Strawberry",
    description: "音楽コレクションを管理・再生する音楽プレーヤー",
    category: "メディア",
    tags: ["音楽", "プレーヤー"],
  },
  shotcut: {
    name: "Shotcut",
    description: "動画を編集するためのビデオエディター",
    category: "クリエイティブ",
    tags: ["動画編集", "制作"],
  },
  gthumb: {
    name: "gThumb",
    description: "画像を閲覧・整理・編集するアプリ",
    category: "クリエイティブ",
    tags: ["画像", "写真"],
  },
  loupe: {
    name: "Loupe",
    description: "画像をシンプルに閲覧するGNOMEアプリ",
    category: "クリエイティブ",
    tags: ["画像", "GNOME"],
  },
  imv: {
    name: "imv",
    description: "軽量な画像ビューアー",
    category: "クリエイティブ",
    tags: ["画像", "軽量"],
  },
  feh: {
    name: "feh",
    description: "画像表示と背景設定に使える軽量ツール",
    category: "クリエイティブ",
    tags: ["画像", "軽量"],
  },
  neovim: {
    name: "Neovim",
    description: "拡張性の高いモーダル型テキストエディター",
    category: "開発ツール",
    tags: ["エディター", "開発"],
  },
  vim: {
    name: "Vim",
    description: "キーボード操作を重視したテキストエディター",
    category: "開発ツール",
    tags: ["エディター", "開発"],
  },
  emacs: {
    name: "GNU Emacs",
    description: "拡張可能なテキストエディター兼作業環境",
    category: "開発ツール",
    tags: ["エディター", "開発"],
  },
  geany: {
    name: "Geany",
    description: "軽量な統合開発環境",
    category: "開発ツール",
    tags: ["IDE", "開発"],
  },
  helix: {
    name: "Helix",
    description: "モーダル編集とLSPを備えたターミナルエディター",
    category: "開発ツール",
    tags: ["エディター", "開発"],
  },
  git: {
    name: "Git",
    description: "ソースコードの変更履歴を管理するバージョン管理ツール",
    category: "開発ツール",
    tags: ["バージョン管理", "開発"],
  },
  podman: {
    name: "Podman",
    description: "コンテナを作成・実行・管理するツール",
    category: "開発ツール",
    tags: ["コンテナ", "開発"],
  },
  docker: {
    name: "Docker",
    description: "コンテナの構築と実行を支援するプラットフォーム",
    category: "開発ツール",
    tags: ["コンテナ", "開発"],
  },
  sqlitebrowser: {
    name: "DB Browser for SQLite",
    description: "SQLiteデータベースをGUIで閲覧・編集するツール",
    category: "開発ツール",
    tags: ["データベース", "開発"],
  },
  foot: {
    name: "foot",
    description: "Wayland向けの軽量ターミナルエミュレーター",
    category: "ターミナル",
    tags: ["ターミナル", "Wayland"],
  },
  tmux: {
    name: "tmux",
    description: "複数のターミナルセッションを管理するツール",
    category: "ターミナル",
    tags: ["ターミナル", "CLI"],
  },
  zellij: {
    name: "Zellij",
    description: "ターミナル内のウィンドウとペインを管理するツール",
    category: "ターミナル",
    tags: ["ターミナル", "CLI"],
  },
  btop: {
    name: "btop",
    description: "CPU、メモリ、ディスクなどの使用状況を表示するツール",
    category: "システム管理",
    tags: ["監視", "システム"],
  },
  "gnome-disk-utility": {
    name: "Disks",
    description: "ストレージデバイスやディスクイメージを管理するアプリ",
    category: "システム管理",
    tags: ["ディスク", "GNOME"],
  },
  baobab: {
    name: "Disk Usage Analyzer",
    description: "ディスク使用量を視覚的に確認するアプリ",
    category: "システム管理",
    tags: ["ディスク", "GNOME"],
  },
  "kdePackages.filelight": {
    name: "Filelight",
    description: "ディスク使用量を円形グラフで可視化するアプリ",
    category: "システム管理",
    tags: ["ディスク", "KDE"],
  },
  "gnome-system-monitor": {
    name: "System Monitor",
    description: "プロセスやシステムリソースの使用状況を確認するアプリ",
    category: "システム管理",
    tags: ["監視", "GNOME"],
  },
};

/** Build display-only cards for curated IDs when Nix cannot resolve catalog metadata. */
export function buildRecommendationFallbackApps(ids: string[]): App[] {
  return ids.flatMap((id) => {
    const metadata = FALLBACK_METADATA[id];
    if (!metadata) return [];

    return [
      {
        id,
        ...metadata,
        version: "",
        iconCandidates: [],
      },
    ];
  });
}
