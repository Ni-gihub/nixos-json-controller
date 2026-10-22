import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { Navigate, Route, Routes, useLocation } from "react-router";
import WindowTitleBar from "@/components/WindowTitleBar";
import AppDetail from "@/pages/AppDetail";
import { shouldRestoreDiscoverScroll } from "@/pages/discover-search-state";
import Discover from "@/pages/Discover";
import { AppStateProvider } from "@/lib/app-state";

function App() {
  const location = useLocation();
  const isDiscoverRoute = location.pathname === "/discover";
  const [discoverMounted, setDiscoverMounted] = useState(isDiscoverRoute);
  const shouldRenderDiscover = discoverMounted || isDiscoverRoute;
  const savedDiscoverScrollYRef = useRef(0);
  const savedDiscoverQueryRef = useRef(
    new URLSearchParams(location.search).get("query")?.trim() ?? "",
  );
  const activeDiscoverRouteRef = useRef(isDiscoverRoute);
  const previousPathnameRef = useRef(location.pathname);

  useLayoutEffect(() => {
    const previousPathname = previousPathnameRef.current;
    const currentQuery =
      new URLSearchParams(location.search).get("query")?.trim() ?? "";

    if (
      isDiscoverRoute &&
      shouldRestoreDiscoverScroll(
        previousPathname,
        savedDiscoverQueryRef.current,
        currentQuery,
      )
    ) {
      // Restore before paint so the retained list never flashes at the top.
      window.scrollTo(0, savedDiscoverScrollYRef.current);
      window.dispatchEvent(new Event("scroll"));
    }

    activeDiscoverRouteRef.current = isDiscoverRoute;
    previousPathnameRef.current = location.pathname;
    if (isDiscoverRoute) {
      savedDiscoverQueryRef.current = currentQuery;
    }
  }, [isDiscoverRoute, location.pathname, location.search]);

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
