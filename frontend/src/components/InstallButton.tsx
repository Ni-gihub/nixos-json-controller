import { useLayoutEffect, useRef, useState, type FormEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import InstallProgressIndicator from "@/components/InstallProgressIndicator";
import { submitInstallPassword } from "@/components/install-flow";
import { useAppState } from "@/lib/app-state";

type InstallButtonProps = {
  packageId: string;
  appName: string;
  installed: boolean;
  statusKnown?: boolean;
  statusLoading?: boolean;
  statusError?: string | null;
  onRetryStatus?: () => void | Promise<void>;
  className?: string;
  size?: "default" | "sm" | "lg";
};

function InstallButtonInstance({
  packageId,
  appName,
  installed,
  statusKnown = true,
  statusLoading = false,
  statusError = null,
  onRetryStatus,
  className,
  size = "default",
}: InstallButtonProps) {
  const {
    installPhases,
    beginInstallRequest,
    setInstallRequestPhase,
    finishInstallRequest,
    markPackageActiveVerified,
    markPackageNeedsVerification,
    refreshPackage,
  } = useAppState();
  const [passwordRequired, setPasswordRequired] = useState(false);
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [installSucceeded, setInstallSucceeded] = useState(false);
  const [verificationPending, setVerificationPending] = useState(false);
  const passwordPromptOpenRef = useRef(false);
  const unmountedRef = useRef(false);

  const installPhase = installPhases[packageId] ?? null;
  const busy = installPhase !== null;
  const effectiveInstalled = installed || installSucceeded;

  useLayoutEffect(() => {
    unmountedRef.current = false;
    return () => {
      unmountedRef.current = true;
      // A password prompt may be abandoned by navigating to another page.
      // Release only that prompt's reservation; an active install owns its lock
      // until its command settles.
      if (passwordPromptOpenRef.current) {
        finishInstallRequest(packageId);
      }
    };
  }, [finishInstallRequest, packageId]);

  const install = async (authPassword: string | null) => {
    setError(null);
    try {
      const result = await invoke<{
        package: string;
        installed: boolean;
        active_state_verified: boolean;
      }>("install_app", {
        package: packageId,
        password: authPassword,
      });
      if (!result.installed) {
        throw new Error(
          `インストール処理が完了しませんでした: ${result.package}`,
        );
      }

      if (result.active_state_verified) {
        // Reuse the backend's verification after nixos-rebuild instead of
        // running another Nix evaluation for the same package.
        markPackageActiveVerified(result.package);
        setInstallSucceeded(true);
      } else {
        // A successful no-op path does not prove that the package is active.
        // Invalidate older lookups and refresh the actual state in the background.
        markPackageNeedsVerification(result.package);
        setInstallSucceeded(false);
        setVerificationPending(true);
        void refreshPackage(result.package)
          .then((state) => {
            if (unmountedRef.current || state === null) return;
            if (state.active === true) {
              setInstallSucceeded(true);
            } else {
              setError(
                "処理は完了しましたが、アクティブなシステムでインストール状態を確認できませんでした。",
              );
            }
          })
          .catch((err) => {
            if (!unmountedRef.current) {
              setError(`インストール状態の確認に失敗しました: ${String(err)}`);
            }
          })
          .finally(() => {
            if (!unmountedRef.current) setVerificationPending(false);
          });
      }
    } catch (err) {
      setError(String(err));
    } finally {
      // Do not retain the password in React state after either success or failure.
      setPassword("");
    }
  };

  const handleInstall = async () => {
    if (busy || verificationPending || effectiveInstalled || !statusKnown) return;
    if (!beginInstallRequest(packageId, "checking")) return;

    let keepReservation = false;
    setError(null);
    try {
      const available = await invoke<boolean>("sudo_available");
      if (unmountedRef.current) return;

      if (available) {
        setInstallRequestPhase(packageId, "installing");
        await install(null);
      } else if (!unmountedRef.current) {
        setInstallRequestPhase(packageId, "password");
        passwordPromptOpenRef.current = true;
        setPasswordRequired(true);
        keepReservation = true;
      }
    } catch (err) {
      setError(String(err));
    } finally {
      if (!keepReservation) {
        finishInstallRequest(packageId);
      }
    }
  };

  const handlePasswordSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!password || installPhase !== "password") return;

    const submittedPassword = password;
    passwordPromptOpenRef.current = false;
    setInstallRequestPhase(packageId, "installing");

    try {
      await submitInstallPassword(submittedPassword, {
        closePasswordPrompt: () => setPasswordRequired(false),
        clearPassword: () => setPassword(""),
        install,
      });
    } finally {
      finishInstallRequest(packageId);
    }
  };

  const closePasswordPrompt = () => {
    passwordPromptOpenRef.current = false;
    setPassword("");
    setPasswordRequired(false);
    finishInstallRequest(packageId);
  };

  const renderInstallButtonContent = () => {
    if (effectiveInstalled) return "インストール済み";
    if (verificationPending) return "処理完了";

    switch (installPhase) {
      case "installing":
        return <InstallProgressIndicator label="インストール中..." />;
      case "checking":
        return <InstallProgressIndicator label="確認中..." />;
      case "password":
        return "認証待ち...";
      default:
        return "インストール";
    }
  };

  return (
    <div className="w-full">
      {error && <p className="mb-3 text-sm text-destructive">{error}</p>}
      <Button
        type="button"
        size={size}
        className={className}
        disabled={busy || verificationPending || effectiveInstalled || !statusKnown}
        onClick={(event) => {
          event.preventDefault();
          event.stopPropagation();
          void handleInstall();
        }}
      >
        {renderInstallButtonContent()}
      </Button>

      {verificationPending && (
        <p className="mt-2 text-xs text-muted-foreground" role="status">
          アクティブなシステムへの反映を確認中...
        </p>
      )}

      {!statusKnown && !effectiveInstalled && !verificationPending && (
        <div className="mt-2 space-y-1">
          <p className="text-xs text-muted-foreground">
            {statusLoading
              ? "インストール状態を確認中..."
              : statusError
                ? "インストール状態の取得に失敗しました。"
                : "このパッケージのインストール状態を確認できません。"}
          </p>
          {statusError && (
            <p className="break-words text-xs text-destructive">
              {statusError}
            </p>
          )}
          {!statusLoading && onRetryStatus && (
            <Button
              type="button"
              size="sm"
              variant="outline"
              onClick={() => void onRetryStatus()}
            >
              再試行
            </Button>
          )}
        </div>
      )}

      {passwordRequired && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/45 p-6 backdrop-blur-sm"
          onClick={closePasswordPrompt}
        >
          <form
            onSubmit={handlePasswordSubmit}
            className="w-full max-w-sm rounded-2xl border border-primary/15 bg-card p-6 shadow-2xl shadow-blue-950/10"
            onClick={(event) => event.stopPropagation()}
          >
            <h2 className="text-lg font-semibold">認証が必要です</h2>
            <p className="mt-2 text-sm text-muted-foreground">
              {appName}をインストールするため、パスワードを入力してください。
            </p>
            <Input
              autoFocus
              className="mt-4"
              type="password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              placeholder="パスワード"
              autoComplete="current-password"
            />
            <div className="mt-6 flex justify-end gap-2">
              <Button
                type="button"
                variant="outline"
                onClick={closePasswordPrompt}
              >
                キャンセル
              </Button>
              <Button
                type="submit"
                className="bg-primary text-primary-foreground hover:bg-primary/90"
                disabled={!password || installPhase !== "password"}
              >
                続行
              </Button>
            </div>
          </form>
        </div>
      )}
    </div>
  );
}

function InstallButton(props: InstallButtonProps) {
  // Package changes must create a fresh prompt/request state boundary. This also
  // prevents an old sudo_available response from mutating a newly selected app.
  return <InstallButtonInstance key={props.packageId} {...props} />;
}

export default InstallButton;
