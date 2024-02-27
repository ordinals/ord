Settings
========

`ord` can be configured with command line options, environment variables, a
configuration file, and default values.

When multiple sources configure the same thing, precedence is in order of
command line options, then environment variables, then the configuration file,
and finally default values.

The path to the configuration can be given with `--config <CONFIG_PATH>`. `ord`
will error if `<CONFIG_PATH>` doesn't exist. The path to a configuration
directory can be given with `--config-dir <CONFIG_DIR_PATH>`, in which case the
config path is `<CONFIG_DIR_PATH>/ord.yaml`. It is not an error if
`<CONFIG_DIR_PATH>/ord.yaml` does not exist, and `ord` will use a configuration
with default values.

All settings can be configured with command line options, but not all settings
can yet be configured with environmnet variables or a configuration file.

`ord`'s configuration can be viewd as JSON with `ord settings`.

| setting | CLI | environment variable | default value |
| --- | --- | --- | --- |
| bitcoin RPC password | `--bitcoin-rpc-pass <PASSWORD>` | `ORD_BITCOIN_RPC_PASS` | none |
| bitcoin RPC username | `--bitcoin-rpc-user <USERNAME>` | `ORD_BITCOIN_RPC_USER` | none |
| chain | `--chain <CHAIN>` | `ORD_CHAIN` | mainnet |
