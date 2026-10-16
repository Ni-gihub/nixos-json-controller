import { useCallback, useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";

type RecommendationCarouselProps = {
  label: string;
  resetKey: string;
  children: ReactNode;
};

/** Horizontal ranking rail with subtle navigation hints and a visible next-card peek. */
export default function RecommendationCarousel({
  label,
  resetKey,
  children,
}: RecommendationCarouselProps) {
  const viewportRef = useRef<HTMLDivElement>(null);
  const [canScrollLeft, setCanScrollLeft] = useState(false);
  const [canScrollRight, setCanScrollRight] = useState(false);

  const updateScrollState = useCallback(() => {
    const viewport = viewportRef.current;
    if (!viewport) return;

    const maxScrollLeft = viewport.scrollWidth - viewport.clientWidth;
    setCanScrollLeft(viewport.scrollLeft > 2);
    setCanScrollRight(
      maxScrollLeft > 2 && viewport.scrollLeft < maxScrollLeft - 2,
    );
  }, []);

  useEffect(() => {
    const viewport = viewportRef.current;
    if (!viewport) return;

    // Keep each genre rail anchored at rank 1 when its catalog data changes.
    viewport.scrollLeft = 0;
    updateScrollState();
    viewport.addEventListener("scroll", updateScrollState, { passive: true });
    const resizeObserver = new ResizeObserver(updateScrollState);
    resizeObserver.observe(viewport);
    if (viewport.firstElementChild)
      resizeObserver.observe(viewport.firstElementChild);

    return () => {
      viewport.removeEventListener("scroll", updateScrollState);
      resizeObserver.disconnect();
    };
  }, [resetKey, updateScrollState]);

  const scroll = (direction: -1 | 1) => {
    const viewport = viewportRef.current;
    if (!viewport) return;

    viewport.scrollBy({
      left: direction * Math.max(220, Math.round(viewport.clientWidth * 0.72)),
      behavior: window.matchMedia("(prefers-reduced-motion: reduce)").matches
        ? "auto"
        : "smooth",
    });
  };

  return (
    <div className="relative min-w-0">
      <div
        ref={viewportRef}
        role="region"
        aria-label={`${label}。横方向にスクロールできます`}
        tabIndex={0}
        className="nxc-carousel-track flex snap-x snap-mandatory gap-4 overflow-x-auto pb-3 pr-14 outline-none focus-visible:rounded-xl focus-visible:ring-2 focus-visible:ring-primary/40"
      >
        {children}
      </div>

      {canScrollLeft && (
        <>
          <div
            aria-hidden="true"
            className="pointer-events-none absolute inset-y-0 left-0 z-[1] w-14 bg-gradient-to-r from-background via-background/65 to-transparent"
          />
          <button
            type="button"
            aria-label={`${label}を左にスクロール`}
            onClick={() => scroll(-1)}
            className="absolute left-1 top-1/2 z-10 flex size-9 -translate-y-1/2 items-center justify-center rounded-full border border-border/70 bg-background/80 text-foreground/55 opacity-65 shadow-sm backdrop-blur-sm transition hover:bg-background/95 hover:text-primary hover:opacity-100 focus-visible:opacity-100"
          >
            <ChevronLeft className="size-5" />
          </button>
        </>
      )}

      {canScrollRight && (
        <>
          <div
            aria-hidden="true"
            className="pointer-events-none absolute inset-y-0 right-0 z-[1] w-16 bg-gradient-to-l from-background via-background/70 to-transparent"
          />
          <button
            type="button"
            aria-label={`${label}を右にスクロール`}
            onClick={() => scroll(1)}
            className="absolute right-1 top-1/2 z-10 flex size-9 -translate-y-1/2 items-center justify-center rounded-full border border-border/70 bg-background/80 text-foreground/55 opacity-65 shadow-sm backdrop-blur-sm transition hover:bg-background/95 hover:text-primary hover:opacity-100 focus-visible:opacity-100"
          >
            <ChevronRight className="size-5" />
          </button>
        </>
      )}
    </div>
  );
}
