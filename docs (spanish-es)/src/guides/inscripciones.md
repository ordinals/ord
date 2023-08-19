Guía de Inscripción Ordinal
=========================

Los sats individuales pueden inscribirse con contenidos arbitrarios, creando
artefactos digitales nativos de Bitcoin que pueden ser conservados en un monedero Bitcoin y
transferidos usando transacciones de Bitcoin. Las inscripciones son duraderas, inmutables,
seguras y descentralizadas, como el propio Bitcoin.

Trabajar con inscripciones requiere un nodo completo de Bitcoin para darte una visión del
estado actual de la blockchain de Bitcoin, y un monedero que pueda crear
inscripciones y ejecutar el control de sats al construir transacciones para enviar
inscripciones a otro monedero.

Bitcoin Core proporciona tanto un nodo completo de Bitcoin como un monedero. Sin embargo, el monedero
de Bitcoin Core no puede crear inscripciones y no realiza el control de sats.

Para esto es necesario [`ord`](https://github.com/ordinals/ord), la utilidad ordinal. `ord`
no implementa su propio monedero, por lo que los subcomandos `ord wallet` interactúan con
los monederos de Bitcoin Core.

Esta guía cubre:

1. Instalación de Bitcoin Core
2. Sincronización de la blockchain de Bitcoin
3. Creación de un monedero Bitcoin Core
4. Uso de `ord wallet receive` para recibir sats
5. Creación de inscripciones con `ord wallet inscribe`
6. Envío de inscripciones con `ord wallet send`
7. Recepción de inscripciones con `ord wallet receive`

Obtener Ayuda
------------

Si encuentras dificultades, intenta pedir ayuda en el [Server Discord de Ordinals](https://discord.com/invite/87cjuz4FYg), o revisar en GitHub para temas relevantes
[problemas](https://github.com/ordinals/ord/issues) e
[discusiones](https://github.com/ordinals/ord/discussions).

Instalación de Bitcoin Core
-----------------------

Bitcoin Core está disponible en [bitcoincore.org](https://bitcoincore.org/) en la
[página de descarga](https://bitcoincore.org/en/download/).

Crear inscripciones requiere Bitcoin Core 24 o versiones posteriores.

Esta guía no cubre en detalle la instalación de Bitcoin Core. Una vez que hayas instalado Bitcoin Core,
deberías ser capaz de ejecutar exitosamente `bitcoind -version` desde la línea
de comandos.

Configuración de Bitcoin Core
------------------------

`ord` requiere el índice de transacciones de Bitcoin Core.

Para configurar tu nodo Bitcoin Core para mantener un índice de transacciones,
añade lo siguiente a tu `bitcoin.conf`:

```
txindex=1
```

O ejecuta `bitcoind` con `-txindex`:

```
bitcoind -txindex
```

Sincronización de la Blockchain de Bitcoin
------------------------------

Para sincronizar la cadena, ejecuta:

```
bitcoind -txindex
```

...y déjalo en ejecución hasta que  `getblockcount`:

```
bitcoin-cli getblockcount
```

coincida con el recuento de bloques en un explorador de bloques como
[el explorador de bloques mempool.space](https://mempool.space/). `ord` interactúa con `bitcoind`, por lo que
deberías dejar `bitcoind` ejecutándose en segundo plano cuando estés usando `ord`.

Instalación de  `ord`
----------------

La utilidad `ord` está escrita en Rust y puede ser compilada desde
[source](https://github.com/ordinals/ord). Los binarios precompilados están disponibles en la
[página de lanzamientos](https://github.com/ordinals/ord/releases).

Puedes instalar el último binario precompilado desde la línea de comandos con:

```sh
curl --proto '=https' --tlsv1.2 -fsLS https://ordinals.com/install.sh | bash -s
```

Una vez instalado `ord`, deberías ser capaz de ejecutar:

```
ord --version
```

Esto mostrará el número de versión de `ord`.

Creación de un Monedero Bitcoin Core
------------------------------

`ord` utiliza Bitcoin Core para gestionar las claves privadas, firmar transacciones y
transmitir las transacciones a la red Bitcoin.

Para crear una cartera Bitcoin Core llamada `ord` para usar con `ord`, ejecuta:

```
ord wallet create
```

Recibir Sats
--------------

Las inscripciones se realizan sobre sats individuales, utilizando transacciones Bitcoin normales
que pagan comisiones en sats, así que tu cartera necesitará algunos sats.

Obtén una nueva dirección de tu cartera `ord` ejecutando:


```
ord wallet receive
```

Y envía fondos a esa dirección.

Puedes ver las transacciones pendientes con:

```
ord wallet transactions
```

Una vez confirmada la transacción, deberías poder ver las transacciones
salientes con `ord wallet outputs`.

Creación de Contenido de Inscripción
----------------------------

Los sats pueden ser inscritos con cualquier tipo de contenido, pero la cartera `ord` solo
admite tipos de contenido que pueden ser visualizados por el explorador de bloques `ord`.

Además, las inscripciones se incluyen en las transacciones, por lo que cuanto mayor sea el
contenido, mayor será la comisión que la transacción de inscripción debe pagar.

El contenido de la inscripción se incluye en los testigos de la transacción, que reciben un
descuento del testigo. Para calcular la comisión aproximada que una transacción de inscripción
pagará, divide el tamaño del contenido por cuatro y multiplica por la tasa de comisión.

Las transacciones de inscripción deben ser menores a 400.000 unidades de peso, o no serán
retransmitidas por Bitcoin Core. Un byte de contenido de inscripción cuesta una
unidad de peso. Dado que una transacción de inscripción incluye no solo el contenido de la inscripción,
limita el contenido de la inscripción a menos de 400.000 unidades de peso. 390.000
unidades de peso deberían ser seguras.

Creación de Inscripciones
---------------------

Para crear una inscripción con el contenido de `FILE`, ejecuta:


```
ord wallet inscribe --fee-rate FEE_RATE FILE
```

Ord devolverá dos IDs de transacción, uno para la transacción de compromiso y uno
para la transacción de revelación, y el ID de la inscripción. Los ID de las inscripciones tienen el
formato `TXIDiN`, donde `TXID` es el ID de la transacción de revelación,
y `N` es el índice de la inscripción en la transacción de revelación.

La transacción de compromiso se compromete a un tapscript que contiene el contenido de la inscripción,
y la transacción de revelación gasta de ese tapscript, revelando
el contenido en la cadena e inscribiéndolo en el primer sat del input que
contiene el tapscript correspondiente.

Espera a que la transacción de revelación sea minada. Puedes verificar el estado del
compromiso y de las transacciones de revelación usando 
[el explorador de bloques mempool.space.](https://mempool.space/).

Una vez que la transacción de revelación ha sido minada, el ID de la inscripción debería ser
mostrado al ejecutar:

```
ord wallet inscriptions
```

Y cuando visites [el explorador ordinal](https://ordinals.com/) en
`ordinals.com/inscription/INSCRIPTION_ID`.

Envío de Inscripciones
--------------------

Pide al destinatario que genere una nueva dirección ejecutando:

```
ord wallet receive
```

Envía la inscripción ejecutando:

```
ord wallet send --fee-rate <FEE_RATE> <ADDRESS> <INSCRIPTION_ID>
```

Ve la transacción pendiente con:

```
ord wallet transactions
```

Una vez confirmada la transacción de envío, el destinatario puede confirmar la recepción ejecutando:

```
ord wallet inscriptions
```

RRecepción de Inscripciones
----------------------

Genera una nueva dirección de recepción usando:

```
ord wallet receive
```

El remitente puede transferir la inscripción a tu dirección usando:

```
ord wallet send ADDRESS INSCRIPTION_ID
```

Ve la transacción pendiente con:
```
ord wallet transactions
```

Una vez confirmada la transacción de envío, puedes confirmar la recepción ejecutando:

```
ord wallet inscriptions
```
