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
