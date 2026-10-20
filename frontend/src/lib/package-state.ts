export type PackageState = {
  name: string;
  configured: boolean | null;
  active: boolean | null;
};

/**
 * Tracks package-specific state lookups so a response started before a newer
 * install result cannot overwrite that result when it arrives late.
 */
export class PackageStateRevision {
  private readonly revisions = new Map<string, number>();

  current(packageName: string): number {
    return this.revisions.get(packageName) ?? 0;
  }

  advance(packageName: string): number {
    const next = this.current(packageName) + 1;
    this.revisions.set(packageName, next);
    return next;
  }

  isCurrent(packageName: string, revision: number): boolean {
    return this.current(packageName) === revision;
  }
}

/** State for a package whose active status was verified by the install executor. */
export function withVerifiedActivePackageState(
  states: Record<string, PackageState>,
  packageName: string,
): Record<string, PackageState> {
  return {
    ...states,
    [packageName]: {
      name: packageName,
      // Active state is verified, but configured provenance was not returned.
      configured: null,
      active: true,
    },
  };
}

/** Invalidate stale state while an unverified successful operation is checked. */
export function withUnknownPackageState(
  states: Record<string, PackageState>,
  packageName: string,
): Record<string, PackageState> {
  return {
    ...states,
    [packageName]: {
      name: packageName,
      configured: null,
      active: null,
    },
  };
}
