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
  PackageStateRevision,
  withUnknownPackageState,
  withVerifiedActivePackageState,
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
  markPackageActiveVerified: (packageName: string) => void;
  markPackageNeedsVerification: (packageName: string) => void;
  refreshPackages: (packageNames: string[]) => Promise<void>;
  refreshPackage: (packageName: string) => Promise<AppState | null>;
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
  const packageStateRevisionRef = useRef<PackageStateRevision | null>(null);
  if (packageStateRevisionRef.current === null) {
    packageStateRevisionRef.current = new PackageStateRevision();
  }
  const packageStateRevision = packageStateRevisionRef.current;

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

  const markPackageActiveVerified = useCallback(
    (packageName: string) => {
      packageStateRevision.advance(packageName);
      setStates((current) =>
        withVerifiedActivePackageState(current, packageName),
      );
    },
    [packageStateRevision],
  );

  const markPackageNeedsVerification = useCallback(
    (packageName: string) => {
      packageStateRevision.advance(packageName);
      setStates((current) => withUnknownPackageState(current, packageName));
    },
    [packageStateRevision],
  );

  const refreshPackages = useCallback(
    async (packageNames: string[]) => {
      if (packageNames.length === 0) return;

      // Capture each package's revision before issuing the request. Any package
      // installed while this request is pending advances its revision and makes
      // that part of the older response stale.
      const requestRevisions = new Map(
        packageNames.map((name) => [name, packageStateRevision.current(name)]),
      );

      setLoading(true);
      setError(null);
      try {
        const result = await invoke<AppState[]>("get_app_states", {
          packages: packageNames,
        });
        setStates((current) => {
          const next = { ...current };
          for (const state of result) {
            const revision = requestRevisions.get(state.name);
            if (
              revision !== undefined &&
              packageStateRevision.isCurrent(state.name, revision)
            ) {
              next[state.name] = state;
            }
          }
          return next;
        });
      } catch (err) {
        setError(String(err));
      } finally {
        setLoading(false);
      }
    },
    [packageStateRevision],
  );

  const refreshPackage = useCallback(
    async (packageName: string) => {
      setError(null);
      const revision = packageStateRevision.current(packageName);
      const state = await invoke<AppState>("get_app_state", {
        package: packageName,
      });
      if (!packageStateRevision.isCurrent(packageName, revision)) return null;
      setStates((current) => ({ ...current, [state.name]: state }));
      return state;
    },
    [packageStateRevision],
  );

  const value = useMemo(
    () => ({
      states,
      loading,
      error,
      installPhases,
      beginInstallRequest,
      setInstallRequestPhase,
      finishInstallRequest,
      markPackageActiveVerified,
      markPackageNeedsVerification,
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
      markPackageActiveVerified,
      markPackageNeedsVerification,
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
