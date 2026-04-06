# `trail_lab`

Crate-local verification app for the `trail` shared crate.

## Run

```bash
cargo run -p saddle-rendering-trail-lab
```

With BRP on the default lab port:

```bash
TRAIL_LAB_BRP_PORT=15752 cargo run -p saddle-rendering-trail-lab
```

Timed exit for batch checks:

```bash
TRAIL_LAB_EXIT_AFTER_SECONDS=3 cargo run -p saddle-rendering-trail-lab
```

## E2E

```bash
cargo run -p saddle-rendering-trail-lab --features e2e -- trail_smoke
cargo run -p saddle-rendering-trail-lab --features e2e -- trail_billboard
cargo run -p saddle-rendering-trail-lab --features e2e -- trail_locked
cargo run -p saddle-rendering-trail-lab --features e2e -- trail_reset
cargo run -p saddle-rendering-trail-lab --features e2e -- trail_view_source
```

## BRP

```bash
TRAIL_LAB_BRP_PORT=15752 cargo run -p saddle-rendering-trail-lab
BRP_PORT=15752 uv run --project .codex/skills/bevy-brp/script brp world query bevy_ecs::name::Name
BRP_PORT=15752 uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/trail_lab.png
BRP_PORT=15752 uv run --project .codex/skills/bevy-brp/script brp extras shutdown
```
