Guía de Inscripción Ordinal
===========================

Los sats individuales pueden ser inscritos con contenido arbitrario, creando artefactos digitales nativos de Bitcoin que pueden ser almacenados en una cartera de Bitcoin y transferidos usando transacciones de Bitcoin. Las inscripciones son tan duraderas, inmutables, seguras y descentralizadas como el propio Bitcoin.

Trabajar con inscripciones requiere un nodo completo de Bitcoin, para darte una visión del estado actual de la cadena de bloques de Bitcoin, y una cartera que pueda crear inscripciones y realizar el control de sat al construir transacciones para enviar inscripciones a otra cartera.

Bitcoin Core proporciona tanto un nodo completo de Bitcoin como una cartera. Sin embargo, la cartera de Bitcoin Core no puede crear inscripciones y no realiza el control de sat.

Esto requiere [`ord`](https://github.com/ordinals/ord), la utilidad ordinal. `ord` no implementa su propia cartera, por lo que los subcomandos de `ord wallet` interactúan con las carteras de Bitcoin Core.

Esta guía cubre:

1. Instalar Bitcoin Core
2. Sincronizar la cadena de bloques de Bitcoin
3. Crear una cartera de Bitcoin Core
4. Usar `ord wallet receive` para recibir sats
5. Crear inscripciones con `ord wallet inscribe`
6. Enviar inscripciones con `ord wallet send`
7. Recibir inscripciones con `ord wallet receive`

Obtener Ayuda
------------

Si te quedas atascado, intenta pedir ayuda en el [Servidor de Discord de Ordinals](https://discord.com/invite/87cjuz4FYg), o consultando en GitHub los [problemas](https://github.com/ordinals/ord/issues) y [discusiones](https://github.com/ordinals/ord/discussions) relevantes.

Instalar Bitcoin Core
---------------------

Bitcoin Core está disponible en [bitcoincore.org](https://bitcoincore.org/) en la [página de descarga](https://bitcoincore.org/en/download/).

Hacer inscripciones requiere Bitcoin Core 24 o una versión más reciente.

Esta guía no cubre en detalle la instalación de Bitcoin Core. Una vez instalado Bitcoin Core, deberías poder ejecutar `bitcoind -version` con éxito desde la línea de comandos.

Configurando Bitcoin Core
-------------------------

`ord` requiere el índice de transacciones de Bitcoin Core.

Para configurar tu nodo de Bitcoin Core para mantener un índice de transacciones, añade lo siguiente a tu `bitcoin.conf`:

```
txindex=1
```

O ejecuta `bitcoind` con `-txindex`:

```
bitcoind -txindex
```

Sincronizando la Cadena de Bloques de Bitcoin
--------------------------------------------

Para sincronizar la cadena, ejecuta:

```
bitcoind -txindex
```

...y déjalo ejecutándose hasta que `getblockcount`:

```
bitcoin-cli getblockcount
```

coincida con el recuento de bloques en un explorador de bloques como [el explorador de bloques mempool.space](https://mempool.space/). `ord` interactúa con `bitcoind`, así que debes dejar `bitcoind` ejecutándose en segundo plano cuando estés usando `ord`.

Instalando `ord`
----------------

La utilidad `ord` está escrita en Rust y se puede construir desde la [fuente](https://github.com/ordinals/ord). Los binarios precompilados están disponibles en la [página de lanzamientos](https://github.com/ordinals/ord/releases).

Puedes instalar el último binario precompilado desde la línea de comandos con:

```sh
curl --proto '=https' --tlsv1.2 -fsLS https://ordinals.com/install.sh | bash -s
```

Una vez que `ord` esté instalado, deberías poder ejecutar:

```
ord --version
```

Lo cual imprimirá el número de versión de `ord`.

Creando una Cartera de Bitcoin Core
-----------------------------------

`ord` utiliza Bitcoin Core para gestionar las claves privadas, firmar transacciones y transmitir transacciones a la red de Bitcoin.

Para crear una cartera de Bitcoin Core llamada `ord` para usar con `ord`, ejecuta:

```
ord wallet create
```

Recibiendo Sats
----------------

Las inscripciones se hacen en sats individuales, usando transacciones normales de Bitcoin que pagan comisiones en sats, así que tu cartera necesitará algunos sats.

Obtén una nueva dirección de tu cartera `ord` ejecutando:

```
ord wallet receive
```

Y envíale algunos fondos.

Puedes ver las transacciones pendientes con:

```
ord wallet transactions
```

Una vez que la transacción se confirme, deberías poder ver las salidas de las transacciones con `ord wallet outputs`.

Creando Contenido de Inscripción
----------------------------

Los sats pueden ser inscritos con cualquier tipo de contenido, pero la cartera `ord` solo admite tipos de contenido que pueden ser mostrados por el explorador de bloques `ord`.

Además, las inscripciones se incluyen en las transacciones, por lo que cuanto mayor sea el contenido, mayor será la comisión que debe pagar la transacción de inscripción.

El contenido de la inscripción se incluye en los testigos de la transacción, que reciben el descuento de testigo. Para calcular la comisión aproximada que pagará una transacción de inscripción, divide el tamaño del contenido por cuatro y multiplícalo por la tasa de comisión.

Las transacciones de inscripción deben ser menores de 400,000 unidades de peso, o no serán retransmitidas por Bitcoin Core. Un byte de contenido de inscripción cuesta una unidad de peso. Dado que una transacción de inscripción incluye no solo el contenido de la inscripción, limita el contenido de la inscripción a menos de 400,000 unidades de peso. 390,000 unidades de peso deberían ser seguras.

Creando Inscripciones
---------------------

Para crear una inscripción con el contenido del archivo `FILE`, ejecuta:

```
ord wallet inscribe --fee-rate FEE_RATE FILE
```

Ord mostrará dos ID de transacciones, una para la transacción de compromiso y otra para la transacción de revelación, y el ID de la inscripción. Los ID de inscripción tienen la forma `TXIDiN`, donde `TXID` es el ID de transacción de la transacción de revelación, y `N` es el índice de la inscripción en la transacción de revelación.

La transacción de compromiso se compromete a un tapscript que contiene el contenido de la inscripción, y la transacción de revelación gasta ese tapscript, revelando el contenido en la cadena e inscribiéndolo en el primer sat del input que contiene el tapscript correspondiente.

Espera a que la transacción de revelación sea minada. Puedes comprobar el estado de las transacciones de compromiso y revelación utilizando [el explorador de bloques mempool.space](https://mempool.space/).

Una vez que la transacción de revelación haya sido minada, el ID de la inscripción debería aparecer cuando ejecutes:

```
ord wallet inscriptions
```

Y cuando visites [el explorador de ordinals](https://ordinals.com/) en `ordinals.com/inscription/INSCRIPTION_ID`.

Enviando Inscripciones
----------------------

Pide al destinatario que genere una nueva dirección ejecutando:

```
ord wallet receive
```

Envía la inscripción ejecutando:

```
ord wallet send --fee-rate <FEE_RATE> <ADDRESS> <INSCRIPTION_ID>
```

Consulta la transacción pendiente con:

```
ord wallet transactions
```

Una vez que se confirme la transacción de envío, el destinatario puede confirmar la recepción ejecutando:

```
ord wallet inscriptions
```

Recibiendo Inscripciones
-----------------------

Genera una nueva dirección de recepción usando:

```
ord wallet receive
```

El remitente puede transferir la inscripción a tu dirección usando:

```
ord wallet send ADDRESS INSCRIPTION_ID
```

Consulta la transacción pendiente con:

```
ord wallet transactions
```

Una vez que se confirme la transacción de envío, puedes confirmar la recepción ejecutando:

```
ord wallet inscriptions
```

Esta guía te proporciona una completa descripción de cómo trabajar con inscripciones usando la herramienta `ord`. Como una nueva forma de crear artefactos digitales en la cadena de bloques de Bitcoin, las inscripciones ofrecen una interesante oportunidad para explorar nuevos usos y aplicaciones en el ecosistema de criptomonedas.
