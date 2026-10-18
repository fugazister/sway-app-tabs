# sway-app-tabs

Keep every tiled window of chosen apps together in one tabbed container in [sway](https://swaywm.org).

When a new window of a listed app opens, it is moved next to an existing window
of the same app (on whatever workspace that one lives) and the two are wrapped
in a tabbed container that holds only that app's windows. Further windows join
the same group. A lone first window is left untouched, and floating windows
(dialogs) are ignored.

Apps are matched by Wayland `app_id` or, for XWayland windows, X11 class
(find them with `swaymsg -t get_tree`).

## Install

```sh
cargo install --locked --git https://github.com/fugazister/sway-app-tabs
```

## Usage

In your sway config:

```
exec_always ~/.cargo/bin/sway-app-tabs BambuStudio org.example.App
```

Restarting (e.g. on `swaymsg reload`) replaces the previous instance.

It is a single process holding one IPC subscription, using about 3 MB of RAM
and doing work only when a window opens.
