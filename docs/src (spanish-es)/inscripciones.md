Inscripciones
============

Las inscripciones incrustan sats con contenido arbitrario, creando artefactos
digitales nativos de bitcoin, más comúnmente conocidos como NFT. Las inscripciones
no requieren una sidechain o un token separado.

Estos sats grabados luego pueden ser transferidos utilizando transacciones bitcoin,
enviados a direcciones bitcoin, y almacenados en UTXOs de bitcoin. Estas transacciones,
direcciones, y UTXOs son normales en todos los aspectos, con la excepción de que para enviar sats individuales, las
transacciones deben controlar el orden y el valor de las entradas y salidas
según la teoría de los ordinales.

El modelo de contenido de la inscripción es el de la web. Una inscripción
consiste en un tipo de contenido, también conocido como tipo MIME, y el contenido
mismo, que es una cadena de bytes. Esto permite recuperar el contenido
de la inscripción desde un servidor web, y crear inscripciones HTML que utilizan y
remezclan el contenido de otras inscripciones.

El contenido de la inscripción es completamente on-chain, almacenado en los scripts
de gasto de la ruta de scripts de taproot. Los scripts de taproot tienen muy pocas
restricciones en su contenido, y además reciben el descuento de testigo, haciendo
el almacenamiento del contenido de la inscripción relativamente económico.

Dado que los gastos de los scripts de taproot solo pueden hacerse desde salidas de
taproot existentes, las inscripciones se realizan utilizando un procedimiento de
compromiso/revelación en dos fases. Primero, en la transacción de compromiso, se crea una
salida de taproot que se compromete en un script que contiene el contenido
de la inscripción. En segundo lugar, en la transacción de revelación, la salida creada
en la transacción de compromiso se gasta, revelando el contenido de la inscripción
on-chain.

El contenido de la inscripción se serializa utilizando impulsos de datos dentro
de condicionales no ejecutados, llamados "bolsas". Las bolsas consisten en un
OP_FALSE OP_IF … OP_ENDIF que envuelve cualquier número de impulsos de datos.
Dado que las bolsas son efectivamente operaciones nulas, no cambian la semántica
del script en el que están incluidas, y pueden ser combinadas con cualquier otro
script de bloqueo.

Una inscripción de texto que contiene la cadena "Hello, world!" se serializa como
sigue:

```
OP_FALSE
OP_IF
  OP_PUSH "ord"
  OP_PUSH 1
  OP_PUSH "text/plain;charset=utf-8"
  OP_PUSH 0
  OP_PUSH "Hello, world!"
OP_ENDIF
```

Primero se empuja la cadena `ord`, para desambiguar las inscripciones de otros usos
de bolsas.

`OP_PUSH 1` indica que el siguiente empuje contiene el tipo de contenido, y `OP_PUSH 0`
indica que los empujes de datos siguientes contienen el contenido en sí. Para
inscripciones grandes deben utilizarse más empujes de datos, ya que una
de las pocas restricciones de taproot es que los empujes de datos individuales no
pueden ser mayores de 520 bytes.

El contenido de la inscripción se contiene dentro de la entrada de una transacción
de revelación, y la inscripción se realiza en el primer sat de su entrada. Este sat
puede ser rastreado utilizando las reglas familiares de la teoría de los ordinales,
permitiendo que sea transferido, comprado, vendido, perdido en comisiones, y
recuperado.

Contenido
-------

El modelo de datos de las inscripciones es el de una respuesta HTTP, permitiendo al
contenido de la inscripción ser servido desde un servidor web y visualizado en un
navegador web.

Sandboxing
----------

Las inscripciones HTML y SVG están aisladas para prevenir referencias a contenidos
off-chain, manteniendo así las inscripciones inmutables y autocontenidas.

Esto se logra cargando las inscripciones HTML y SVG dentro de `iframes` con
el atributo `sandbox`, así como sirviendo el contenido de la inscripción con
cabeceras `Content-Security-Policy`.

Recursión
---------

Una excepción importante al aislamiento es la recursión: se permite el acceso al punto final
`/content` de `ord`, permitiendo a las inscripciones acceder al contenido
de otras inscripciones solicitando `/content/<INSCRIPTION_ID>`.

Esto tiene una serie de interesantes casos de uso:

- Remezclar el contenido de inscripciones existentes.

- Publicar fragmentos de código, imágenes, audio, o hojas de estilo como recursos
  públicos compartidos.

- Colecciones de arte generativo donde un algoritmo está inscrito como JavaScript,
  e instanciado por múltiples inscripciones con semillas únicas.

- Colecciones de imágenes de perfil generativas donde accesorios y atributos son
  inscritos como imágenes individuales, o en un atlas de texturas compartido, y luego
  combinados, al estilo collage, en combinaciones únicas en múltiples inscripciones.

Otros puntos finales a los que las inscripciones pueden acceder son los siguientes:

- `/blockheight`: altura del bloque más reciente.
- `/blockhash`: hash del bloque más reciente.
- `/blockhash/<HEIGHT>`: hash del bloque en la altura de bloque dada.
- `/blocktime`: timestamp UNIX del bloque más reciente.
