Caza de Sats
===========

*Esta guía está desactualizada. Desde que se escribió, el binario `ord` fue modificado
para construir el índice satoshi completo solo cuando se suministra la bandera `--index-sats`.
Además, `ord` ahora tiene una billetera integrada que envuelve una billetera de Bitcoin Core.
Vea `ord wallet --help`.*

La caza de ordinales es difícil pero gratificante. La sensación de poseer una billetera llena
de UTXOs, impregnada con el aroma de sats raros y exóticos, es incomparable.

Los ordinales son números para los satoshis. Cada satoshi tiene un número ordinal y
cada número ordinal tiene un satoshi.

Preparación
-----------

Hay algunas cosas que necesitarás antes de comenzar.

1. Primero, necesitarás un nodo Bitcoin Core sincronizado con un índice de transacciones. Para
   activar la indexación de transacciones, pasa `-txindex` en la línea de comandos:

   ```sh
   bitcoind -txindex
   ```

   O coloca lo siguiente en tu [archivo de configuración de Bitcoin](https://github.com/bitcoin/bitcoin/blob/master/doc/bitcoin-conf.md#configuration-file-path):

   ```
   txindex=1
   ```

   Inícialo y espera a que se ponga al día con la punta de la cadena, momento en el cual el
   siguiente comando debería imprimir la altura actual del bloque:

   ```sh
   bitcoin-cli getblockcount
   ```

2. En segundo lugar, necesitarás un índice `ord` sincronizado.

  - Obtén una copia de `ord` desde [el repositorio](https://github.com/ordinals/ord/).

  - Ejecuta `RUST_LOG=info ord index`. Debería conectarse a tu nodo bitcoin core
    y comenzar la indexación.

  - Espera a que termine de indexar.

3. En tercer lugar, necesitarás una billetera con UTXOs que quieras buscar.

Buscando Ordinales Raros
---------------------------

### Buscando Ordinales Raros en una Billetera Bitcoin Core

El comando `ord wallet` es simplemente un envoltorio alrededor de la API RPC de Bitcoin Core, así que
buscar ordinales raros en una billetera Bitcoin Core es fácil. Asumiendo que
tu billetera se llama `foo`:

1. Carga tu billetera:

   ```sh
   bitcoin-cli loadwallet foo
   ```

2. Muestra cualquier ordinal raro en los UTXOs de la billetera `foo`:

   ```sh
   ord wallet sats
   ```

### Buscando Ordinales Raros en una Billetera no Bitcoin Core

El comando `ord wallet` es simplemente un envoltorio alrededor de la API RPC de Bitcoin Core, así que para
buscar ordinales raros en una billetera no Bitcoin Core, necesitarás importar
los descriptores de tu billetera a Bitcoin Core.

[Descriptores](https://github.com/bitcoin/bitcoin/blob/master/doc/descriptors.md)
describen las maneras en que las billeteras generan claves privadas y públicas.

Deberías importar los descriptores a Bitcoin Core solo para las claves públicas de tu billetera, no sus claves privadas.

Si se compromete el descriptor de la clave pública de tu billetera, un atacante podrá
ver las direcciones de tu billetera, pero tus fondos estarán seguros.

Si se compromete el descriptor de la clave privada de tu billetera, un atacante puede vaciar
tu billetera de fondos.

1. Obtén el descriptor de la billetera de la billetera cuyos UTXOs quieres buscar por
   ordinales raros. Se verá algo así:

   ```
   wpkh([bf1dd55e/84'/0'/0']xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)#csvefu29
   ```

2. Crea una billetera solo para observación llamada `foo-watch-only`:

   ```sh
   bitcoin-cli createwallet foo-watch-only true true
   ```

   ¡Siéntete libre de darle un mejor nombre que `foo-watch-only`!

3. Carga la billetera `foo-watch-only`:

   ```sh
   bitcoin-cli loadwallet foo-watch-only
   ```

4. Importa los descriptores de tu billetera a `foo-watch-only`:

   ```sh
   bitcoin-cli importdescriptors \
     '[{ "desc": "wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)#tpnxnxax", "timestamp":0 }]'
   ```

   Si conoces la marca de tiempo Unix cuando tu billetera comenzó a recibir
   transacciones, puedes usarla para el valor de `"timestamp"` en lugar de `0`.
   Esto reducirá el tiempo que tarda Bitcoin Core en buscar los UTXOs de tu
   billetera.

5. Comprueba que todo haya funcionado:

   ```sh
   bitcoin-cli getwalletinfo
   ```

6. Muestra los ordinales raros de tu billetera:

   ```sh
   ord wallet sats
   ```

### Buscando Ordinales Raros en una Billetera que Exporta Descriptores de Múltiples Rutas

Algunos descriptores describen múltiples rutas en un solo descriptor usando ángulos, por ejemplo, `<0;1>`. Los descriptores de múltiples rutas aún no son compatibles con Bitcoin Core, por lo que primero necesitarás convertirlos en múltiples descriptores y luego importar esos descriptores múltiples a Bitcoin Core.

1. Primero, obtén el descriptor de múltiples rutas de tu billetera. Se verá algo así:

   ```
   wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/<0;1>/*)#fw76ulgt
   ```

2. Crea un descriptor para la ruta de la dirección de recepción:

   ```
   wpkh([bf1dd55e/84'/0'/0']xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)
   ```

   Y la ruta de la dirección de cambio:

   ```
   wpkh([bf1dd55e/84'/0'/0']xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/1/*)
   ```

3. Obtén y anota la suma de comprobación para el descriptor de la dirección de recepción, en este caso
   `tpnxnxax`:

   ```sh
   bitcoin-cli getdescriptorinfo \
     'wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)'
   ```

   ```json
   {
     "descriptor": "wpkh([bf1dd55e/84'/0'/0']xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)#csvefu29",
     "checksum": "tpnxnxax",
     "isrange": true,
     "issolvable": true,
     "hasprivatekeys": false
   }
   ```

   Y para el descriptor de la dirección de cambio, en este caso `64k8wnd7`:

   ```sh
   bitcoin-cli getdescriptorinfo \
     'wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/1/*)'
   ```

   ```json
   {
     "descriptor": "wpkh([bf1dd55e/84'/0'/0']xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/1/*)#fyfc5f6a",
     "checksum": "64k8wnd7",
     "isrange": true,
     "issolvable": true,
     "hasprivatekeys": false
   }
   ```

4. Carga la billetera en la que deseas importar los descriptores:

   ```sh
   bitcoin-cli loadwallet foo-watch-only
   ```

5. Ahora importa los descriptores, con las sumas de comprobación correctas, en Bitcoin Core.

   ```sh
   bitcoin-cli \
    importdescriptors \
    '[
      {
        "desc": "wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)#tpnxnxax"
        "timestamp":0
      },
      {
        "desc": "wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/1/*)#64k8wnd7",
        "timestamp":0
      }
    ]'
   ```

   Si conoces la marca de tiempo Unix cuando tu billetera comenzó a recibir transacciones por primera vez, puedes usarla para el valor de los campos `"timestamp"` en lugar de `0`. Esto reducirá el tiempo que tarda Bitcoin Core en buscar los UTXO de tu billetera.

6. Verifica que todo haya funcionado:

   ```sh
   bitcoin-cli getwalletinfo
   ```

7. Muestra los ordinales raros de tu billetera:

   ```sh
   ord wallet sats
   ```

### Exportando Descriptores

#### Billetera Sparrow

Navega a la pestaña `Configuración`, luego a `Política de Script`, y presiona el botón de edición para mostrar el descriptor.

### Transfiriendo Ordinales

La billetera `ord` admite la transferencia de satoshis específicos. También puedes usar los comandos `bitcoin-cli` `createrawtransaction`, `signrawtransactionwithwallet`, y `sendrawtransaction`, pero hacerlo es complejo y está fuera del alcance de esta guía.
