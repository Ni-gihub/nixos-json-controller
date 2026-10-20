export type InstallPasswordActions = {
  closePasswordPrompt: () => void;
  clearPassword: () => void;
  install: (password: string) => Promise<void>;
};

/**
 * Dismiss the credential UI and clear its state before waiting for installation.
 * The password is passed to the pending operation only after the UI is cleared.
 */
export async function submitInstallPassword(
  password: string,
  actions: InstallPasswordActions,
): Promise<void> {
  actions.closePasswordPrompt();
  actions.clearPassword();
  await actions.install(password);
}
