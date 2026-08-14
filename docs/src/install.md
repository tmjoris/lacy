# Install

After installation, don't forget to do [setup](./setup.md)!

- [Cargo (Universal)](#cargo-universal)
- [Homebrew (macOS/Linux)](#homebrew-macoslinux)
- [NixOS ❄️](#nixos-️)
  - [1. Home Manager Module (Recommended)](#1-home-manager-module-recommended)
  - [2. Nix Overlay (NixOS / Home Manager)](#2-nix-overlay-nixos--home-manager)
    - [For NixOS](#for-nixos)
    - [For Home Manager](#for-home-manager)
  - [3. Run `lacy` directly](#3-run-lacy-directly)


## Cargo (Universal)

```sh
cargo install lacy
```

## Homebrew (macOS/Linux)

```sh
brew install timothebot/tap/lacy
```

### AUR (Arch Linux)

```sh
yay -S lacy # or whatever aur helper you're using
```

## Windows (x64 & Windows on Arm)

```sh
cargo install lacy
```

Prebuilt binaries are also published on the [GitHub Releases](https://github.com/timothebot/lacy/releases) page for both `x86_64-pc-windows-msvc` (Windows x64) and `aarch64-pc-windows-msvc` (Windows on Arm — runs natively on Snapdragon X and other Arm64 devices, no x64 emulation). Download the `.zip` matching your architecture, extract `lacy.exe` to a folder on your `PATH`, then continue with [setup](./setup.md#powershell).

## NixOS ❄️

There are a couple of ways to get `lacy` up and running on your Nix system. Both methods require adding `lacy` to your `flake.nix` inputs.

```nix
# flake.nix
{
  inputs = {
    lacy.url = "github:timothebot/lacy";
    # ... other inputs
  };

  outputs = { self, nixpkgs, lacy, ... }@inputs: {
    # ...
  };
}
```

### 1. Home Manager Module (Recommended)

This is the easiest way to manage `lacy` if you're using Home Manager. The module handles most of the setup for you. You **must** explicitly add the `lacy` flake as an **overlay** to your configuration to ensure the `lacy` package is available.

```nix
# home.nix
{ pkgs, inputs, ... }: {
  # Explicitly add the overlay to make lacy visible in your pkgs set
  nixpkgs.overlays = [ inputs.lacy.overlays.default ];

  imports = [
    inputs.lacy.homeManagerModules.default
  ];

  programs.lacy.enable = true;
}
```

### 2. Nix Overlay (NixOS / Home Manager)

If you prefer to manage packages directly without using the Home Manager module, this is the way to go. You apply the overlay and then add the `lacy` package to your system or user packages.

#### For NixOS

```nix
# In your NixOS configuration (e.g., /etc/nixos/configuration.nix)
{ pkgs, inputs, ... }: {
  nixpkgs.overlays = [ inputs.lacy.overlays.default ];

  environment.systemPackages = [ pkgs.lacy ];
}
```

#### For Home Manager

```nix
# In your home-manager configuration (e.g., ~/.config/nixpkgs/home.nix)
{ pkgs, inputs, ... }: {
  nixpkgs.overlays = [ inputs.lacy.overlays.default ];

  home.packages = [ pkgs.lacy ];
}
```

### 3. Run `lacy` directly

Just want to try it out? You can run `lacy` from the command line without adding it to your system configuration.

```bash
nix run github:timothebot/lacy
```
