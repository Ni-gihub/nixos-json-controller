/** Build a stable, ordered list of distinct icon sources from the package fields. */
export function collectIconCandidates(
  icon?: string,
  iconCandidates?: string[],
): string[] {
  return Array.from(new Set([...(icon ? [icon] : []), ...(iconCandidates ?? [])]))
}

/** Return the candidate at the current fallback index, or undefined when all have failed. */
export function getActiveIconCandidate(
  candidates: string[],
  index: number,
): string | undefined {
  return candidates[index]
}

/** Advance one position after an image fails or is detected as visually blank. */
export function nextCandidateIndex(currentIndex: number, candidateCount: number): number {
  return Math.min(Math.max(0, currentIndex) + 1, candidateCount)
}

/** Detect images that are nearly transparent or too close to the light icon background. */
export function isVisuallyBlankPixels(pixels: ArrayLike<number>): boolean {
  const background = [247, 248, 250] as const
  const minimumContrast = 22
  const minimumVisiblePixels = 8
  let contrastingPixels = 0

  for (let offset = 0; offset < pixels.length; offset += 4) {
    const alpha = pixels[offset + 3] / 255
    if (alpha < 0.08) continue

    const red = pixels[offset] * alpha + background[0] * (1 - alpha)
    const green = pixels[offset + 1] * alpha + background[1] * (1 - alpha)
    const blue = pixels[offset + 2] * alpha + background[2] * (1 - alpha)
    const contrast = Math.max(
      Math.abs(red - background[0]),
      Math.abs(green - background[1]),
      Math.abs(blue - background[2]),
    )

    if (contrast >= minimumContrast) {
      contrastingPixels += 1
      if (contrastingPixels >= minimumVisiblePixels) return false
    }
  }

  return true
}
