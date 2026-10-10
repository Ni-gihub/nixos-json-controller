import assert from 'node:assert/strict'
import test from 'node:test'
import React from 'react'
import { renderToStaticMarkup } from 'react-dom/server'
import { createServer } from 'vite'
import {
  collectIconCandidates,
  getActiveIconCandidate,
  isVisuallyBlankPixels,
  nextCandidateIndex,
} from './app-icon-logic.ts'

test('failed image candidates advance in order and end with no active image', () => {
  const candidates = collectIconCandidates('first.png', ['second.png', 'second.png', 'third.png'])
  assert.deepEqual(candidates, ['first.png', 'second.png', 'third.png'])

  let index = 0
  assert.equal(getActiveIconCandidate(candidates, index), 'first.png')

  // onError and the blank-image detector share this transition.
  index = nextCandidateIndex(index, candidates.length)
  assert.equal(getActiveIconCandidate(candidates, index), 'second.png')

  index = nextCandidateIndex(index, candidates.length)
  assert.equal(getActiveIconCandidate(candidates, index), 'third.png')

  index = nextCandidateIndex(index, candidates.length)
  assert.equal(getActiveIconCandidate(candidates, index), undefined)
})

test('blank local pixels are detected so the component can try the next icon', () => {
  const pixels = new Uint8ClampedArray(24 * 24 * 4)
  for (let offset = 0; offset < pixels.length; offset += 4) {
    pixels[offset] = 255
    pixels[offset + 1] = 255
    pixels[offset + 2] = 255
    pixels[offset + 3] = 255
  }

  assert.equal(isVisuallyBlankPixels(pixels), true)

  // Eight visible pixels meet the minimum-contrast threshold.
  for (let pixel = 0; pixel < 8; pixel += 1) {
    const offset = pixel * 4
    pixels[offset] = 0
    pixels[offset + 1] = 0
    pixels[offset + 2] = 0
  }
  assert.equal(isVisuallyBlankPixels(pixels), false)
})

test('rejects icon sources that cannot be inspected instead of retaining a blank overlay', async () => {
  const vite = await createServer({
    configFile: './vite.config.ts',
    logLevel: 'silent',
    server: { middlewareMode: true },
    appType: 'custom',
  })

  const hadDocument = Object.hasOwn(globalThis, 'document')
  const originalDocument = globalThis.document

  try {
    const { isVisuallyBlankLocalImage } = await vite.ssrLoadModule('/src/components/AppIcon.tsx')
    const image = {
      currentSrc: 'data:image/svg+xml;base64,PHN2Zy8+',
      src: 'data:image/svg+xml;base64,PHN2Zy8+',
    }

    // Simulate WebKit refusing pixel reads for an SVG with external references.
    globalThis.document = {
      createElement: () => ({
        getContext: () => ({
          drawImage: () => { throw new Error('canvas is not inspectable') },
        }),
      }),
    }
    assert.equal(isVisuallyBlankLocalImage(image), true)

    // Unexpected remote URLs and unavailable canvas contexts are also rejected.
    assert.equal(isVisuallyBlankLocalImage({
      currentSrc: 'https://example.com/blank.png',
      src: 'https://example.com/blank.png',
    }), true)
    globalThis.document = { createElement: () => ({ getContext: () => null }) }
    assert.equal(isVisuallyBlankLocalImage(image), true)
  } finally {
    if (hadDocument) {
      globalThis.document = originalDocument
    } else {
      delete globalThis.document
    }
    await vite.close()
  }
})

test('renders the generic Package icon when no candidate exists', async () => {
  const vite = await createServer({
    configFile: './vite.config.ts',
    logLevel: 'silent',
    server: { middlewareMode: true },
    appType: 'custom',
  })

  try {
    const { default: AppIcon } = await vite.ssrLoadModule('/src/components/AppIcon.tsx')
    const markup = renderToStaticMarkup(React.createElement(AppIcon, {
      name: 'Firefox',
      iconCandidates: [],
      imageClassName: 'size-8',
      fallbackClassName: 'size-6',
    }))

    assert.match(markup, /<svg/)
    assert.doesNotMatch(markup, /<img/)
  } finally {
    await vite.close()
  }
})
