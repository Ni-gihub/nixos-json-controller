import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  InstallRequestLock,
  type InstallRequestPhase,
} from "./install-request-lock";
import {
  withInstalledPackageState,
  type PackageState,
} from "./package-state";

export type AppState = PackageState;

type AppStateContextValue = {
  states: Record<string, AppState>;
  loading: boolean;
  error: string | null;
  installPhases: Record<string, InstallRequestPhase>;
  beginInstallRequest: (
    packageName: string,
    phase: InstallRequestPhase,
  ) => boolean;
  setInstallRequestPhase: (
    packageName: string,
    phase: InstallRequestPhase,
  ) => void;
  finishInstallRequest: (packageName: string) => void;
  markPackageInstalled: (packageName: string) => void;
  refreshPackages: (packageNames: string[]) => Promise<void>;
  refreshPackage: (packageName: string) => Promise<void>;
};

const AppStateContext = createContext<AppStateContextValue | null>(null);

export function AppStateProvider({ children }: { children: ReactNode }) {
  const [states, setStates] = useState<Record<string, AppState>>({});
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [installPhases, setInstallPhases] = useState<
    Record<string, InstallRequestPhase>
  >({});
  const installLockRef = useRef<InstallRequestLock | null>(null);
  if (installLockRef.current === null) {
    installLockRef.current = new InstallRequestLock();
  }
  const installLock = installLockRef.current;

  const beginInstallRequest = useCallback(
    (packageName: string, phase: InstallRequestPhase) => {
      if (!installLock.tryAcquire(packageName)) return false;
      setInstallPhases((current) => ({ ...current, [packageName]: phase }));
      return true;
    },
    [installLock],
  );

  const setInstallRequestPhase = useCallback(
    (packageName: string, phase: InstallRequestPhase) => {
      if (!installLock.isLocked(packageName)) return;
      setInstallPhases((current) =>
        current[packageName] === phase
          ? current
          : { ...current, [packageName]: phase },
      );
    },
    [installLock],
  );

  const finishInstallRequest = useCallback(
    (packageName: string) => {
      installLock.release(packageName);
      setInstallPhases((current) => {
        if (!(packageName in current)) return current;
        const next = { ...current };
        delete next[packageName];
        return next;
      });
    },
    [installLock],
  );

  const markPackageInstalled = useCallback((packageName: string) => {
    setStates((current) => withInstalledPackageState(current, packageName));
  }, []);

  const refreshPackages = useCallback(async (packageNames: string[]) => {
    if (packageNames.length === 0) return;

    setLoading(true);
    setError(null);
    try {
      const result = await invoke<AppState[]>("get_app_states", {
        packages: packageNames,
      });
      setStates((current) => ({
        ...current,
        ...Object.fromEntries(result.map((state) => [state.name, state])),
      }));
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  const refreshPackage = useCallback(async (packageName: string) => {
    setError(null);
    const state = await invoke<AppState>("get_app_state", {
      package: packageName,
    });
    setStates((current) => ({ ...current, [state.name]: state }));
  }, []);

  const value = useMemo(
    () => ({
      states,
      loading,
      error,
      installPhases,
      beginInstallRequest,
      setInstallRequestPhase,
      finishInstallRequest,
      markPackageInstalled,
      refreshPackages,
      refreshPackage,
    }),
    [
      states,
      loading,
      error,
      installPhases,
      beginInstallRequest,
      setInstallRequestPhase,
      finishInstallRequest,
      markPackageInstalled,
      refreshPackages,
      refreshPackage,
    ],
  );

  return (
    <AppStateContext.Provider value={value}>
      {children}
    </AppStateContext.Provider>
  );
}

// eslint-disable-next-line react-refresh/only-export-components -- keep the context hook next to its provider
export function useAppState() {
  const context = useContext(AppStateContext);
  if (!context) {
    throw new Error("useAppState must be used within AppStateProvider");
  }
  return context;
}
