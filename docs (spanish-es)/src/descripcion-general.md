Resumen de la Teoría Ordinal
=======================

Los ordinales son un esquema de numeración para los satoshis que permite rastrear y
transferir satoshis individuales. Estos números son llamados [numeri ordinali](https://ordinals.com). 
Los satoshis son numerados en el orden en que son minados, y transferidos de las entradas a las salidas de las transacciones
en orden de llegada. Tanto el esquema de numeración como el de transferencia se basan
en el orden, el esquema de numeración en el orden en que los satoshis son minados, y
el esquema de transferencia en el orden de las entradas y salidas de las transacciones. De ahí el
nombre, *ordinales*.

Los detalles técnicos están disponibles en [
BIP](https://github.com/ordinals/ord/blob/master/bip.mediawiki).

La teoría ordinal no requiere un token separado, otra blockchain, o cualquier
modificación a Bitcoin. Funciona ya ahora.

Los números ordinales tienen algunas representaciones diferentes:

- *Notación entera:*:
  [`2099994106992659`](https://ordinals.com/sat/2099994106992659) El número ordinal,
asignado basado en el orden en que el satoshi fue minado.

- *Notación decimal:*:
  [`3891094.16797`](https://ordinals.com/sat/3891094.16797) El primer
número es la altura del bloque en que el satoshi fue minado, el segundo es
el desplazamiento del satoshi dentro del bloque.

- *Notación de grado:*:
  [`3°111094′214″16797‴`](https://ordinals.com/sat/3%C2%B0111094%E2%80%B2214%E2%80%B316797%E2%80%B4).
  Llegaremos a esto en un momento.

- *Notación porcentual:*:
  [`99.99971949060254%`](https://ordinals.com/sat/99.99971949060254%25) .
  La posición del satoshi en el suministro de Bitcoin, expresada como porcentaje.

- *Nombre*: [`satoshi`](https://ordinals.com/sat/satoshi). Una codificación del
número ordinal utilizando los caracteres de `a` a `z`.

Bienes arbitrarios, como NFTs, tokens de seguridad, cuentas o stablecoins pueden
ser adjuntados a los satoshis utilizando números ordinales como identificadores estables.

Ordinals es un proyecto de código abierto, desarrollado [en
GitHub](https://github.com/ordinals/ord).  El proyecto consta de un BIP que describe
el esquema ordinal, un índice que se comunica con un nodo Bitcoin Core para
rastrear la posición de todos los satoshis, un monedero que permite realizar transacciones conscientes del orden,
un explorador de bloques para la exploración interactiva de la blockchain,
funcionalidades para inscribir los satoshis con artefactos digitales, y este manual.

Rareza
------

Los seres humanos son coleccionistas, y dado que ahora los satoshis pueden ser rastreados y transferidos,
las personas naturalmente querrán coleccionarlos. Los teóricos ordinales pueden decidir por
sí mismos cuáles satoshis son raros y deseables, pero hay algunas pistas…

Bitcoin tiene eventos periódicos, algunos frecuentes, otros más raros, y estos
se prestan naturalmente a un sistema de rareza. Estos eventos periódicos son:

- *Bloques*: Un nuevo bloque se mina aproximadamente cada 10 minutos, desde ahora hasta el fin de los tiempos.

- *Ajustes de dificultad*: Cada 2016 bloques, o aproximadamente cada dos
semanas, la red Bitcoin responde a los cambios en el hashrate ajustando el
objetivo de dificultad que los bloques deben alcanzar para ser aceptados.

- *Halving*: Cada 210,000 bloques, o aproximadamente cada cuatro años, la cantidad de
nuevos satoshis creados en cada bloque se reduce a la mitad.

- *Ciclos*: Cada seis halvings, sucede algo mágico: el halving y el
ajuste de dificultad coinciden. Esto se llama una conjunción, y el período de tiempo
entre las conjunciones es un ciclo. Una conjunción ocurre aproximadamente cada 24
años. La primera conjunción debería ocurrir alrededor de 2032.

Esto nos da los siguientes niveles de rareza:


- `común`: Cualquier satoshi que no sea el primer sat de su bloque
- `poco común`: El primer sat de cada bloque
- `raro`: El primer sat de cada periodo de ajuste de dificultad
- `épico`: El primer sat de cada época de halving
- `legendario`: El primer sat de cada ciclo
- `mítico`: El primer sat del bloque de génesis

Esto nos lleva a la notación de grados, que representa de manera única un número ordinal
de manera que la rareza de un satoshi sea fácil de ver a simple vista:

```
A°B′C″D‴
│ │ │ ╰─ Índice de sat en el bloque
│ │ ╰─── Índice del bloque en el período de ajuste de dificultad
│ ╰───── Índice del bloque en la época de halving
╰─────── Ciclo, numerado desde 0
```

Los teóricos ordinales a menudo usan los términos "hora", "minuto", "segundo", y "tercero"
para A, B, C, y D, respectivamente.

Ahora para dar algunos ejemplos. Este satoshi es común:

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

Si el desplazamiento del bloque es cero, puede omitirse. Este es el satoshi poco común
de arriba:


```
1°1′1″
│ │ ╰─ No es el primer bloque en el período de ajuste de dificultad
│ ╰─── No es el primer bloque en la época de halving
╰───── Segundo ciclo
```

Suministro de Satoshis Raros
-------------------

### Suministro Total

- `común`: 2.1 cuatrillones
- `poco común`: 6,929,999
- `raro`: 3437
- `épico`: 32
- `legendario`: 5
- `mítico`: 1

### Suministro Actual

- `común`: 1.9 cuatrillones
- `poco común`: 745,855
- `raro`: 369
- `épico`: 3
- `legendario`: 0
- `mítico`: 1

En este momento, incluso los satoshis poco comunes son bastante raros. Al momento de escribir,
se han minado 745,855 satoshis poco comunes - uno por cada 25.6 bitcoins en circulación.

Nombres
-----

Cada satoshi tiene un nombre, compuesto por las letras A a Z,
que se acorta cuanto más se mina el satoshi en el futuro.
Pueden comenzar cortos y alargarse, pero entonces todos los buenos nombres cortos
quedarían atrapados en el bloque de génesis no gastable.

Por ejemplo, el nombre de 1905530482684727° es "iaiufjszmoba".
El nombre del último satoshi en ser minado es "a".
Cada combinación de 10 caracteres o menos está allí fuera, o estará, algún día.

Exóticos
-------

Los satoshis pueden ser apreciados por razones distintas a su nombre o rareza. Podría ser
por una cualidad del número en sí, como tener una raíz cuadrada o cúbica entera.
O podría deberse a una conexión con un evento histórico, como los satoshis
del bloque 477,120, el bloque en el que se activó SegWit, o 2099999997689999°,
el último satoshi que será minado alguna vez.

A tales satoshis se les llama "exóticos". Cuáles satoshis son exóticos y qué
los hace tales es subjetivo. Se alienta a los teóricos ordinales a buscar
exóticos basados en criterios de su propia concepción.

Inscripciones
------------

Los satoshis pueden ser grabados con contenidos arbitrarios, creando artefactos digitales
nativos de Bitcoin. La inscripción se realiza enviando el satoshi a grabar en
una transacción que revela el contenido de la inscripción en cadena.
Este contenido queda entonces inextricablemente ligado a ese satoshi,
convirtiéndolo en un artefacto digital inmutable que puede ser rastreado,
transferido, acaparado, comprado, vendido, perdido y redescubierto.


Arqueología
-----------

Ha surgido una vibrante comunidad de arqueólogos dedicados a catalogar y recoger los primeros NFT.
[Aquí hay un excelente resumen de los NFT históricos de Chainleft.](https://mirror.xyz/chainleft.eth/MzPWRsesC9mQflxlLo-N29oF4iwCgX3lacrvaG9Kjko)

Una fecha comúnmente aceptada como límite para los primeros NFT es el 19 de marzo de 2018,
fecha en la que el primer contrato ERC-721, [SU SQUARES](https://tenthousandsu.com/), fue implementado
en Ethereum.

Si los ordinales interesan o no a los arqueólogos de los NFT es una cuestión abierta!
En cierto sentido, los ordinales fueron creados a principios de 2022,
cuando se finalizó la especificación de los ordinales. En este sentido, no son de interés histórico.

Pero en otro sentido, los ordinales fueron en efecto creados por Satoshi Nakamoto
en 2009 cuando minó el bloque de génesis de Bitcoin. En este sentido, los ordinales,
y en particular los primeros ordinales, son ciertamente de interés histórico.

Muchos teóricos de los ordinales prefieren esta última visión.
No es el último motivo por el cual los ordinales fueron descubiertos independientemente
en al menos dos ocasiones separadas, mucho antes de que comenzara la era de los NFT modernos.

El 21 de Agosto de 2012, Charlie Lee [publicó una propuesta para añadir la Prueba de Participación
a Bitcoin en el foro de Bitcoin Talk.](https://bitcointalk.org/index.php?topic=102355.0). 
Esto no era un esquema de activos, pero utilizaba el algoritmo ordinal,
y fue implementado pero nunca distribuido.

El 8 de Octubre de 2012, jl2012 [publicó un esquema en el mismo foro]
(https://bitcointalk.org/index.php?topic=117224.0) que utiliza la notación
decimal y tiene todas las propiedades importantes de los ordinales.
El esquema se discutió pero nunca se implementó.

Estas invenciones independientes de los ordinales indican de alguna manera
que los ordinales fueron descubiertos, o redescubiertos, y no inventados.
Los ordinales son una inevitabilidad de la matemática de Bitcoin,
que deriva no de su moderna documentación, sino de su antigua génesis.
Son el culmen de una secuencia de eventos puesta en marcha
con la minería del primer bloque, hace tantos años.





