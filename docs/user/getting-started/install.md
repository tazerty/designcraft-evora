---
title: Install EvoDesign
description: Download EvoDesign 0.7.1 for Windows 11, macOS or Linux, or install it from Upstream Launcher.
order: 20
verified: 0.7.1
---

Download the file for your system from the [latest release](https://github.com/tazerty/evodesign/releases/latest) and install it. In the file names, `<version>` stands for the version number, for example `0.7.1`. The release also has `SHA256SUMS.txt`, which lists a checksum for every file. The [EvoDesign page on upstream-site.com](https://evora.st) links to the download as well.

## Windows 11

1. Open the [latest release](https://github.com/tazerty/evodesign/releases/latest).
2. Download `evodesign-<version>-windows-x64.msi`. It fits most PCs (64-bit Intel and AMD).
3. Run the file and follow the installer.
4. Start **EvoDesign** from the Start menu.

The release also has an `arm64` installer for ARM PCs such as Snapdragon, and a 32-bit `x86` installer. To run without installing, download `evodesign-<version>-windows-x64-portable.zip` (or the `arm64` or `x86` version), unzip it and start the program inside. The installers and executables are code-signed.

## macOS

1. Download `evodesign-<version>-macos-universal.dmg`. It works on Apple silicon and Intel Macs.
2. Open it and drag EvoDesign to Applications.

The app is signed and notarized.

## Linux

Pick one file from the release. Replace `x86_64` with `aarch64` on ARM.

| File                                          | Notes                                                    |
|-----------------------------------------------|----------------------------------------------------------|
| `evodesign-<version>-linux-x86_64.AppImage` | Runs anywhere.                                           |
| `evodesign-<version>-linux-x86_64.flatpak`  | Sandboxed. Install with `flatpak install --user <file>`. |
| `evodesign-<version>-linux-x86_64.deb`      | Debian and Ubuntu.                                       |
| `evodesign-<version>-linux-x86_64.rpm`      | Fedora, RHEL and openSUSE.                               |
| `evodesign-<version>-linux-x86_64.tar.gz`   | Unpack anywhere.                                         |

For RISC-V there is a `.deb`, an `.rpm` and a tarball, named with `riscv64` in place of `x86_64`. There is no AppImage or Flatpak for RISC-V. The RISC-V build needs glibc 2.39 or newer (Ubuntu 24.04 or newer). A FreeBSD build (`evodesign-<version>-freebsd-x86_64.tar.gz`) is on the release page too.

## Install with Upstream Launcher

Upstream Launcher lists EvoDesign and can install and update it for you. See [Upstream Launcher](https://evora.st) for how it works.

## First run

Open EvoDesign. You land on the start screen. Continue with [Getting started](index.md). If the window closes right away, see [EvoDesign closes right after it starts](../troubleshooting/app-closes-at-start.md).
