# EvoDesign app icon

**A calico cat wearing a polka-dot scarf**, as an engraving-style head-and-shoulders portrait in the
Building Apps' owl-template framing: a full-bleed colour field, the cat in three-quarter view, its
shoulders running off the bottom edge.

## Palette

Exactly three colours:

| Colour | Hex | Used for |
|---|---|---|
| Ink | `#0b0b0c` | Line work and contour |
| Paper | `#efe9dc` | The figure (the cat and its scarf) |
| EvoDesign green (app colour) | `#7bb51c` | The full-bleed background field |

## Geometry

A 512 × 512 viewBox, clipped to a rounded square with `rx=112`. Windows and Linux use the full tile. The
macOS icons (`.icns`, `evodesign-macos-512.png`) sit on Apple's grid: an 824/1024 body with a
transparent margin.

## Provenance

The owner drew the cat in Upstream. The 2880 px drawing, keyed to the three-colour palette, lives
outside this repo in evorules at `assets/app-icons/evodesign/source.png`. It was vectorised with
evorules' `assets/logo-options/_tools/vectorize_tile.py`: `evodesign.svg` is traced at 2048 px and
`evodesign-small.svg` at 1024 px. Design rules: evorules `standards/icon-design.md`.

## Files

| File | Use |
|---|---|
| `evodesign.svg` | Canonical artwork (about 680 KB) |
| `evodesign-small.svg` | Lighter vector (about 350 KB); also `hicolor/scalable` |
| `evodesign-1024.png` | 1024 px render of the full tile |
| `evodesign-macos-512.png` | Runtime Dock icon on macOS (embedded by `apps/evodesign`) |
| `evodesign.icns` | macOS bundle icon (`CFBundleIconFile`) |
| `evodesign.ico` | Windows exe icon, 16–256 px (embedded by `apps/evodesign/build.rs`) |
| `hicolor/<size>/apps/ai.storyteller.evodesign.png` | Linux icon theme, 16–512 px; the 256 px one is also the runtime window icon on Windows and Linux |
| `hicolor/scalable/apps/ai.storyteller.evodesign.svg` | Linux scalable icon |
| `LICENSE.txt` | MIT OR Apache-2.0 |

The app ID is `ai.storyteller.evodesign`. It's used for the hicolor icon names, the Wayland app ID and
`packaging/linux/ai.storyteller.evodesign.desktop`.

## Regenerate

Edit `evodesign.svg` (or `evodesign-small.svg`), then run `packaging/icons.sh`. It needs `resvg`, and
`iconutil` for the `.icns` (macOS only). It packs the `.ico` with `cargo xtask ico`.
