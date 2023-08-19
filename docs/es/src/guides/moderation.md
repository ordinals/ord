Moderación
==========

`ord` incluye un explorador de bloques, que puedes ejecutar localmente con `ord server`.

El explorador de bloques permite ver las inscripciones. Las inscripciones son contenido generado por los usuarios, que puede ser objetable o ilegal.

Es responsabilidad de cada individuo que ejecuta una instancia del explorador de bloques ordinal entender sus responsabilidades con respecto al contenido ilegal y decidir qué política de moderación es apropiada para su instancia.

Para evitar que se muestren inscripciones particulares en una instancia de `ord`, se pueden incluir en un archivo de configuración YAML, que se carga con la opción `--config`.

Para ocultar inscripciones, primero crea un archivo de configuración con la ID de inscripción que quieres ocultar:

```yaml
hidden:
- 0000000000000000000000000000000000000000000000000000000000000000i0
```

El nombre sugerido para los archivos de configuración de `ord` es `ord.yaml`, pero se puede utilizar cualquier nombre de archivo.

Luego pasa el archivo a `--config` al iniciar el servidor:

`ord --config ord.yaml server`

Ten en cuenta que la opción `--config` viene después de `ord` pero antes del subcomando `server`.

`ord` debe reiniciarse para cargar los cambios en el archivo de configuración.

`ordinals.com`
--------------

Las instancias de `ordinals.com` utilizan `systemd` para ejecutar el servicio `ord server`, que se llama `ord`, con un archivo de configuración ubicado en `/var/lib/ord/ord.yaml`.

Para ocultar una inscripción en `ordinals.com`:

1. Conéctate al servidor mediante SSH
2. Agrega la ID de la inscripción a `/var/lib/ord/ord.yaml`
3. Reinicia el servicio con `systemctl restart ord`
4. Supervisa el reinicio con `journalctl -u ord`

Actualmente, `ord` tarda en reiniciarse, por lo que el sitio no volverá a estar en línea inmediatamente.
