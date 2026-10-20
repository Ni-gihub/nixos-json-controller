import { useState, type FormEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

type InstallButtonProps = {
  packageId: string;
  appName: string;
  installed: boolean;
  statusKnown?: boolean;
  statusLoading?: boolean;
  statusError?: string | null;
  onRetryStatus?: () => void | Promise<void>;
  onInstalled?: () => void | Promise<void>;
  className?: string;
  size?: "default" | "sm" | "lg";
};

function InstallButton({
  packageId,
  appName,
  installed,
  statusKnown = true,
  statusLoading = false,
  statusError = null,
  onRetryStatus,
  onInstalled,
  className,
  size = "default",
}: InstallButtonProps) {
  const [installing, setInstalling] = useState(false);
  const [passwordRequired, setPasswordRequired] = useState(false);
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [installSucceeded, setInstallSucceeded] = useState(false);

  const effectiveInstalled = installed || installSucceeded;

  const install = async (authPassword: string | null) => {
    setInstalling(true);
    setError(null);
    try {
      await invoke("install_app", {
        package: packageId,
        password: authPassword,
      });
      setInstallSucceeded(true);
      setPassword("");
      setPasswordRequired(false);
      try {
        await onInstalled?.();
      } catch (err) {
        setError(String(err));
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setInstalling(false);
    }
  };

  const handleInstall = async () => {
    if (installing || effectiveInstalled || !statusKnown) return;

    setError(null);
    try {
      const available = await invoke<boolean>("sudo_available");
      if (available) {
        await install(null);
      } else {
        setPasswordRequired(true);
      }
    } catch (err) {
      setError(String(err));
    }
  };

  const handlePasswordSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!password) return;
    setPasswordRequired(false);
    await install(password);
  };

  return (
    <div className="w-full">
      {error && <p className="mb-3 text-sm text-destructive">{error}</p>}
      <Button
        type="button"
        size={size}
        className={className}
        disabled={installing || effectiveInstalled || !statusKnown}
        onClick={(event) => {
          event.preventDefault();
          event.stopPropagation();
          void handleInstall();
        }}
      >
        {effectiveInstalled
          ? "インストール済み"
          : installing
            ? "インストール中..."
            : "インストール"}
      </Button>

      {!statusKnown && !effectiveInstalled && (
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
          onClick={() => {
            setPassword("");
            setPasswordRequired(false);
          }}
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
                onClick={() => {
                  setPassword("");
                  setPasswordRequired(false);
                }}
              >
                キャンセル
              </Button>
              <Button
                type="submit"
                className="bg-primary text-primary-foreground hover:bg-primary/90"
                disabled={!password}
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

export default InstallButton;
