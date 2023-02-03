Moderation
==========

`ord` includes a block explorer, which you can run locally with `ord server`.

The block explorer allows viewing inscriptions. Inscriptions are user-generated
content, which may be objectionable or unlawful.

It is the responsibility of each individual who runs an ordinal block explorer
instance to understand their responsibilities with respect to unlawful content,
and decide what moderation policy is appropriate for their instance.

In order to prevent particular inscriptions from being displayed on an `ord`
instance, they can be included in a YAML config file, which is loaded with the
`--config` option.

To hide inscriptions, first create a config file, with the inscription ID you
want to hide:

```yaml
hidden:
- 0000000000000000000000000000000000000000000000000000000000000000i0
```

The suggested name for `ord` config files is `ord.yaml`, but any filename can
be used.

Then pass the file to `--config` when starting the server:

`ord --config ord.yaml server`

Note that the `--config` option comes after `ord` but before the `server`
subcommand.

`ord` must be restarted in to load changes to the config file.

`ordinals.com`
--------------

The `ordinals.com` instances use `systemd` to run the `ord server` service,
which is called `ord`, with a config file located at `/var/lib/ord/ord.yaml`.

To hide an inscription on `ordinals.com`:

1. SSH into the server
2. Add the inscription ID to `/var/lib/ord/ord.yaml`
3. Restart the service with `systemctl restart ord`
4. Monitor the restart with `journalctl -u ord`

Currently, `ord` is slow to restart, so the site will not come back online
immediately.
