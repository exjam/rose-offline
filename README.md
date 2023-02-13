# rose-offline

An open source server for ROSE Online, compatible with the official 129_129en irose client or [rose-offline-client](https://github.com/exjam/rose-offline-client).

# Running the server
Run rose-offline-server from your installed official client directory (the folder containing data.idx), or you can use the `--data-idx` or `--data-path` arguments as described below.

## Optional arguments:
- `--data-idx=<path/to/data.idx>` Path to irose 129en data.idx
- `--data-path=<path/to/data>` Path to extracted irose 129en game files
- `--ip=<ip>` IP to listen for client connections, defaults to 127.0.0.1
