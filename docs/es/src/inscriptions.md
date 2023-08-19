Inscripciones
============

Las inscripciones inscriben sats con contenido arbitrario, creando artefactos digitales nativos de Bitcoin, más conocidos como NFTs (Tokens No Fungibles). Las inscripciones no requieren una sidechain o un token separado.

Estos sats inscritos pueden luego ser transferidos mediante transacciones de Bitcoin, enviados a direcciones de Bitcoin y mantenidos en salidas no gastadas (UTXOs) de Bitcoin. Estas transacciones, direcciones y UTXOs son transacciones normales, direcciones y UTXOs de Bitcoin en todos los aspectos, con la excepción de que para enviar sats individuales, las transacciones deben controlar el orden y el valor de las entradas y salidas según la teoría ordinal.

El modelo de contenido de las inscripciones es similar al de la web. Una inscripción consta de un tipo de contenido, también conocido como tipo MIME, y el propio contenido, que es una cadena de bytes. Esto permite que el contenido de la inscripción se recupere desde un servidor web y para crear inscripciones HTML que utilicen y mezclen el contenido de otras inscripciones.

El contenido de la inscripción está completamente en la cadena de bloques, almacenado en scripts de gasto de ruta de taproot. Los scripts de taproot tienen muy pocas restricciones en su contenido y, además, reciben un descuento en el testigo, lo que hace que el almacenamiento de contenido de inscripción sea relativamente económico.

Dado que las salidas de scripts de taproot solo se pueden crear a partir de salidas de taproot existentes, las inscripciones se realizan mediante un procedimiento de compromiso/revelación de dos fases. Primero, en la transacción de compromiso, se crea una salida de taproot que se compromete a un script que contiene el contenido de la inscripción. En segundo lugar, en la transacción de revelación, se gasta la salida creada por la transacción de compromiso, revelando el contenido de la inscripción en la cadena de bloques.

El contenido de la inscripción se serializa utilizando empujes de datos dentro de condicionales no ejecutados, llamados "sobres". Los sobres consisten en un `OP_FALSE OP_IF … OP_ENDIF` que envuelve cualquier cantidad de empujes de datos. Dado que los sobres son efectivamente no operaciones, no cambian la semántica del script en el que se incluyen y se pueden combinar con cualquier otro script de bloqueo.

Una inscripción de texto que contiene la cadena "¡Hola, mundo!" se serializa de la siguiente manera:

```
OP_FALSE
OP_IF
  OP_PUSH "ord"
  OP_PUSH 1
  OP_PUSH "text/plain;charset=utf-8"
  OP_PUSH 0
  OP_PUSH "¡Hola, mundo!"
OP_ENDIF
```

Primero se empuja la cadena `ord` para desambiguar las inscripciones de otros usos de sobres.

`OP_PUSH 1` indica que el siguiente empuje contiene el tipo de contenido y `OP_PUSH 0` indica que los empujes de datos subsiguientes contienen el contenido mismo. Deben utilizarse múltiples empujes de datos para inscripciones grandes, ya que una de las pocas restricciones de taproot es que los empujes de datos individuales no pueden tener más de 520 bytes.

El contenido de la inscripción se encuentra dentro de la entrada de una transacción de revelación, y la inscripción se realiza en el primer sat de su entrada. Este sat puede luego ser rastreado utilizando las reglas familiares de la teoría ordinal, lo que permite que se transfiera, compre, venda, se pierda en tarifas y se recupere.

Contenido
--------

El modelo de datos de las inscripciones es similar al de una respuesta HTTP, lo que permite que el contenido de la inscripción se sirva desde un servidor web y se vea en un navegador web.

Aislamiento
----------

Las inscripciones HTML y SVG están aisladas para evitar referencias a contenido fuera de la cadena, manteniendo así las inscripciones inmutables y autocontenidas.

Esto se logra cargando las inscripciones HTML y SVG dentro de `iframes` con el atributo `sandbox`, así como sirviendo el contenido de la inscripción con encabezados `Content-Security-Policy`.

Recursión
---------

Una excepción importante al aislamiento es la recursión: se permite el acceso al punto final `/content` de `ord`, lo que permite que las inscripciones accedan al contenido de otras inscripciones solicitando `/content/<ID_INSCRIPCION>`.

Esto tiene varios casos de uso interesantes:

- Mezclar el contenido de inscripciones existentes.

- Publicar fragmentos de código, imágenes, audio o hojas de estilo como recursos públicos compartidos.

- Colecciones de arte generativo donde un algoritmo está inscrito como JavaScript y se instancia desde múltiples inscripciones con semillas únicas.

- Colecciones generativas de imágenes de perfil donde los accesorios y atributos están inscritos como imágenes individuales o en un atlas de texturas compartido, y luego se combinan, estilo collage, en combinaciones únicas en múltiples inscripciones.

Otros puntos finales a los que las inscripciones pueden acceder son los siguientes:

- `/blockheight`: altura del último bloque.
- `/blockhash`: hash del último bloque.
- `/blockhash/<ALTURA>`: hash del bloque en la altura de bloque dada.
- `/blocktime`: marca de tiempo UNIX del último bloque.
