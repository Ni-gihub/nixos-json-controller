import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Package } from "lucide-react";
import { findExternalAppIcon } from "./external-app-icon";
import {
  collectIconCandidates,
  getActiveIconCandidate,
  isVisuallyBlankLocalImage,
  nextCandidateIndex,
} from "./app-icon-logic";

function reportIconDiagnostic(message: string) {
  if (import.meta.env.VITE_ICON_DEBUG !== "1") return;
  void invoke("log_icon_debug", { message }).catch(() => {
    // The component can also be rendered outside Tauri during frontend development.
  });
}

function describeIconSource(source: string): string {
  if (source.startsWith("data:image/png;base64,"))
    return "PNG (" + source.length + " chars)";
  if (
    source.startsWith("data:image/svg+xml;base64,") ||
    source.startsWith("data:image/svg+xml;charset=utf-8,")
  ) {
    return "SVG (" + source.length + " chars)";
  }
  if (source.startsWith("data:image/jpeg;base64,"))
    return "JPEG (" + source.length + " chars)";
  if (source.startsWith("data:"))
    return "data URL (" + source.length + " chars)";
  return source.startsWith("https://") || source.startsWith("http://")
    ? "remote URL"
    : "unknown source";
}

const DEBUG_FALLBACK_ONLY =
  import.meta.env.VITE_ICON_DEBUG_FALLBACK_ONLY === "1";

function describeRenderedElement(element: Element) {
  const rect = element.getBoundingClientRect();
  const style = window.getComputedStyle(element);

  return {
    tag: element.tagName.toLowerCase(),
    className: element.getAttribute("class"),
    bounds: {
      x: Number(rect.x.toFixed(1)),
      y: Number(rect.y.toFixed(1)),
      width: Number(rect.width.toFixed(1)),
      height: Number(rect.height.toFixed(1)),
    },
    display: style.display,
    visibility: style.visibility,
    opacity: style.opacity,
    color: style.color,
    stroke: style.stroke,
    fill: style.fill,
    backgroundColor: style.backgroundColor,
    overflow: style.overflow,
    position: style.position,
    zIndex: style.zIndex,
  };
}

type AppIconProps = {
  name: string;
  packageId?: string;
  icon?: string;
  iconCandidates?: string[];
  imageClassName: string;
  fallbackClassName: string;
};

/** Renders a local icon first and resolves a branded external icon only as a fallback. */
function AppIcon({
  name,
  packageId = name,
  icon,
  iconCandidates,
  imageClassName,
  fallbackClassName,
}: AppIconProps) {
  // This opt-in mode isolates Lucide/WebView rendering from package and external image sources.
  const candidates = DEBUG_FALLBACK_ONLY
    ? []
    : collectIconCandidates(icon, iconCandidates);
  const candidateKey = JSON.stringify(candidates);
  const [failedCandidate, setFailedCandidate] = useState<{
    key: string;
    index: number;
  } | null>(null);
  const [externalResult, setExternalResult] = useState<{
    key: string;
    src: string | null;
    done: boolean;
  } | null>(null);
  const containerRef = useRef<HTMLSpanElement>(null);
  const packageIconRef = useRef<SVGSVGElement>(null);
  const imageRef = useRef<HTMLImageElement>(null);

  // Reset the effective index when the package/candidate list changes.
  const candidateIndex =
    failedCandidate?.key === candidateKey ? failedCandidate.index : 0;
  const activeIcon = getActiveIconCandidate(candidates, candidateIndex);
  const lookupKey = packageId + "|" + name;
  const externalSrc =
    !DEBUG_FALLBACK_ONLY && externalResult?.key === lookupKey
      ? externalResult.src
      : null;
  const externalComplete =
    !DEBUG_FALLBACK_ONLY &&
    externalResult?.key === lookupKey &&
    externalResult.done;
  const displayedIcon = activeIcon ?? externalSrc;

  useEffect(() => {
    if (DEBUG_FALLBACK_ONLY || activeIcon || externalComplete) return;
    const container = containerRef.current;
    if (!container) return;

    let cancelled = false;
    let started = false;
    const startLookup = () => {
      if (started) return;
      started = true;
      void findExternalAppIcon(packageId, name).then((src) => {
        if (!cancelled) setExternalResult({ key: lookupKey, src, done: true });
      });
    };

    // Do not spend requests on the whole catalog at once; only resolve icons near the viewport.
    if (typeof IntersectionObserver === "undefined") {
      startLookup();
    } else {
      const observer = new IntersectionObserver(
        (entries) => {
          if (entries.some((entry) => entry.isIntersecting)) {
            observer.disconnect();
            startLookup();
          }
        },
        { rootMargin: "120px" },
      );
      observer.observe(container);
      return () => {
        cancelled = true;
        observer.disconnect();
      };
    }

    return () => {
      cancelled = true;
    };
  }, [
    DEBUG_FALLBACK_ONLY,
    activeIcon,
    externalComplete,
    lookupKey,
    name,
    packageId,
  ]);

  useEffect(() => {
    if (import.meta.env.VITE_ICON_DEBUG !== "1") return;

    const container = containerRef.current;
    const svg = packageIconRef.current;
    if (!container || !svg) return;

    const parent = container.parentElement;
    const ancestors = parent
      ? [parent, parent.parentElement].filter(
          (element): element is HTMLElement => element !== null,
        )
      : [];
    const pathStyles = Array.from(svg.querySelectorAll("path"))
      .slice(0, 3)
      .map((path) => {
        const style = window.getComputedStyle(path);
        return {
          stroke: style.stroke,
          fill: style.fill,
          opacity: style.opacity,
          pathLength: path.getAttribute("d")?.length ?? 0,
        };
      });
    const image = imageRef.current;

    reportIconDiagnostic(
      name +
        ": rendered DOM " +
        JSON.stringify({
          fallbackOnly: DEBUG_FALLBACK_ONLY,
          devicePixelRatio: window.devicePixelRatio,
          container: describeRenderedElement(container),
          packageSvg: {
            ...describeRenderedElement(svg),
            widthAttribute: svg.getAttribute("width"),
            heightAttribute: svg.getAttribute("height"),
            viewBox: svg.getAttribute("viewBox"),
            strokeAttribute: svg.getAttribute("stroke"),
            fillAttribute: svg.getAttribute("fill"),
            pathCount: svg.querySelectorAll("path").length,
            samplePaths: pathStyles,
          },
          ancestors: ancestors.map(describeRenderedElement),
          image: image
            ? {
                ...describeRenderedElement(image),
                source: describeIconSource(image.currentSrc || image.src),
                naturalWidth: image.naturalWidth,
                naturalHeight: image.naturalHeight,
              }
            : null,
        }),
    );
  }, [candidateIndex, candidateKey, displayedIcon, name]);

  useEffect(() => {
    if (!activeIcon && !externalSrc) {
      const fallbackMessage = DEBUG_FALLBACK_ONLY
        ? ": forced generic fallback; external lookup disabled"
        : ": no local icon candidate; showing generic fallback while checking external source";
      reportIconDiagnostic(name + fallbackMessage);
    }
  }, [activeIcon, externalSrc, name]);

  const tryNextCandidate = () => {
    setFailedCandidate((current) => ({
      key: candidateKey,
      index: nextCandidateIndex(
        current?.key === candidateKey ? current.index : 0,
        candidates.length,
      ),
    }));
  };

  return (
    <span
      ref={containerRef}
      className="relative inline-grid place-items-center"
    >
      {!displayedIcon ? (
        <Package
          ref={packageIconRef}
          aria-hidden="true"
          className={fallbackClassName}
        />
      ) : (
        <>
          {/* Keep a visible generic icon underneath images that fail or are nearly blank. */}
          <Package
            ref={packageIconRef}
            aria-hidden="true"
            className={
              "absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 " +
              fallbackClassName
            }
          />
          <img
            ref={imageRef}
            src={displayedIcon}
            alt=""
            aria-label={name + " icon"}
            className={imageClassName}
            loading="lazy"
            onLoad={(event) => {
              const blank = isVisuallyBlankLocalImage(event.currentTarget);
              reportIconDiagnostic(
                name +
                  ": loaded candidate " +
                  (activeIcon ? candidateIndex + 1 : "external") +
                  "; source=" +
                  describeIconSource(displayedIcon) +
                  "; visuallyBlank=" +
                  blank +
                  "; image=" +
                  JSON.stringify({
                    ...describeRenderedElement(event.currentTarget),
                    source: describeIconSource(
                      event.currentTarget.currentSrc ||
                        event.currentTarget.src,
                    ),
                    naturalWidth: event.currentTarget.naturalWidth,
                    naturalHeight: event.currentTarget.naturalHeight,
                  }),
              );
              if (blank) {
                if (activeIcon) tryNextCandidate();
                else
                  setExternalResult({ key: lookupKey, src: null, done: true });
              }
            }}
            onError={() => {
              reportIconDiagnostic(
                name +
                  ": failed candidate " +
                  (activeIcon ? candidateIndex + 1 : "external") +
                  "; source=" +
                  describeIconSource(displayedIcon),
              );
              if (activeIcon) tryNextCandidate();
              else setExternalResult({ key: lookupKey, src: null, done: true });
            }}
          />
        </>
      )}
    </span>
  );
}

export default AppIcon;
