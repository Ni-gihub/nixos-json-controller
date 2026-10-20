export type InstallRequestPhase = "checking" | "password" | "installing";

/** Synchronous per-package lock used to prevent duplicate install commands. */
export class InstallRequestLock {
  private readonly activePackages = new Set<string>();

  tryAcquire(packageName: string): boolean {
    if (this.activePackages.has(packageName)) return false;
    this.activePackages.add(packageName);
    return true;
  }

  release(packageName: string): void {
    this.activePackages.delete(packageName);
  }

  isLocked(packageName: string): boolean {
    return this.activePackages.has(packageName);
  }
}
