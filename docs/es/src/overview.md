Descripción General de la Teoría Ordinal
==============================

Los números ordinales son un esquema de numeración para los satoshis que permite el seguimiento y la transferencia de sats individuales. Estos números se llaman [números ordinales](https://ordinals.com). Los satoshis se numeran en el orden en que se extraen y se transfieren desde las entradas de transacciones a las salidas de transacciones en el orden en que fueron extraídos. Tanto el esquema de numeración como el esquema de transferencia se basan en el *orden*, el esquema de numeración en el *orden* en que se extraen los satoshis y el esquema de transferencia en el *orden* de las entradas y salidas de transacciones. De ahí el nombre, *ordinales*.

Los detalles técnicos están disponibles en [el BIP](https://github.com/ordinals/ord/blob/master/bip.mediawiki).

La teoría ordinal no requiere un token separado, otra cadena de bloques o ningún cambio en Bitcoin. Funciona en este momento.

Los números ordinales tienen varias representaciones diferentes:

- *Notación entera*:
  [`2099994106992659`](https://ordinals.com/sat/2099994106992659) El número
  ordinal, asignado según el orden en que se extrajo el satoshi.

- *Notación decimal*:
  [`3891094.16797`](https://ordinals.com/sat/3891094.16797) El primer número
  es la altura del bloque en la que se extrajo el satoshi, el segundo es el desplazamiento del satoshi dentro del bloque.

- *Notación de grados*:
  [`3°111094′214″16797‴`](https://ordinals.com/sat/3%C2%B0111094%E2%80%B2214%E2%80%B316797%E2%80%B4).
  Llegaremos a eso en un momento.

- *Notación percentil*:
  [`99.99971949060254%`](https://ordinals.com/sat/99.99971949060254%25) .
  La posición del satoshi en el suministro de Bitcoin, expresada como un porcentaje.

- *Nombre*: [`satoshi`](https://ordinals.com/sat/satoshi). Una codificación del número ordinal utilizando los caracteres de `a` a `z`.

Activos arbitrarios, como NFTs, tokens de seguridad, cuentas o stablecoins, pueden ser vinculados a los satoshis utilizando los números ordinales como identificadores estables.

Ordinals es un proyecto de código abierto, desarrollado [en GitHub](https://github.com/ordinals/ord). El proyecto consta de un BIP que describe el esquema ordinal, un índice que se comunica con un nodo de Bitcoin Core para rastrear la ubicación de todos los satoshis, una billetera que permite realizar transacciones con conciencia ordinal, un explorador de bloques para la exploración interactiva de la cadena de bloques, funcionalidad para inscribir satoshis con artefactos digitales y este manual.

Rarezas
------

Los humanos son coleccionistas, y dado que ahora los satoshis se pueden rastrear y transferir, naturalmente las personas querrán coleccionarlos. Los teóricos ordinales pueden decidir por sí mismos qué sats son raros y deseables, pero aquí hay algunas pistas...

Bitcoin tiene eventos periódicos, algunos frecuentes y otros menos comunes, y naturalmente estos se prestan para un sistema de rareza. Estos eventos periódicos son:

- *Bloques*: Se extrae aproximadamente un nuevo bloque cada 10 minutos, desde ahora hasta el fin de los tiempos.

- *Ajustes de dificultad*: Cada 2016 bloques, o aproximadamente cada dos semanas, la red de Bitcoin responde a los cambios en la tasa de hash ajustando el objetivo de dificultad que los bloques deben cumplir para ser aceptados.

- *Halvings*: Cada 210,000 bloques, aproximadamente cada cuatro años, la cantidad de nuevos sats creados en cada bloque se reduce a la mitad.

- *Ciclos*: Cada seis halvings, algo mágico sucede: el halving y el ajuste de dificultad coinciden. Esto se llama conjunción, y el período de tiempo entre las conjunciones es un ciclo. Una conjunción ocurre aproximadamente cada 24 años. La primera conjunción debería ocurrir en algún momento en 2032.

Esto nos da los siguientes niveles de rareza:

- `común`: Cualquier sat que no sea el primer sat de su bloque
- `poco común`: El primer sat de cada bloque
- `raro`: El primer sat de cada período de ajuste de dificultad
- `épico`: El primer sat de cada época de halving
- `legendario`: El primer sat de cada ciclo
- `mítico`: El primer sat del bloque génesis

Lo cual nos lleva a la notación de grado, que representa de manera inequívoca un número ordinal de una manera que hace que la rareza de un satoshi sea fácil de ver de un vistazo:

```
A°B′C″D‴
│ │ │ ╰─ Índice del sat en el bloque
│ │ ╰─── Índice del bloque en el período de ajuste de dificultad
│ ╰───── Índice del bloque en la época de halving
╰─────── Ciclo, numerado a partir de 0
```

Los teóricos ordinales a menudo usan los términos "hora", "minuto", "segundo" y "tercero" para *A*, *B*, *C* y *D*, respectivamente.

Ahora algunos ejemplos. Este satoshi es común:

```
1°1′1″1‴
│ │ │ ╰─ No es el primer sat en el bloque
│ │ ╰─── No es el primer bloque en el período de ajuste de dificultad
│ ╰───── No es el primer bloque en la época de halving
╰─────── Segundo ciclo
```


Este satoshi es poco común:

```
1°1′1″0‴
│ │ │ ╰─ Primer sat en el bloque
│ │ ╰─── No es el primer bloque en el período de ajuste de dificultad
│ ╰───── No es el primer bloque en la época de halving
╰─────── Segundo ciclo
```

Este satoshi es raro:

```
1°1′0″0‴
│ │ │ ╰─ Primer sat en el bloque
│ │ ╰─── Primer bloque en el período de ajuste de dificultad
│ ╰───── No es el primer bloque en la época de halving
╰─────── Segundo ciclo
```

Este satoshi es épico:

```
1°0′1″0‴
│ │ │ ╰─ Primer sat en el bloque
│ │ ╰─── No es el primer bloque en el período de ajuste de dificultad
│ ╰───── Primer bloque en la época de halving
╰─────── Segundo ciclo
```

Este satoshi es legendario:

```
1°0′0″0‴
│ │ │ ╰─ Primer sat en el bloque
│ │ ╰─── Primer bloque en el período de ajuste de dificultad
│ ╰───── Primer bloque en la época de halving
╰─────── Segundo ciclo
```

Y este satoshi es mítico:

```
0°0′0″0‴
│ │ │ ╰─ Primer sat en el bloque
│ │ ╰─── Primer bloque en el período de ajuste de dificultad
│ ╰───── Primer bloque en la época de halving
╰─────── Primer ciclo
```

Si el desplazamiento del bloque es cero, puede omitirse. Este es el satoshi poco común de arriba:

```
1°1′1″
│ │ ╰─ No es el primer bloque en el período de ajuste de dificultad
│ ╰─── No es el primer bloque en la época de halving
╰───── Segundo ciclo
```

Suministro de Satoshi Raros
------------------------

### Suministro Total

- `común`: 2.1 billones de billones
- `poco común`: 6,929,999
- `raro`: 3437
- `épico`: 32
- `legendario`: 5
- `mítico`: 1

### Suministro Actual

- `común`: 1.9 billones de billones
- `poco común`: 745,855
- `raro`: 369
- `épico`: 3
- `legendario`: 0
- `mítico`: 1

En este momento, incluso los satoshis poco comunes son bastante raros. Hasta la fecha de esta escritura, se han extraído 745,855 satoshis poco comunes, uno por cada 25.6 bitcoins en circulación.

Nombres
-----

Cada satoshi tiene un nombre, que consiste en las letras *A* a *Z*, y que se hace más corto a medida que el satoshi se extrae más en el futuro. Podrían comenzar siendo cortos y hacerse más largos, pero entonces todos los buenos y cortos nombres estarían atrapados en el bloque génesis no gastable.

Como ejemplo, el nombre de 1905530482684727° es "iaiufjszmoba". El nombre del último satoshi que se extraerá es "a". Cada combinación de 10 caracteres o menos está ahí afuera, o estará ahí afuera, algún día.

Exóticos
-------

Los satoshis pueden ser valorados por razones distintas a su nombre o rareza. Esto podría deberse a una cualidad del número en sí mismo, como tener una raíz cuadrada o cúbica entera. O podría deberse a una conexión con un evento histórico, como los satoshis del bloque 477,120, el bloque en el que se activó SegWit, o 2099999997689999°, el último satoshi que se extraerá.

Tales satoshis se llaman "exóticos". Qué satoshis son exóticos y qué los hace exóticos es subjetivo. Se alienta a los teóricos ordinales a buscar exóticos en función de criterios que ellos mismos elaboren.

Inscripciones
------------

Los satoshis pueden ser inscritos con contenido arbitrario, creando artefactos digitales nativos de Bitcoin. La inscripción se realiza enviando el satoshi que se inscribirá en una transacción que revela el contenido de la inscripción en la cadena. Este contenido está inextricablemente vinculado a ese satoshi, convirtiéndolo en un artefacto digital inmutable que se puede rastrear, transferir, acumular, comprar, vender, perder y redescubrir.

Arqueología
-----------

Ha surgido una comunidad activa de arqueólogos dedicados a catalogar y recolectar NFT tempranos. [Aquí hay un gran resumen de NFT históricos por Chainleft.](https://mirror.xyz/chainleft.eth/MzPWRsesC9mQflxlLo-N29oF4iwCgX3lacrvaG9Kjko)

Una fecha comúnmente aceptada para los NFT tempranos es el 19 de marzo de 2018, la fecha en que se implementó el primer contrato ERC-721, [SU SQUARES](https://tenthousandsu.com/), en Ethereum.

¡Si los ordinales son o no de interés para los arqueólogos de NFT es una pregunta abierta! En un sentido, los ordinales se crearon a principios de 2022, cuando se finalizó la especificación de los Ordinales. En este sentido, no son de interés histórico.

En otro sentido, sin embargo, los ordinales fueron creados por Satoshi Nakamoto en 2009 cuando extrajo el bloque génesis de Bitcoin. En este sentido, los ordinales, y especialmente los ordinales tempranos, ciertamente son de interés histórico.

Muchos teóricos ordinales favorecen la última visión. Esto se debe en parte a que los ordinales fueron descubiertos de manera independiente en al menos dos ocasiones separadas, mucho antes de que comenzara la era de los NFT modernos.

El 21 de agosto de 2012, Charlie Lee [publicó una propuesta para agregar la prueba de participación a Bitcoin en el foro de Bitcoin Talk](https://bitcointalk.org/index.php?topic=102355.0). Esto no era un esquema de activos, pero utilizaba el algoritmo ordinal y se implementó pero nunca se desplegó.

El 8 de octubre de 2012, jl2012 [publicó un esquema en el mismo foro](https://bitcointalk.org/index.php?topic=117224.0) que utiliza la notación decimal y tiene todas las propiedades importantes de los ordinales. El esquema se discutió pero nunca se implementó.

Estas invenciones independientes de los ordinales indican de alguna manera que los ordinales fueron descubiertos, o redescubiertos, y no inventados. Los ordinales son una inevitabilidad de las matemáticas de Bitcoin, que se derivan no de su documentación moderna, sino de su antiguo génesis. Son la culminación de una secuencia de eventos iniciados con la extracción del primer bloque, hace tantos años.
