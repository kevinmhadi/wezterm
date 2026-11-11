---
tags:
  - clipboard
---
# `allow_osc52_clipboard_read`

{{since('nightly')}}

Controls whether applications can read the system clipboard using OSC 52 escape
sequences. This is a boolean setting; the default is `false`.

- `false`: ignore clipboard-read requests without replying.
- `true`: read the clipboard asynchronously and send an OSC 52 response to the
  requesting pane.

## Enable clipboard reads

Add `config.allow_osc52_clipboard_read = true` before `return config` in your
`wezterm.lua`. A complete minimal configuration is:

```lua
local wezterm = require 'wezterm'
local config = wezterm.config_builder()

config.allow_osc52_clipboard_read = true

return config
```

Set it to `false`, or remove the assignment, to disable reads again. This setting
does not change OSC 52 clipboard writes or user-initiated paste actions.

## SSH and mux domains

Configure this on the machine running the WezTerm GUI, which owns the clipboard.
For example, to let a Linux application read your Mac clipboard over SSH, enable
it in the Mac's `wezterm.lua`. No X forwarding or Linux clipboard utility is
required; the application must issue an OSC 52 clipboard query.

For WezTerm mux domains, enabling the setting only on the server is not enough:
the GUI client's effective window configuration controls access. Both client
and server must use a build supporting clipboard queries (mux codec version 47).
Multiple attached GUI clients with reads enabled may each reply with their own
clipboard contents.

## Security

**Enable only for trusted sessions.** Any program able to write escape sequences
to an enabled window's panes can read the clipboard without prompting. This
includes remote programs over SSH and escape sequences in files printed by
commands such as `cat`. Clipboard contents can include passwords or other secrets.

Permission and pane ownership are checked before reading and again before
sending the result. Disabled requests and failed reads produce no reply, so
applications should use a timeout. There is no separate configurable OSC 52
read-timeout setting.

See [OSC 52 clipboard queries](../../../escape-sequences.md#osc-52-clipboard-queries)
for request syntax, selection handling, and response encoding.
