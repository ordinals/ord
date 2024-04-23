Batch Inscribing
================

Multiple inscriptions can be created at the same time using the
[pointer field](./../inscriptions/pointer.md). This is especially helpful for
collections, or other cases when multiple inscriptions should share the same
parent, since the parent can passed into a reveal transaction that creates
multiple children.

To create a batch inscription using a batchfile in `batch.yaml`, run the
following command:

```bash
ord wallet batch --fee-rate 21 --batch batch.yaml
```

Example `batch.yaml`
--------------------

```yaml
{{#include ../../../batch.yaml}}
```
