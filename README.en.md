# Redragon Rift GT10: disable the back buttons

A Windows program that turns off the 4 back buttons (paddles / macro buttons) of the Redragon Rift GT10 gamepad. Those buttons repeat X, Y, A and B, and games often pick them up before the front ones, for example when you bind controls.

Windows lists this pad as "G710 USB gaming controller" (ShanWan, USB `2563:0575`), so it is also searched as Redragon GT710 or G710.

## What it does

Windows cannot disable single buttons of a gamepad, and this model has no vendor software for it. The program sits between the pad and the games:

1. It reads the pad directly.
2. It drops the signals of the back buttons.
3. It passes everything else to a virtual Xbox 360 pad (ViGEmBus).
4. It hides the original pad from games (HidHide), so they do not see two pads.

Games see one Xbox 360 pad with the front buttons and without the back ones. It also works in games that only accept Xbox pads.

## Install

1. Run `FiltroMando_Setup_0.1.0.exe`. Windows may warn about an unknown publisher because the installer is not signed.
2. The installer adds ViGEmBus and HidHide if they are missing. It needs administrator rights.
3. If it installed HidHide, restart the PC when asked. Until you restart, games still see the original pad and the back buttons still get through.
4. Open "Filtro de mando" from the Start menu. While it runs, the back buttons do not reach games.

## Build

Requirements: 64-bit Windows (tested on Windows 11), [Rust](https://rustup.rs) (MSVC toolchain) and [Inno Setup 6](https://jrsoftware.org/isinfo.php).

```powershell
.\scripts\descargar-dependencias.ps1
cargo build --release
& "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer\filtro_mando.iss
```

The installer is written to `dist\`.

## Known limits

- Tested with one pad on two PCs running Windows 11, with ViGEmBus 1.22.0 and HidHide 1.5.230. On the second PC, which had neither driver, the installer installed both and the filter worked.
- Not tested on Windows 10 or with other pads.
- The installer is unsigned.
- The UI and the docs are in Spanish. The report layout of the pad is described in [docs/como-funciona.md](docs/como-funciona.md).

## License

MIT for the code. HidHide (MIT) and ViGEmBus (BSD-3-Clause) are bundled in the installer; their licenses are in `licencias-terceros/`.
