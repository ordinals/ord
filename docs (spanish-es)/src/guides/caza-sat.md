Caza de Sat
===========

*Esta guía no está actualizada. Desde que fue escrita, el binario `ord` ha sido modificado
para construir el índice completo de satoshis solo cuando se proporciona el flag `--index-sats`.
Además, ord ahora tiene una billetera integrada que envuelve una billetera Bitcoin Core.
Consulta `ord wallet --help`.*

La caza de ordinales es desafiante pero gratificante. La sensación de tener una billetera llena
de UTXOs, impregnada del aroma de sats raros y exóticos, es inigualable.

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

   O inserta lo siguiente en tu [archivo de configuración de Bitcoin]
   (https://github.com/bitcoin/bitcoin/blob/master/doc/bitcoin-conf.md#configuration-file-path):

   ```
   txindex=1
   ```

   Inícialo y espera a que se sincronice con la punta de la cadena; en ese momento, el
   siguiente comando debería mostrar la altura del bloque actual:

   ```sh
   bitcoin-cli getblockcount
   ```

2. En segundo lugar, necesitarás un índice `ord` sincronizado.

   - Obtiene una copia de `ord` del [repositorio].(https://github.com/ordinals/ord/).

   - Ejecuta `RUST_LOG=info ord index`. Debería conectarse a tu nodo bitcoin core
   y comenzar la indexación.

   - Espera a que termine la indexación.

3. En tercer lugar, necesitarás una billetera con UTXOs que quieras buscar.

Búsqueda de Ordinales Raros
---------------------------

### Búsqueda de ordinales raros en una billetera Bitcoin Core

El comando `ord wallet` es solo un envoltorio alrededor de la API RPC de Bitcoin Core, por lo que
buscar ordinales raros en una billetera Bitcoin Core es fácil. Suponiendo que tu
billetera se llame `foo`:

1. Carga tu billetera:

   ```sh
   bitcoin-cli loadwallet foo
   ```

2. Muestra cualquier ordinal raro de los UTXOs de la billetera `foo`:

   ```sh
   ord wallet sats
   ```

### Búsqueda de ordinales raros en una billetera no Bitcoin Core

El comando `ord wallet` es solo un envoltorio alrededor de la API RPC de Bitcoin Core, así que para
buscar ordinales raros en una billetera no Bitcoin Core, tendrás que importar
los descriptores de tu billetera en Bitcoin Core.

[Los descriptores](https://github.com/bitcoin/bitcoin/blob/master/doc/descriptors.md)
describen las formas en que las billeteras generan claves privadas y claves públicas.

Debes importar en Bitcoin Core solo los descriptores para las claves públicas de tu billetera,
no sus claves privadas.

Si el descriptor de la clave pública de tu billetera se ve comprometido, un atacante podrá
ver las direcciones de tu billetera, pero tus fondos estarán seguros.

Si el descriptor de la clave privada de tu billetera se ve comprometido, un atacante puede vaciar
los fondos de tu billetera.

1. Obtén el descriptor de la billetera de la billetera cuyos UTXOs quieres buscar por
   ordinales raros. Se parecerá a algo así:

   ```
   wpkh([bf1dd55e/84'/0'/0']xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)#csvefu29
   ```

2. Crea una billetera de solo lectura llamada: `foo-watch-only`:

   ```sh
   bitcoin-cli createwallet foo-watch-only true true
   ```

   ¡Siéntete libre de darle un nombre mejor que `foo-watch-only`!

3. Carga la cartera `foo-watch-only`:

   ```sh
   bitcoin-cli loadwallet foo-watch-only
   ```

4. Importa los descriptores de tu cartera a `foo-watch-only`:

   ```sh
   bitcoin-cli importdescriptors \
     '[{ "desc": "wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)#tpnxnxax", "timestamp":0 }]'
   ```

   Si conoces el timestamp Unix cuando tu cartera comenzó a recibir transacciones, 
   puedes usarlo para el valor de `"timestamp"` en lugar de `0`. 
   Esto reducirá el tiempo necesario para que 
   Bitcoin Core busque tus UTXOs en la cartera.

5. Verifica que todo haya funcionado:

   ```sh
   bitcoin-cli getwalletinfo
   ```

6. Muestra los ordinales raros de tu cartera:

   ```sh
   ord wallet sats
   ```

### Búsqueda de ordinales raros en una cartera que exporta descriptores multi-path

Algunos descriptores describen múltiples rutas en un único descriptor 
usando los signos de menor y mayor, por ejemplo, `<0;1>`. 
Los descriptores multi-path aún no son soportados por Bitcoin Core, 
por lo que primero deberás convertirlos en múltiples descriptores y 
luego importar estos descriptores en Bitcoin Core.

1. Primero, obtén el descriptor multi-path de tu cartera. Se parecerá a algo como esto:

   ```
   wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/<0;1>/*)#fw76ulgt
   ```

2. Crea un descriptor para la dirección de recepción:

   ```
   wpkh([bf1dd55e/84'/0'/0']xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)
   ```

   Y para la dirección de cambio:


   ```
   wpkh([bf1dd55e/84'/0'/0']xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/1/*)
   ```

3. Obtén y anota el checksum para el descriptor de la dirección de recepción, en este caso  
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

4. Carga la cartera en la que deseas importar los descriptores:

   ```sh
   bitcoin-cli loadwallet foo-watch-only
   ```

5. Ahora importa los descriptores, con los checksum correctos, en Bitcoin Core.

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

   Si conoces el timestamp Unix del momento en que tu monedero comenzó a recibir transacciones, 
   podrías usarlo para el valor de los campos `"timestamp"` en lugar de `0`. 
   Esto reducirá el tiempo que Bitcoin Core tarda en buscar tus UTXO en el monedero.

6. Verifica que todo haya funcionado correctamente:

   ```sh
   bitcoin-cli getwalletinfo
   ```

7. Muestra los ordinales raros de tu monedero:

   ```sh
   ord wallet sats
   ```

### Exportar los Descriptores

#### Sparrow Wallet

Ve a `Configuración`, luego a `Política de Script`, y presiona el botón de edición
para ver el descriptor.

### Transferir los Ordinales

El monedero `ord` admite la transferencia de satoshis específicos. También puedes usar
los comandos `bitcoin-cli`, `createrawtransaction`, `signrawtransactionwithwallet`,
y `sendrawtransaction`, pero cómo hacerlo es complejo y queda fuera del alcance de esta
guía.
