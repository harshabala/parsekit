#!/usr/bin/env swift
// ParseKit DMG background — logical window 640×440 (tauri.conf + create-dmg).
// 1× on purpose: the background PNG's point size MUST equal the create-dmg
// window size (640×440). At 2× the PNG reports as 1280×880 pt, and Retina Finder
// anchors it top-left without fitting, cropping the text to "Drag ParseKit t…".
// At 1×, pixels == points == window, so the instruction line is always centered.
import AppKit
import CoreGraphics

let scale: CGFloat = 1
let windowW: CGFloat = 640
let windowH: CGFloat = 440
let width = windowW * scale
let height = windowH * scale
let margin: CGFloat = 40 * scale

guard
  let rep = NSBitmapImageRep(
    bitmapDataPlanes: nil,
    pixelsWide: Int(width),
    pixelsHigh: Int(height),
    bitsPerSample: 8,
    samplesPerPixel: 4,
    hasAlpha: true,
    isPlanar: false,
    colorSpaceName: .deviceRGB,
    bytesPerRow: 0,
    bitsPerPixel: 0
  )
else {
  fputs("Failed to create bitmap\n", stderr)
  exit(1)
}
// Pixel dimensions must match rep.size so drawing coordinates align with the bitmap.
rep.size = NSSize(width: width, height: height)

guard let ctx = NSGraphicsContext(bitmapImageRep: rep)?.cgContext else {
  fputs("No graphics context\n", stderr)
  exit(1)
}
ctx.interpolationQuality = .high
ctx.setShouldAntialias(true)
ctx.setAllowsFontSmoothing(true)

NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(cgContext: ctx, flipped: false)

let space = CGColorSpaceCreateDeviceRGB()
let colors = [
  CGColor(red: 0.86, green: 0.80, blue: 0.74, alpha: 1),
  CGColor(red: 0.94, green: 0.90, blue: 0.86, alpha: 1),
  CGColor(red: 0.98, green: 0.96, blue: 0.93, alpha: 1),
] as CFArray
if let gradient = CGGradient(colorsSpace: space, colors: colors, locations: [0, 0.5, 1]) {
  ctx.drawLinearGradient(
    gradient,
    start: CGPoint(x: 0, y: 0),
    end: CGPoint(x: 0, y: height),
    options: []
  )
}

// Single instruction line at top — nothing drawn over the icon row (y ≈ 198 in create-dmg).
let title = "Drag ParseKit to Applications"
let titleFont = NSFont.systemFont(ofSize: 22 * scale, weight: .semibold)
let titleColor = NSColor(calibratedRed: 0.20, green: 0.17, blue: 0.14, alpha: 1)
let titleAttrs: [NSAttributedString.Key: Any] = [
  .font: titleFont,
  .foregroundColor: titleColor,
]
let titleSize = (title as NSString).size(withAttributes: titleAttrs)
let titleX = (width - titleSize.width) / 2
// Cocoa origin bottom-left: place near top with safe margin
let titleY = height - margin - titleSize.height
(title as NSString).draw(at: CGPoint(x: titleX, y: titleY), withAttributes: titleAttrs)

NSGraphicsContext.restoreGraphicsState()

let outURL = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
  .appendingPathComponent("dmg-background.png")

guard let png = rep.representation(using: .png, properties: [:]) else {
  fputs("Failed to encode PNG\n", stderr)
  exit(1)
}
try png.write(to: outURL)
print("Wrote \(outURL.path) (\(Int(width))×\(Int(height)) px, \(Int(windowW))×\(Int(windowH)) pt window)")