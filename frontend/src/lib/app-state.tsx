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
  fetchPackageStateBatches,
  PackageStateRevision,
  PendingRequestCounter,
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
  const pendingStateRequestsRef = useRef<PendingRequestCounter | null>(null);
  if (pendingStateRequestsRef.current === null) {
    pendingStateRequestsRef.current = new PendingRequestCounter();
  }
  const pendingStateRequests = pendingStateRequestsRef.current;

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

      // Each request gets a new revision per package. Only the newest request
      // for a given ID may update that ID; unrelated IDs in the same batch remain valid.
      const requestRevisions = new Map(
        packageNames.map((name) => [name, packageStateRevision.advance(name)]),
      );

      const requestStartedAt = performance.now();
      const shouldLogPerf =
        import.meta.env.DEV && import.meta.env.VITE_PERF_DEBUG === "1";
      pendingStateRequests.begin();
      setLoading(true);
      setError(null);
      try {
        const { results, failures } = await fetchPackageStateBatches(
          packageNames,
          (packageBatch) =>
            invoke<AppState[]>("get_app_states", {
              packages: packageBatch,
            }),
        );

        // Apply every successful batch even if another batch still failed after retry.
        setStates((current) => {
          const next = { ...current };
          for (const state of results) {
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

        const currentFailures = failures.filter(
          ({ packageNames: failedNames }) =>
            failedNames.some((name) => {
              const revision = requestRevisions.get(name);
              return (
                revision !== undefined &&
                packageStateRevision.isCurrent(name, revision)
              );
            }),
        );
        if (currentFailures.length > 0) {
          setError(
            [
              "Some package states could not be refreshed after retry:",
              String(currentFailures[0].error),
            ].join(" "),
          );
        }
      } catch (err) {
        // A stale request must not replace the error state of newer requests.
        const hasCurrentPackage = [...requestRevisions].some(
          ([name, revision]) => packageStateRevision.isCurrent(name, revision),
        );
        if (hasCurrentPackage) setError(String(err));
      } finally {
        setLoading(pendingStateRequests.finish());
        if (shouldLogPerf) {
          console.info("[nxc-perf] package_state_refresh", {
            elapsed_ms: Math.round(performance.now() - requestStartedAt),
            package_count: packageNames.length,
          });
        }
      }
    },
    [packageStateRevision, pendingStateRequests],
  );

  const refreshPackage = useCallback(
    async (packageName: string) => {
      setError(null);
      const revision = packageStateRevision.advance(packageName);
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
