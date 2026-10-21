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

/** Sends an icon diagnostic when enabled and accepted by the optional package filter. */
function reportIconDiagnostic(message: string) {
  if (import.meta.env.VITE_ICON_DEBUG !== "1") return;

  const packageFilter =
    import.meta.env.VITE_ICON_DEBUG_PACKAGE?.trim().toLowerCase();
  if (packageFilter) {
    const separator = message.indexOf(":");
    const subject = separator === -1 ? message : message.slice(0, separator);
    if (!subject.toLowerCase().includes(packageFilter)) return;
  }

  void invoke("log_icon_debug", { message }).catch(() => {
    // The component can also be rendered outside Tauri during frontend development.
  });
}

/** Describes an icon source without logging its full URL or data payload. */
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

/** Captures computed CSS and layout metrics for a rendered element. */
function describeRenderedElement(element: Element) {
  const rect = element.getBoundingClientRect();
  const style = window.getComputedStyle(element);
  const htmlElement = element as HTMLElement;

  return {
    tag: element.tagName.toLowerCase(),
    className: element.getAttribute("class"),
    bounds: {
      x: Number(rect.x.toFixed(2)),
      y: Number(rect.y.toFixed(2)),
      left: Number(rect.left.toFixed(2)),
      top: Number(rect.top.toFixed(2)),
      right: Number(rect.right.toFixed(2)),
      bottom: Number(rect.bottom.toFixed(2)),
      width: Number(rect.width.toFixed(2)),
      height: Number(rect.height.toFixed(2)),
    },
    cssSize: {
      width: style.width,
      height: style.height,
      minWidth: style.minWidth,
      maxWidth: style.maxWidth,
      minHeight: style.minHeight,
      maxHeight: style.maxHeight,
      boxSizing: style.boxSizing,
      fontSize: style.fontSize,
      zoom: style.zoom,
      transform: style.transform,
      translate: style.translate,
      scale: style.scale,
      rotate: style.rotate,
    },
    layoutSize: {
      offsetWidth: Number.isFinite(htmlElement.offsetWidth)
        ? htmlElement.offsetWidth
        : null,
      offsetHeight: Number.isFinite(htmlElement.offsetHeight)
        ? htmlElement.offsetHeight
        : null,
      clientWidth: Number.isFinite(htmlElement.clientWidth)
        ? htmlElement.clientWidth
        : null,
      clientHeight: Number.isFinite(htmlElement.clientHeight)
        ? htmlElement.clientHeight
        : null,
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

/** Collects rendering diagnostics from an element up to the document root. */
function describeAncestors(element: Element) {
  const ancestors: ReturnType<typeof describeRenderedElement>[] = [];
  let current: Element | null = element;

  while (current) {
    ancestors.push(describeRenderedElement(current));
    if (current === document.documentElement) break;
    current = current.parentElement;
  }

  return ancestors;
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
  const [refreshedCandidates, setRefreshedCandidates] = useState<{
    packageId: string;
    name: string;
    sourceIconCandidatesKey: string;
    candidates: string[];
  } | null>(null);
  const sourceIconCandidatesKey = JSON.stringify(iconCandidates ?? []);
  // A refresh only supersedes the props used to request it. New catalog props must win.
  // Compare candidate contents rather than array identity: parents may recreate an equivalent array.
  // This opt-in mode isolates Lucide/WebView rendering from package and external image sources.
  const currentIconCandidates =
    refreshedCandidates?.packageId === packageId &&
    refreshedCandidates.name === name &&
    refreshedCandidates.sourceIconCandidatesKey === sourceIconCandidatesKey
      ? refreshedCandidates.candidates
      : iconCandidates;
  const candidates = DEBUG_FALLBACK_ONLY
    ? []
    : collectIconCandidates(icon, currentIconCandidates);
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

  // A verified install can make a previously missing local icon available.
  // Ask Rust for fresh candidates so the current card/detail can render and persist it.
  useEffect(() => {
    let cancelled = false;
    const handlePackageInstalled = (event: Event) => {
      const detail = (event as CustomEvent<{ packageId?: string }>).detail;
      if (detail?.packageId !== packageId) return;

      void invoke<string[]>("get_icon_candidates", { package: packageId, name })
        .then((nextCandidates) => {
          if (!cancelled) {
            setRefreshedCandidates({
              packageId,
              name,
              sourceIconCandidatesKey: JSON.stringify(iconCandidates ?? []),
              candidates: nextCandidates,
            });
          }
        })
        .catch(() => {
          // Icon refresh is best-effort; keep the existing fallback on failure.
        });
    };

    window.addEventListener("nxc:package-installed", handlePackageInstalled);
    return () => {
      cancelled = true;
      window.removeEventListener(
        "nxc:package-installed",
        handlePackageInstalled,
      );
    };
  }, [packageId, name, sourceIconCandidatesKey]);

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
    const image = imageRef.current;
    const renderedElement = image ?? svg;
    if (!container || !renderedElement) return;

    const ancestors = describeAncestors(renderedElement);
    const pathStyles = svg
      ? Array.from(svg.querySelectorAll("path"))
          .slice(0, 3)
          .map((path) => {
            const style = window.getComputedStyle(path);
            return {
              stroke: style.stroke,
              fill: style.fill,
              opacity: style.opacity,
              pathLength: path.getAttribute("d")?.length ?? 0,
            };
          })
      : [];

    reportIconDiagnostic(
      name +
        ": rendered DOM " +
        JSON.stringify({
          fallbackOnly: DEBUG_FALLBACK_ONLY,
          viewport: {
            devicePixelRatio: window.devicePixelRatio,
            innerWidth: window.innerWidth,
            innerHeight: window.innerHeight,
            outerWidth: window.outerWidth,
            outerHeight: window.outerHeight,
            screenWidth: window.screen.width,
            screenHeight: window.screen.height,
            documentWidth: document.documentElement.clientWidth,
            documentHeight: document.documentElement.clientHeight,
            visualViewport: window.visualViewport
              ? {
                  width: window.visualViewport.width,
                  height: window.visualViewport.height,
                  scale: window.visualViewport.scale,
                  offsetLeft: window.visualViewport.offsetLeft,
                  offsetTop: window.visualViewport.offsetTop,
                }
              : null,
          },
          container: describeRenderedElement(container),
          packageSvg: svg
            ? {
                ...describeRenderedElement(svg),
                widthAttribute: svg.getAttribute("width"),
                heightAttribute: svg.getAttribute("height"),
                viewBox: svg.getAttribute("viewBox"),
                strokeAttribute: svg.getAttribute("stroke"),
                fillAttribute: svg.getAttribute("fill"),
                pathCount: svg.querySelectorAll("path").length,
                samplePaths: pathStyles,
              }
            : null,
          ancestors,
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
                    event.currentTarget.currentSrc || event.currentTarget.src,
                  ),
                  naturalWidth: event.currentTarget.naturalWidth,
                  naturalHeight: event.currentTarget.naturalHeight,
                }),
            );
            if (blank) {
              if (activeIcon) tryNextCandidate();
              else setExternalResult({ key: lookupKey, src: null, done: true });
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
      )}
    </span>
  );
}

export default AppIcon;
