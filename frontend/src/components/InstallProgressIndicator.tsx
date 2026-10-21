type InstallProgressIndicatorProps = {
  label: string;
  ariaLabel?: string;
};

function InstallProgressIndicator({
  label,
  ariaLabel = label,
}: InstallProgressIndicatorProps) {
  return (
    <span
      role="status"
      aria-live="polite"
      aria-label={ariaLabel}
      className="inline-flex items-center justify-center gap-2 whitespace-nowrap"
    >
      <svg
        aria-hidden="true"
        viewBox="0 0 24 24"
        className="size-4 shrink-0 animate-spin text-current motion-reduce:animate-none"
        fill="none"
      >
        <circle
          cx="12"
          cy="12"
          r="9"
          stroke="currentColor"
          strokeWidth="3"
          opacity="0.2"
        />
        <circle
          cx="12"
          cy="12"
          r="9"
          stroke="currentColor"
          strokeWidth="3"
          strokeLinecap="round"
          strokeDasharray="38 57"
          transform="rotate(-90 12 12)"
        />
      </svg>
      <span>{label}</span>
    </span>
  );
}

export default InstallProgressIndicator;
