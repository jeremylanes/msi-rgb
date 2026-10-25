<!-- bmad:context -->
<!-- Verified 2026-10-25 against ea39160. Managed by bmad-project-context; edits inside this block are replaced on refresh. Keep anything you want preserved outside the markers. -->

## msi_rgb

Native Linux RGB keyboard controller for MSI Katana 15 HX laptops (MS-1565 / VID 1462, PID 1601). Rust 2024 edition, GTK4, direct Linux HID Feature Reports via `/dev/hidraw*` with hardware flash persistence. Planning and BMad artifacts live in `_bmad-output/`.

## Policy

- Never execute git commands; inspect files directly under `.git/` if metadata is needed.
- Never read, create, or modify any `.env` or `env` file or directory.
- Write all code and code comments in English; provide explanations to user in French.
- No emojis anywhere in code, comments, or documentation.
- Never execute database migrations.

## Where things are

- Native hardware communication: `src/backend/native_hid.rs` (Feature Reports `0x01`, `0x02`, `0xA0`)
- Background worker & color ticker: `src/main.rs` and `src/app/service.rs`
- User interface & color wheel: `src/ui/`
- Profiles and atomic XDG storage: `src/profiles/` and `src/persistence/storage.rs`
- All operational scripts: `bin/`

## Running and verifying

- Run `./bin/run-tests` to verify formatting, clippy (`-D warnings`), unit/integration tests, and doc-tests in a single run.
- Deploy user release binary and desktop assets via `./bin/install.sh --user` (`~/.local/bin/msi-rgb`).
- Offline simulation without physical hardware: run with `--mock`.

## Conventions that differ from defaults

- Put all operational, maintenance, and test scripts in `bin/`.
- Every public module, struct, enum, and function must include doc comments (`///`) with at least one usage example.
- Full keyboard synchronization must use Mode 1 Steady across zone mask `0x0F`; do not use hardware Cycle Mode 3 which injects hardcoded MCU phase delays.

## Known pitfalls

- Do not re-introduce OpenRGB; hardware communication is strictly direct Linux HID.
- High-frequency color tickers must never emit `0xA0` (Save to Flash) to avoid hardware wear and visual stuttering.
- Applying dynamic profiles must launch the detached background daemon (`--daemon`) via `libc::setsid()` so effects persist when terminal/GUI exits.

<!-- /bmad:context -->
