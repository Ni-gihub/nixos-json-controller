import { useMemo } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, Store, X } from "lucide-react";

/** Frameless-window titlebar styled to blend into the nxc App Store UI. */
export default function WindowTitleBar() {
  const appWindow = useMemo(() => (isTauri() ? getCurrentWindow() : null), []);
  // Browser previews keep the branded bar; native window controls are Tauri-only.
  const isDesktopApp = appWindow !== null;

  return (
    <header
      data-tauri-drag-region
      className="sticky top-0 z-50 flex h-10 shrink-0 items-center gap-2 border-b border-border bg-background px-3 text-foreground"
    >
      <span
        data-tauri-drag-region
        className="flex size-6 shrink-0 items-center justify-center rounded-md bg-[#007aff] text-white"
      >
        <Store className="size-3.5" />
      </span>
      <span
        data-tauri-drag-region
        className="select-none text-xs font-semibold tracking-tight"
      >
        nxc App Store
      </span>
      <span data-tauri-drag-region className="min-w-0 flex-1 self-stretch" />

      {isDesktopApp && appWindow && (
        <div className="flex h-full shrink-0 items-center gap-0.5">
          <button
            type="button"
            aria-label="最小化"
            title="最小化"
            onClick={() => void appWindow.minimize().catch(console.error)}
            className="flex size-8 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#007aff]/40"
          >
            <Minus className="size-4" />
          </button>
          <button
            type="button"
            aria-label="最大化／元に戻す"
            title="最大化／元に戻す"
            onClick={() => void appWindow.toggleMaximize().catch(console.error)}
            className="flex size-8 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#007aff]/40"
          >
            <Square className="size-3.5" />
          </button>
          <button
            type="button"
            aria-label="閉じる"
            title="閉じる"
            onClick={() => void appWindow.close().catch(console.error)}
            className="flex size-8 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-red-500 hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-500/40"
          >
            <X className="size-4" />
          </button>
        </div>
      )}
    </header>
  );
}
