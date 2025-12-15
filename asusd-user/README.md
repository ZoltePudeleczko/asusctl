# daemon-user

This crate is for the binary of `asusd-user` and its helper lib.

The purpose of `asusd-user` is to run in userland and provide the user + third-party apps an interface for RGB keyboard per-key lighting and effects.

`asusd-user` should try to be as simple as possible while allowing a decent degree of control.

## TODO

- [ ] CLI for basic settings/interaction
- [ ] RGB keyboard per-key programs
- [ ] User profiles (fan, cpu etc). These would be replacing the system-daemon profiles only when the user is active, otherwise system-daemon defaults to system settings.
