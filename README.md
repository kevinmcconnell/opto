# Opto

See how busy your machine has been, and why.

![Opto showing CPU, clock, temperature, memory, network, disk, GPU and agent charts beside the top processes](.github/assets/screenshot.png)

Opto is a small system activity recorder and viewer for a single-user Linux
desktop. It has two parts:

- **`opto-collect`** samples CPU load and clock speed, temperatures,
  memory, network, disk, GPU and per-process CPU time every five seconds into a SQLite database. It runs as
  a systemd user service, reads only `/proc`, `/sys` and the local logs of
  AI coding agents, and keeps a rolling window of five days.
- **`opto`** is a GTK 4 / libadwaita app. Pick a time range and see charts of
  that activity, plus the processes that used the most CPU in that range.

On Omarchy, the app follows the active theme. Elsewhere it uses the Adwaita
look and the desktop's light or dark preference.

## Building

Opto is written in Rust and needs the `gtk4`, `libadwaita` and `sqlite`
libraries plus `pkgconf`. On Arch:

```
sudo pacman -S --needed rust pkgconf gtk4 libadwaita sqlite
```

Then:

```
make install
```

This builds in release mode, installs both binaries to `~/.local/bin`,
installs the desktop file, icon and user service, starts the collector, and
sets up the [Claude Code status line](#ai-agents).
Run `make uninstall` to remove it all again.

The last two steps are `opto setup`, which you can run yourself at any time.
`opto setup service` enables and starts the collector's user service, and
`opto setup claude` sets up the status line; plain `opto setup` does both,
skipping Claude Code if it is not installed. `opto remove`, `opto remove
service` and `opto remove claude` undo them. Removing the service keeps the
recorded data. Setup restarts a running collector only when the installed
binary differs from the one it runs, so running it again leaves no gap in the
charts.

To stage a system-wide install for a package, without the setup step:

```
make install-files DESTDIR="$pkgdir" PREFIX=/usr UNITDIR=/usr/lib/systemd/user
```

## Data

The database lives at `~/.local/share/opto/opto.db`. Run
`opto-collect --help` for the sampling interval and retention flags, and
`opto --help` for the viewer's options.

To keep the disk quiet, the collector writes each sample to a file under
`$XDG_RUNTIME_DIR/opto/`, which is on tmpfs, and moves the samples into
`opto.db` every five minutes. The app reads both, but `opto.db` on its own
can be up to five minutes behind. [AGENTS.md](AGENTS.md) has the details.

## Other machines

The app can show another machine's history over ssh:

```
opto me@host
```

You can also type an ssh destination in the machine picker in the header,
which keeps a list of recent machines. The other machine needs Opto
installed and collecting, with `opto-collect` in `~/.local/bin` or on the
`PATH`. The queries run there, and the app asks for any ssh password or
security key touch itself.

## AI agents

Opto can chart how much of your Codex and Claude Code rate limits remains,
and the tokens each agent is consuming. Neither needs network access or
credentials. Both cards appear once an agent has reported something.

**Codex** needs no setup. The collector tails the session logs under
`~/.codex/sessions` (or `$CODEX_HOME/sessions`), which carry a quota snapshot
and a token count with every response.

**Claude Code** does not persist its quota anywhere, but it pipes the data to a
status line command after every assistant message. `make install` puts
`opto-claude-statusline` in `~/.local/bin`; it saves that JSON under
`~/.local/state/opto/claude/`, one line per update, and prints nothing.

`make install` also runs `opto setup claude`, which sets the status line
in `~/.claude/settings.json` (or `$CLAUDE_CONFIG_DIR/settings.json`):

```json
{
  "statusLine": {
    "type": "command",
    "command": "opto-claude-statusline"
  }
}
```

If the helper's directory is not on your `PATH`, it writes the helper's full
path instead. It is safe to run again: it adds only that key, leaves the rest
of the file as it was, and does nothing if it is already set. If you already
have a different status line it leaves it alone and warns; add a line to your
script that pipes its input through `opto-claude-statusline` as well.

`opto remove claude`, which `make uninstall` runs, takes the key out again if
it still runs only `opto-claude-statusline`, and leaves any other status line
alone.

The rate-limit fields appear only on claude.ai Pro and Max subscriptions.

## Releasing

The version is set in one place, `[workspace.package]` in `Cargo.toml`.

1. Set the version in `Cargo.toml`, then run `cargo build` to update
   `Cargo.lock`.
2. Commit the two files as "Release opto 0.2.0".
3. Tag the commit `v0.2.0` and push the commit and the tag.

CI checks that the tag matches the version, runs the checks, and publishes
the GitHub release with `opto-0.2.0-x86_64.tar.gz`, a staged `/usr` tree, and
its `.sha256`.

Then update the [`opto-bin`](https://aur.archlinux.org/packages/opto-bin)
AUR package: set `pkgver`, reset `pkgrel` to 1, put the new checksum in
`sha256sums`, run `makepkg --printsrcinfo > .SRCINFO`, and commit and push.

## License

MIT. See [LICENSE](LICENSE).
