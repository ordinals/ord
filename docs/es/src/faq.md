Preguntas frecuentes sobre la teoría ordinal
===========================================

¿Qué es la teoría ordinal?
---------------------------

La teoría ordinal es un protocolo para asignar números de serie a los satoshis, la
mínima subdivisión de un bitcoin, y rastrear esos satoshis mientras son
gastados en transacciones.

Estos números de serie son números grandes, como este 804766073970493. Cada
satoshi, que es ¹⁄₁₀₀₀₀₀₀₀₀ de un bitcoin, tiene un número ordinal.

¿La teoría ordinal requiere una cadena lateral, un token separado o cambios en Bitcoin?
----------------------------------------------------------------------------------------

¡No! La teoría ordinal funciona ahora mismo, sin una cadena lateral, y el único token
necesario es el propio bitcoin.

¿Para qué sirve la teoría ordinal?
-----------------------------------

Recolección, comercio y planificación. La teoría ordinal asigna identidades a
los satoshis individuales, permitiendo que sean rastreados y comercializados individualmente, como
curiosidades y por su valor numismático.

La teoría ordinal también permite inscripciones, un protocolo para adjuntar contenido arbitrario
a los satoshis individuales, convirtiéndolos en artefactos digitales nativos de bitcoin.

¿Cómo funciona la teoría ordinal?
----------------------------------

Los números ordinales se asignan a los satoshis en el orden en que se extraen.
El primer satoshi en el primer bloque tiene el número ordinal 0, el segundo tiene
número ordinal 1, y el último satoshi del primer bloque tiene el número ordinal
4,999,999,999.

Los satoshis viven en las salidas, pero las transacciones destruyen las salidas y crean nuevas,
por lo que la teoría ordinal utiliza un algoritmo para determinar cómo los satoshis saltan de las
entradas de una transacción a sus salidas.

Afortunadamente, ese algoritmo es muy simple.

Los satoshis se transfieren en orden de primero en entrar, primero en salir. Piensa en las entradas de una
transacción como una lista de satoshis, y las salidas como una lista de espacios,
esperando recibir un satoshi. Para asignar satoshis de entrada a los espacios, pasa por
cada satoshi en las entradas en orden, y asigna cada uno al primer espacio disponible
en las salidas.

Imaginemos una transacción con tres entradas y dos salidas. Las entradas están
a la izquierda de la flecha y las salidas están a la derecha, todas etiquetadas con
sus valores:

    [2] [1] [3] → [4] [2]

Ahora etiquetemos la misma transacción con los números ordinales de los satoshis
que contiene cada entrada, y signos de interrogación para cada espacio de salida. Los números ordinales
son grandes, así que usemos letras para representarlos:

    [a b] [c] [d e f] → [? ? ? ?] [? ?]

Para averiguar qué satoshi va a qué salida, pasa por los satoshis de entrada en orden y asigna cada uno a un signo de interrogación:

    [a b] [c] [d e f] → [a b c d] [e f]

¿Y las comisiones, podrías preguntar? ¡Buena pregunta! Imaginemos la misma
transacción, esta vez con una comisión de dos satoshis. Las transacciones con comisiones envían más
satoshis en las entradas de los que reciben en las salidas, así que para hacer nuestra
transacción en una que pague comisiones, eliminaremos la segunda salida:

    [2] [1] [3] → [4]

Los satoshis <var>e</var> y <var>f</var> ahora no tienen a dónde ir en las
salidas:

    [a b] [c] [d e f] → [a b c d]

Así que van al minero que minó el bloque como comisiones. [El
BIP](https://github.com/ordinals/ord/blob/master/bip.mediawiki) tiene los detalles,
pero en resumen, las comisiones pagadas por las transacciones se tratan como entradas adicionales a la
transacción coinbase, y se ordenan como sus transacciones correspondientes están
ordenadas en el bloque. La transacción coinbase del bloque podría verse así:

    [SUBSIDY] [e f] → [SUBSIDY e f]

¿Dónde puedo encontrar los detalles técnicos?
---------------------------------------------

[¡El BIP!](https://github.com/ordinals/ord/blob/master/bip.mediawiki)

¿Por qué las inscripciones de satoshis se llaman "artefactos digitales" en lugar de "NFT"?
--------------------------------------------------------------------------------------------

Una inscripción es un NFT, pero se utiliza el término "artefacto digital" en su lugar, porque es sencillo, sugerente y familiar.

La frase "artefacto digital" es muy sugerente, incluso para alguien que nunca ha escuchado el término antes. En comparación, NFT es un acrónimo y no proporciona ninguna indicación de lo que significa si no has escuchado el término antes.

Además, "NFT" suena como terminología financiera, y tanto la palabra "fungible" como el sentido de la palabra "token" que se usa en "NFT" son poco comunes fuera de los contextos financieros.

¿Cómo se comparan las inscripciones de satoshis con...
------------------------------------------------------

### ¿NFTs de Ethereum?

*Las inscripciones siempre son inmutables.*

Simplemente no hay forma de que el creador de una inscripción o el propietario de una inscripción la modifiquen después de haber sido creada.

Los NFTs de Ethereum *pueden* ser inmutables, pero muchos no lo son, y pueden ser cambiados o eliminados por el propietario del contrato NFT.

Para asegurarse de que un NFT de Ethereum en particular sea inmutable, el código del contrato debe ser auditado, lo que requiere un conocimiento detallado de la EVM y la semántica de Solidity.

Es muy difícil para un usuario no técnico determinar si un NFT de Ethereum dado es mutable o inmutable, y las plataformas de NFT de Ethereum no hacen ningún esfuerzo por distinguir si un NFT es mutable o inmutable, y si el código fuente del contrato está disponible y ha sido auditado.

*El contenido de la inscripción siempre está en la cadena.*

No hay forma de que una inscripción se refiera a contenido fuera de la cadena. Esto hace que las inscripciones sean más duraderas, porque el contenido no se puede perder, y más escasas, porque los creadores de inscripciones deben pagar tarifas proporcionales al tamaño del contenido.

Algunos contenidos de NFT de Ethereum están en la cadena, pero mucho está fuera de la cadena, y se almacenan en plataformas como IPFS o Arweave, o en servidores web completamente centralizados. El contenido en IPFS no está garantizado para continuar estando disponible, y ya se ha perdido algún contenido de NFT almacenado en IPFS. Plataformas como Arweave dependen de supuestos económicos débiles y probablemente fallarán catastróficamente cuando estos supuestos económicos ya no se cumplan. Los servidores web centralizados pueden desaparecer en cualquier momento.

Es muy difícil para un usuario no técnico determinar dónde se almacena el contenido de un NFT de Ethereum dado.

*Las inscripciones son mucho más simples.*

Los NFTs de Ethereum dependen de la red y la máquina virtual de Ethereum, que son altamente complejas, están en constante cambio e introducen cambios a través de bifurcaciones duras incompatibles con versiones anteriores.

Las inscripciones, por otro lado, dependen de la cadena de bloques de Bitcoin, que es relativamente simple y conservadora, e introduce cambios a través de bifurcaciones suaves compatibles con versiones anteriores.

*Las inscripciones son más seguras.*

Las inscripciones heredan el modelo de transacción de Bitcoin, que permite a un usuario ver exactamente qué inscripciones está transfiriendo una transacción antes de firmarla. Las inscripciones se pueden ofrecer a la venta utilizando transacciones parcialmente firmadas, que no requieren permitir que un tercero, como una bolsa o un mercado, las transfiera en nombre del usuario.

En comparación, los NFTs de Ethereum están plagados de vulnerabilidades de seguridad para el usuario final. Es común firmar transacciones a ciegas, otorgar a aplicaciones de terceros permisos ilimitados sobre los NFTs de un usuario e interactuar con contratos inteligentes complejos e impredecibles. Esto crea un campo minado de peligros para los usuarios de NFT de Ethereum que simplemente no son una preocupación para los teóricos ordinales.

*Las inscripciones son más escasas.*

Las inscripciones requieren bitcoin para acuñar, transferir y almacenar. Esto parece una desventaja en la superficie, pero la razón de ser de los artefactos digitales es ser escasos y, por lo tanto, valiosos.

Los NFTs de Ethereum, por otro lado, se pueden acuñar en cantidades virtualmente ilimitadas con una sola transacción, haciéndolos inherentemente menos escasos y, por lo tanto, potencialmente menos valiosos.

*Las inscripciones no pretenden admitir regalías en la cadena.*

Las regalías en la cadena son una buena idea en teoría, pero no en la práctica. El pago de regalías no se puede hacer cumplir en la cadena sin restricciones complejas e invasivas. El ecosistema de NFT de Ethereum está lidiando actualmente con la confusión en torno a las regalías, y está llegando colectivamente a la realidad de que las regalías en la cadena, que se comunicaron a los artistas como una ventaja de los NFTs, no son posibles, mientras las plataformas compiten por el fondo y eliminan el soporte de regalías.

Las inscripciones evitan esta situación por completo al no hacer falsas promesas de apoyar regalías en la cadena, evitando así la confusión, el caos y la negatividad de la situación de NFT de Ethereum.

*Las inscripciones desbloquean nuevos mercados.*

La capitalización de mercado y liquidez de Bitcoin son mayores que las de Ethereum por un amplio margen. Gran parte de esta liquidez no está disponible para los NFTs de Ethereum, ya que muchos bitcoiners prefieren no interactuar con el ecosistema de Ethereum debido a preocupaciones relacionadas con la simplicidad, seguridad y descentralización.

Estos bitcoiners pueden estar más interesados en inscripciones que en NFTs de Ethereum, desbloqueando nuevas clases de coleccionistas.

*Las inscripciones cuentan con un modelo de datos más sofisticado.*

Las inscripciones consisten en un tipo de contenido, también conocido como tipo MIME, y contenido, que es una cadena de bytes arbitraria. Este es el mismo modelo de datos utilizado por la web, y permite que el contenido de la inscripción evolucione con la web y llegue a admitir cualquier tipo de contenido compatible con los navegadores web, sin requerir cambios en el protocolo subyacente.

¡Por supuesto! Aquí tienes la traducción al español:

### ¿Activos RGB y Taro?

RGB y Taro son ambos protocolos de activos en la segunda capa construidos sobre Bitcoin. Comparados con las inscripciones, son mucho más complicados, pero mucho más ricos en características.

La teoría ordinal ha sido diseñada desde cero para artefactos digitales, mientras que el caso de uso principal de RGB y Taro son tokens fungibles, por lo que la experiencia de usuario para las inscripciones probablemente sea más sencilla y pulida que la experiencia de usuario para los NFTs de RGB y Taro.

RGB y Taro almacenan contenido fuera de la cadena, lo cual requiere una infraestructura adicional y que podría perderse. Por el contrario, el contenido de la inscripción se almacena en la cadena y no puede perderse.

La teoría ordinal, RGB y Taro están en una etapa muy temprana, por lo que esto es especulación, pero el enfoque de la teoría ordinal podría darle ventaja en términos de características para artefactos digitales, incluyendo un mejor modelo de contenido y características como símbolos globalmente únicos.

### ¿Activos Counterparty?

Counterparty tiene su propio token, XCP, que es necesario para algunas funcionalidades, lo que hace que la mayoría de los bitcoiners lo consideren como una altcoin, y no como una extensión o segunda capa para Bitcoin.

La teoría ordinal ha sido diseñada desde cero para artefactos digitales, mientras que Counterparty fue diseñado principalmente para la emisión de tokens financieros.

Inscripciones para...
-----------------

### Artistas

*Las inscripciones están en Bitcoin.* Bitcoin es la moneda digital con el mayor estatus y la mayor posibilidad de supervivencia a largo plazo. Si quieres garantizar que tu arte sobreviva en el futuro, no hay mejor manera de publicarlo que como inscripciones.

*Almacenamiento en cadena más barato.* A $20,000 por BTC y la tarifa mínima de retransmisión de 1 satoshi por vbyte, publicar contenido de inscripción cuesta $50 por 1 millón de bytes.

*¡Las inscripciones son algo nuevo!* Las inscripciones aún están en desarrollo y aún no se han lanzado en la red principal. Esto te da la oportunidad de ser un adoptante temprano y explorar el medio a medida que evoluciona.

*Las inscripciones son simples.* Las inscripciones no requieren escribir ni entender contratos inteligentes.

*Las inscripciones desbloquean nueva liquidez.* Las inscripciones son más accesibles y atractivas para los tenedores de bitcoin, desbloqueando una clase completamente nueva de coleccionista.

*Las inscripciones están diseñadas para artefactos digitales.* Las inscripciones están diseñadas desde cero para soportar NFTs, y cuentan con un mejor modelo de datos, y características como símbolos globalmente únicos y una procedencia mejorada.

*Las inscripciones no admiten regalías en la cadena.* Esto es negativo, pero solo dependiendo de cómo lo mires. Las regalías en la cadena han sido una bendición para los creadores, pero también han creado una gran cantidad de confusión en el ecosistema NFT de Ethereum. El ecosistema ahora lucha con este problema y está inmerso en una carrera hacia el fondo, hacia un futuro opcional de regalías. Las inscripciones no tienen soporte para regalías en la cadena, porque son técnicamente inviables. Si eliges crear inscripciones, hay muchas formas de sortear esta limitación: retener una parte de tus inscripciones para la venta futura, para beneficiarte de la apreciación futura, o quizás ofrecer incentivos para los usuarios que respeten las regalías opcionales.

### Coleccionistas

*Las inscripciones son simples, claras y no tienen sorpresas.* Siempre son inmutables y en la cadena, sin diligencia debida especial requerida.

*Las inscripciones están en Bitcoin.* Puedes verificar la ubicación y las propiedades de las inscripciones fácilmente con un nodo completo de Bitcoin que controles.

### Bitcoiners

Permíteme comenzar esta sección diciendo: lo más importante que hace la red Bitcoin es descentralizar el dinero. Todos los demás casos de uso son secundarios, incluyendo la teoría ordinal. Los desarrolladores de la teoría ordinal comprenden y reconocen esto, y creen que la teoría ordinal ayuda, al menos de alguna manera, a la misión principal de Bitcoin.

A diferencia de muchas otras cosas en el espacio de las altcoins, los artefactos digitales tienen mérito. Hay, por supuesto, una gran cantidad de NFTs que son feos, estúpidos y fraudulentos. Sin embargo, hay muchos que son fantásticamente creativos, y crear y coleccionar arte ha sido parte de la historia humana desde su inicio, y antecede incluso al comercio y al dinero, que también son tecnologías antiguas.

Bitcoin proporciona una plataforma asombrosa para crear y coleccionar artefactos digitales de una manera segura y descentralizada, que protege a los usuarios y artistas de la misma manera que proporciona una plataforma asombrosa para enviar y recibir valor, y por todas las mismas razones.

Los ordinales e inscripciones aumentan la demanda de espacio en la cadena de bloques de Bitcoin, lo que incrementa el presupuesto de seguridad de Bitcoin, que es vital para salvaguardar la transición de Bitcoin a un modelo de seguridad dependiente de tarifas, ya que el subsidio de bloque se reduce a la insignificancia.

El contenido de la inscripción se almacena en la cadena, y por lo tanto, la demanda de espacio en la cadena de bloques para su uso en inscripciones es ilimitada. Esto crea un comprador de último recurso para *todo* el espacio en la cadena de bloques de Bitcoin. Esto ayudará a mantener un mercado de tarifas robusto, lo que garantiza que Bitcoin permanezca seguro.

Las inscripciones también contrarrestan la narrativa de que Bitcoin no puede ser extendido o utilizado para nuevos casos de uso. Si sigues proyectos como DLCs, Fedimint, Lightning, Taro y RGB, sabes que esta narrativa es falsa, pero las inscripciones proporcionan un contraargumento fácil de entender, y que se dirige a un caso de uso popular y probado, los NFTs, lo que lo hace altamente legible.

Si las inscripciones demuestran, como esperan los autores, ser artefactos digitales muy buscados con una rica historia, servirán como un gancho poderoso para la adopción de Bitcoin: venir por el arte divertido y rico, quedarse por el dinero digital descentralizado.

Las inscripciones son una fuente extremadamente benigna de demanda de espacio en la cadena de bloques. A diferencia, por ejemplo, de las stablecoins, que potencialmente dan a los emisores de stablecoins grandes influencia sobre el futuro del desarrollo de Bitcoin, o DeFi, que podría centralizar la minería al introducir oportunidades para MEV, el arte y los coleccionables digitales en Bitcoin, es poco probable que produzcan entidades individuales con suficiente poder para corromper Bitcoin. El arte es descentralizado.

Los usuarios de inscripciones y los proveedores de servicios tienen incentivos para ejecutar nodos completos de Bitcoin, para publicar y rastrear inscripciones, y así lanzar su peso económico detrás de la cadena honesta.

La teoría ordinal y las inscripciones no afectan significativamente la fungibilidad de Bitcoin. Los usuarios de Bitcoin pueden ignorar ambos y no verse afectados.

Esperamos que la teoría ordinal fortalezca y enriquezca a Bitcoin, y le dé otra dimensión de atractivo y funcionalidad, permitiéndole servir de manera más eficaz su caso de uso principal como la reserva descentralizada de valor de la humanidad.
