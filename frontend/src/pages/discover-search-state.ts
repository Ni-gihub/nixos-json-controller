export type SearchNavigationType = "POP" | "PUSH" | "REPLACE";

export type SearchUrlTransition =
  | { kind: "external"; query: string }
  | { kind: "internal" }
  | { kind: "stale" };

/**
 * Distinguish an app-initiated search-param update from a stale URL value or
 * a navigation initiated outside the current search action.
 */
export function classifySearchUrlTransition(
  urlQuery: string,
  pendingQuery: string | null,
  navigationType: SearchNavigationType,
): SearchUrlTransition {
  // Browser back/forward must win over an app-initiated update still in flight.
  if (navigationType === "POP" || pendingQuery === null) {
    return { kind: "external", query: urlQuery };
  }

  if (urlQuery === pendingQuery) {
    return { kind: "internal" };
  }

  return { kind: "stale" };
}

/** Ignore results from any search request superseded by a newer action. */
export function isCurrentSearchRequest(
  requestId: number,
  latestRequestId: number,
): boolean {
  return requestId === latestRequestId;
}


/** Pause automatic pagination after several consecutive pages with no visible results. */
export function advanceEmptyPageAutoLoadState(
  consecutiveEmptyPages: number,
  resultCount: number,
  maxConsecutiveEmptyPages = 3,
): { consecutiveEmptyPages: number; paused: boolean } {
  const nextCount = resultCount > 0 ? 0 : consecutiveEmptyPages + 1;
  return {
    consecutiveEmptyPages: nextCount,
    paused: nextCount >= maxConsecutiveEmptyPages,
  };
}
