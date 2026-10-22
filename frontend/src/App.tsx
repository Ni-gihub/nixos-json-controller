import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { Navigate, Route, Routes, useLocation } from "react-router";
import WindowTitleBar from "@/components/WindowTitleBar";
import AppDetail from "@/pages/AppDetail";
import Discover from "@/pages/Discover";
import { AppStateProvider } from "@/lib/app-state";

function App() {
  const location = useLocation();
  const isDiscoverRoute = location.pathname === "/discover";
  const [discoverMounted, setDiscoverMounted] = useState(isDiscoverRoute);
  const shouldRenderDiscover = discoverMounted || isDiscoverRoute;
  const savedDiscoverScrollYRef = useRef(0);
  const activeDiscoverRouteRef = useRef(isDiscoverRoute);
  const previousPathnameRef = useRef(location.pathname);

  useLayoutEffect(() => {
    activeDiscoverRouteRef.current = isDiscoverRoute;
  }, [isDiscoverRoute]);

  useEffect(() => {
    const rememberDiscoverScroll = () => {
      if (activeDiscoverRouteRef.current) {
        savedDiscoverScrollYRef.current = window.scrollY;
      }
    };

    window.addEventListener("scroll", rememberDiscoverScroll, {
      passive: true,
    });
    return () => {
      window.removeEventListener("scroll", rememberDiscoverScroll);
    };
  }, []);

  useEffect(() => {
    if (isDiscoverRoute && !discoverMounted) {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- keep Discover mounted after its first visit
      setDiscoverMounted(true);
    }
  }, [discoverMounted, isDiscoverRoute]);

  useEffect(() => {
    const previousPathname = previousPathnameRef.current;
    previousPathnameRef.current = location.pathname;
    if (!isDiscoverRoute || previousPathname === "/discover") return;

    // The hidden Discover page no longer contributes to document height while
    // detail is open, so restore the saved list position once it becomes visible.
    const frame = window.requestAnimationFrame(() => {
      window.scrollTo(0, savedDiscoverScrollYRef.current);
      // Ensure viewport-based virtualization recalculates even if scrollTo is a no-op.
      window.dispatchEvent(new Event("scroll"));
    });
    return () => window.cancelAnimationFrame(frame);
  }, [isDiscoverRoute, location.pathname]);

  return (
    <AppStateProvider>
      <div className="min-h-screen bg-background text-foreground">
        <WindowTitleBar />
        {shouldRenderDiscover && (
          <div hidden={!isDiscoverRoute} aria-hidden={!isDiscoverRoute}>
            <Discover />
          </div>
        )}
        <Routes>
          <Route path="/discover" element={null} />
          <Route path="/apps/:id" element={<AppDetail />} />
          <Route path="/" element={<Navigate to="/discover" replace />} />
          <Route path="*" element={<Navigate to="/discover" replace />} />
        </Routes>
      </div>
    </AppStateProvider>
  );
}

export default App;
