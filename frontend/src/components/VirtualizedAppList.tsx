import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import AppCard from "@/components/AppCard";
import type { App } from "@/catalog/apps";
import { buildRowOffsets, getVirtualRange } from "@/catalog/virtual-list";
import type { AppState } from "@/lib/app-state";

const ESTIMATED_ROW_HEIGHT = 144;
const OVERSCAN_ROWS = 8;

type VirtualizedAppListProps = {
  apps: App[];
  states: Record<string, AppState>;
  loading: boolean;
  searchQuery: string;
  onVisiblePackages: (packageNames: string[]) => void;
};

/**
 * Render only rows near the window viewport while preserving the full list height.
 */
export default function VirtualizedAppList({
  apps,
  states,
  loading,
  searchQuery,
  onVisiblePackages,
}: VirtualizedAppListProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [measuredHeights, setMeasuredHeights] = useState(
    () => new Map<string, number>(),
  );
  const [viewport, setViewport] = useState(() => ({
    scrollY: typeof window === "undefined" ? 0 : window.scrollY,
    height: typeof window === "undefined" ? 0 : window.innerHeight,
  }));
  const [listTop, setListTop] = useState(0);
  const setContainerRef = useCallback((container: HTMLDivElement | null) => {
    containerRef.current = container;
    if (!container) return;

    const nextTop = container.getBoundingClientRect().top + window.scrollY;
    setListTop((current) => (current === nextTop ? current : nextTop));
  }, []);

  useEffect(() => {
    const updateViewport = () => {
      setViewport((current) => {
        const next = { scrollY: window.scrollY, height: window.innerHeight };
        return current.scrollY === next.scrollY &&
          current.height === next.height
          ? current
          : next;
      });

      const container = containerRef.current;
      const nextTop = container
        ? container.getBoundingClientRect().top + window.scrollY
        : 0;
      setListTop((current) => (current === nextTop ? current : nextTop));
    };

    window.addEventListener("scroll", updateViewport, { passive: true });
    window.addEventListener("resize", updateViewport);
    return () => {
      window.removeEventListener("scroll", updateViewport);
      window.removeEventListener("resize", updateViewport);
    };
  }, []);

  const offsets = useMemo(
    () =>
      buildRowOffsets(
        apps.map((app) => app.id),
        measuredHeights,
        ESTIMATED_ROW_HEIGHT,
      ),
    [apps, measuredHeights],
  );
  const range = useMemo(
    () =>
      getVirtualRange(
        offsets,
        viewport.scrollY - listTop,
        viewport.scrollY + viewport.height - listTop,
        OVERSCAN_ROWS,
      ),
    [listTop, offsets, viewport.height, viewport.scrollY],
  );

  useEffect(() => {
    const root = containerRef.current;
    if (!root || typeof ResizeObserver === "undefined") return;

    const observer = new ResizeObserver((entries) => {
      setMeasuredHeights((current) => {
        let next = current;
        for (const entry of entries) {
          const element = entry.target as HTMLElement;
          const id = element.dataset.virtualAppId;
          if (!id) continue;

          const height = element.getBoundingClientRect().height;
          if (!Number.isFinite(height) || height <= 0) continue;

          const previous = current.get(id);
          if (previous === undefined || Math.abs(previous - height) > 1) {
            if (next === current) next = new Map(current);
            next.set(id, height);
          }
        }
        return next;
      });
    });

    root
      .querySelectorAll<HTMLElement>("[data-virtual-app-id]")
      .forEach((element) => observer.observe(element));
    return () => observer.disconnect();
  }, [apps, range.endIndex, range.startIndex]);

  useEffect(() => {
    if (range.endIndex <= range.startIndex) return;
    const visibleIds = apps
      .slice(range.startIndex, range.endIndex)
      .map((app) => app.id);
    onVisiblePackages(visibleIds);
  }, [apps, onVisiblePackages, range.endIndex, range.startIndex]);

  const renderRow = useCallback(
    (app: App) => {
      const state = states[app.id];
      const active = state?.active === true;
      const statusKnown = active || state?.active === false;
      return (
        <div key={app.id} data-virtual-app-id={app.id} className="pb-2">
          <AppCard
            id={app.id}
            name={app.name}
            description={app.description}
            category={app.category}
            tags={app.tags}
            installed={active}
            statusKnown={statusKnown}
            statusLoading={loading && state === undefined}
            icon={app.icon}
            iconCandidates={app.iconCandidates}
            searchQuery={searchQuery}
          />
        </div>
      );
    },
    [loading, searchQuery, states],
  );

  return (
    <div ref={setContainerRef} className="w-full">
      <div aria-hidden="true" style={{ height: range.topOffset }} />
      {apps.slice(range.startIndex, range.endIndex).map(renderRow)}
      <div aria-hidden="true" style={{ height: range.bottomOffset }} />
    </div>
  );
}
