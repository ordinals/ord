Settings
========

`ord` can be configured with the command line, environment variables, a
configuration file, and default values.

The command line takes precedence over environment variables, which take
precedence over the configuration file, which takes precedence over defaults.

The path to the configuration file can be given with `--config <CONFIG_PATH>`.
`ord` will error if `<CONFIG_PATH>` doesn't exist.

The path to a directory containing a configuration file name named `ord.yaml`
can be given with `--config-dir <CONFIG_DIR_PATH>` or `--data-dir
<DATA_DIR_PATH>` in which case the config path is `<CONFIG_DIR_PATH>/ord.yaml`
or `<DATA_DIR_PATH>/ord.yaml`. It is not an error if it does not exist.

If none of `--config`, `--config-dir`, or `--data-dir` are given, and a file
named `ord.yaml` exists in the default data directory, it will be loaded.

For a setting named `--setting-name` on the command line, the environment
variable will be named `ORD_SETTING_NAME`, and the config file field will be
named `setting_name`. For example, the data directory can be configured with
`--data-dir` on the command line, the `ORD_DATA_DIR` environment variable, or
`data_dir` in the config file.

See `ord --help` for documentation of all the settings.

`ord`'s current configuration can be viewed as JSON with the `ord settings`
command.

Example Configuration
---------------------

```yaml
{{#include ../../../ord.yaml}}
```

Hiding Inscription Content
--------------------------

Inscription content can be selectively prevented from being served by `ord
server`.

Unlike other settings, this can only be configured with the configuration file
or environment variables.

To hide inscriptions with an environment variable:

```
export ORD_HIDDEN='6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0 703e5f7c49d82aab99e605af306b9a30e991e57d42f982908a962a81ac439832i0'
```

Or with the configuration file:

```yaml
hidden:
- 6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0
- 703e5f7c49d82aab99e605af306b9a30e991e57d42f982908a962a81ac439832i0
```
