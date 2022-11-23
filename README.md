# dwmbar
A dwm statusbar written in Rust. It avoids running external shell commands for module inputs and
instead gathers the data itself from the unix virtual filesystem.

Beside regular modules that are updated in pre-defined intervals, it also supports updating
specific modules asynchronously. For example, the volume widget listens for pulseaudio volume
change events and triggers an update accordingly.

`dwmbar` is mainly written to suit my specific setup and has personal configuration hardcoded,
however the code is laid out in a generic way such that writing new modules or modifying existing
ones is easy.

## Modules
The following modules are currently available:
- datetime
- battery
- cputemperature
- loadavg
- memory usage
- volume
- vpn
- a helper module to combine module outputs
