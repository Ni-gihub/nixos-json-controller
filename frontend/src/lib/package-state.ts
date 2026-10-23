/** Split package-state IPC requests so each stays within the Rust command's input limit. */
export const DEFAULT_PACKAGE_STATE_BATCH_SIZE = 100;

export function chunkPackageNames(
  packageNames: readonly string[],
  batchSize = DEFAULT_PACKAGE_STATE_BATCH_SIZE,
): string[][] {
  if (!Number.isInteger(batchSize) || batchSize < 1) {
    throw new RangeError("batchSize must be a positive integer");
  }

  const batches: string[][] = [];
  for (let offset = 0; offset < packageNames.length; offset += batchSize) {
    batches.push(packageNames.slice(offset, offset + batchSize));
  }
  return batches;
}

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

/** Counts independent state requests so one completion cannot hide another. */
export class PendingRequestCounter {
  private pending = 0;

  begin(): void {
    this.pending += 1;
  }

  finish(): boolean {
    if (this.pending > 0) this.pending -= 1;
    return this.pending > 0;
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
