export type PackageState = {
  name: string;
  configured: boolean | null;
  active: boolean | null;
};

/**
 * Apply the confirmed result of a successful install without re-evaluating Nix.
 *
 * The install command resolves only after its backend verification succeeds, so
 * the GUI can update shared package state directly instead of issuing another
 * get_app_state request for the same package.
 */
export function withInstalledPackageState(
  states: Record<string, PackageState>,
  packageName: string,
): Record<string, PackageState> {
  return {
    ...states,
    [packageName]: {
      name: packageName,
      configured: true,
      active: true,
    },
  };
}
