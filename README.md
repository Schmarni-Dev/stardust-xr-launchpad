# Launchpad

a cli tool that helps you setup a fully featured StardustXR session without too much hassle

## Basic Usage

run `stardust-xr-launchpad start -- stardust-xr-server` in your tty to setup the main process that manages everything

run `stardust-xr-launchpad igniter` once your OpenXR runtime is ready

run `stardust-xr-launchpad server-started STARDUST_INSTANCE WAYLAND_DISPLAY MOZ_ENABLE_WAYLAND DISPLAY XDG_SESSION_TYPE` in your stardust server config shell script to update the env-vars for systemd services

## License

Unless otherwise specified, all code in this repository is licensed under
the MIT License.

Any contribution intentionally submitted for inclusion in the work by you, shall be
licensed as MIT, without any additional terms or conditions.
