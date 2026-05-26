# Liquid Control

This folder contains the editable control scripts for the installation runtime.

- `./start`: create the detached renderer tmux session.
- `./attach`: attach to the renderer tmux session.
- `./stop`: stop the renderer tmux session.
- `./doctor`: run diagnostics.
- `./update`: intentionally pull and rebuild the baked repo checkout.
- `./config`: edit renderer defaults.
- `./bluetooth`: pair, trust, and connect a Bluetooth keyboard or device.

The repo checkout lives at:

```text
~/liquid
```

The renderer is started in tmux session:

```text
liquid
```
