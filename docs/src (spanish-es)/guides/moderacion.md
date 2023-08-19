Moderación
==========

`ord` incluye un explorador de bloques, que puedes ejecutar localmente con `ord server`.

El explorador de bloques permite visualizar las inscripciones. Las inscripciones 
son contenidos generados por los usuarios, que podrían ser discutibles o ilegales.

Es responsabilidad de cada individuo que maneja una instancia del explorador de bloques ordinales 
entender sus responsabilidades con respecto a los contenidos ilegales 
y decidir qué política de moderación es apropiada para su instancia.

Para evitar que ciertas inscripciones se muestren en una instancia `ord`, 
estas pueden ser incluidas en un archivo de configuración YAML, 
que se carga con la opción `--config`.

Para ocultar las inscripciones, primero crea un archivo de configuración 
con el ID de la inscripción que deseas ocultar:

```yaml
hidden:
- 0000000000000000000000000000000000000000000000000000000000000000i0
```

El nombre sugerido para los archivos de configuración `ord` es `ord.yaml`, 
pero cualquier nombre de archivo puede ser utilizado.

Luego pasa el archivo a `--config` cuando inicies el servidor:

`ord --config ord.yaml server`

Nota que la opción `--config` va después de `ord` pero antes del subcomando `server`.

`ord` debe ser reiniciado para cargar los cambios en el archivo de configuración.

`ordinals.com`
--------------

Las instancias de `ordinals.com` usan `systemd` para ejecutar el servicio `ord server`, 
que se llama `ord`, con un archivo de configuración ubicado en `/var/lib/ord/ord.yaml`.

Para ocultar una inscripción en `ordinals.com`:

1. Inicia sesión SSH en el servidor.
2. Añade el ID de la inscripción a `/var/lib/ord/ord.yaml`.
3. Reinicia el servicio con `systemctl restart ord`.
4. Monitorea el reinicio con `journalctl -u ord`.

Actualmente, `ord` tarda en reiniciar, así que el sitio no volverá a estar en línea inmediatamente.
