FAQ sobre la Teoría Ordinal
==================

¿Qué es la teoría ordinal?
-----------------------

La teoría ordinal es un protocolo para asignar números seriales a los satoshis, la
mínima subdivisión de un bitcoin, y rastrear estos satoshis mientras son
gastados por las transacciones.

Estos números seriales son números grandes, como este 804766073970493. Cada
satoshi, que es ¹⁄₁₀₀₀₀₀₀₀₀ de un bitcoin, tiene un número ordinal.

¿La teoría ordinal requiere una side chain, un token separado, o cambios a Bitcoin?
----------------------------------------------------------------------------------

¡No! La teoría ordinal funciona ahora, sin una side chain, y el único token
necesario es el bitcoin en sí.

¿Para qué es útil la teoría ordinal?
--------------------------------

Coleccionar, intercambiar y diseñar. La teoría ordinal asigna identidades a
los satoshis individuales, permitiendo su rastreo e intercambio individual, como
curiosidades y por valor numismático.

La teoría ordinal también permite las inscripciones, un protocolo para adjuntar contenidos
arbitrarios a los satoshis individuales, transformándolos en artefactos digitales nativos de bitcoin.

¿Cómo funciona la teoría ordinal?
-----------------------------

Los números ordinales se asignan a los satoshis en el orden en que se minan.
El primer satoshi en el primer bloque tiene número ordinal 0, el segundo tiene
número ordinal 1, y el último satoshi del primer bloque tiene número ordinal
4,999,999,999.

Los satoshis residen en las salidas, pero las transacciones destruyen las salidas y crean nuevas,
por lo tanto, la teoría ordinal usa un algoritmo para determinar cómo los satoshis saltan de
las entradas de una transacción a sus salidas.

Afortunadamente, ese algoritmo es muy sencillo.

Los satoshis se transfieren en orden de llegada. Piensa en las entradas de una
transacción como una lista de satoshis, y en las salidas como una lista de espacios,
listos para recibir un satoshi. Para asignar los satoshis de entrada a los espacios, pasa
a través de cada satoshi en las entradas en orden, y asigna cada uno al primer espacio disponible
en las salidas.

Imaginemos una transacción con tres entradas y dos salidas. Las entradas están
a la izquierda de la flecha y las salidas están a la derecha, todas etiquetadas con
sus valores:

    [2] [1] [3] → [4] [2]

Ahora etiquetamos la misma transacción con los números ordinales de los satoshis
que contiene cada entrada, y con signos de interrogación para cada espacio de salida. Los números
ordinales son grandes, por lo que usamos letras para representarlos:

    [a b] [c] [d e f] → [? ? ? ?] [? ?]

Para entender qué satoshi va a qué salida, pasamos a través de los satoshis de entrada
en orden y asignamos cada uno a un signo de interrogación:

    [a b] [c] [d e f] → [a b c d] [e f]

¿Y las comisiones, podrías preguntar? ¡Buena pregunta! Imaginemos la misma
transacción, esta vez con una comisión de dos satoshis. Las transacciones con comisiones envían más
satoshis en las entradas de los que reciben en las salidas, por lo que para transformar nuestra
transacción en una que pague las comisiones, eliminaremos la segunda salida:

    [2] [1] [3] → [4]

Los satoshis <var>e</var> y <var>f</var> ahora no tienen a dónde ir en las
salidas:

    [a b] [c] [d e f] → [a b c d]

Por lo tanto, van al minero que minó el bloque como comisiones. [El 
BIP](https://github.com/ordinals/ord/blob/master/bip.mediawiki) tiene los detalles,
pero en resumen, las comisiones pagadas por las transacciones se tratan como entradas adicionales a la
transacción coinbase, y se ordenan como sus correspondientes transacciones son
ordenadas en el bloque. La transacción coinbase del bloque podría verse
así:

    [SUBSIDY] [e f] → [SUBSIDY e f]

¿Dónde puedo encontrar los detalles técnicos?
------------------------------------------

[¡En el BIP!](https://github.com/ordinals/ord/blob/master/bip.mediawiki)

¿Por qué se llaman "artefactos digitales" a las inscripciones de sat en lugar de "NFT"?
----------------------------------------------------------------------

Una inscripción es un NFT, pero se utiliza el término "artefacto digital" en su lugar,
porque es simple, evocador y familiar.

La frase "artefacto digital" es muy evocadora, incluso para alguien que nunca ha
oído el término antes. En comparación, NFT es un acrónimo y no
proporciona ninguna indicación de lo que significa si nunca has oído el término antes.

Además, "NFT" parece un lenguaje financiero, y tanto la palabra
"fungible" como el sentido de la palabra "token" usada en "NFT" es raro fuera
de los contextos financieros.

¿Cómo se comparan las inscripciones de sat con…
-----------------------------------

### ¿Las NFT de Ethereum?

*Las inscripciones son siempre inmutables.*

Simplemente no hay forma de que el creador de una inscripción,
o el propietario de una inscripción, pueda modificarla después de que se ha creado.

Las NFT de Ethereum pueden ser inmutables, pero muchas no lo son,
y pueden ser cambiadas o eliminadas por el propietario del contrato NFT.

Para asegurarse de que una NFT de Ethereum en particular sea inmutable,
el código del contrato debe ser verificado, lo cual requiere un conocimiento
detallado de la EVM y de la semántica de Solidity.

Es muy difícil para un usuario no técnico determinar si una NFT de Ethereum
es mutable o inmutable, y las plataformas de NFT de Ethereum no hacen ningún esfuerzo
para distinguir si una NFT es mutable o inmutable,
y si el código fuente del contrato está disponible y ha sido verificado.

*El contenido de la inscripción siempre está en la cadena.*

No hay manera de que una inscripción haga referencia a contenidos off-chain.
Esto hace que las inscripciones sean más duraderas, porque el contenido no puede perderse,
y más escasas, porque los creadores de inscripciones deben pagar
comisiones proporcionales al tamaño del contenido.

Algunos contenidos de NFT de Ethereum están en la cadena, pero muchos están off-chain,
y se almacenan en plataformas como IPFS o Arweave, o en servidores web tradicionales,
completamente centralizados. Los contenidos en IPFS no están garantizados para continuar
siendo accesibles, y algunos contenidos de NFT almacenados en IPFS ya se han perdido.
Plataformas como Arweave se basan en débiles suposiciones económicas, y probablemente fallarán
catastróficamente cuando estas suposiciones económicas dejen de ser respetadas.
Los servidores web centralizados pueden desaparecer en cualquier momento.

Es muy difícil para un usuario no técnico determinar
dónde está almacenado el contenido de una NFT de Ethereum en particular.

*Las inscripciones son mucho más simples.*

Las NFT de Ethereum dependen de la red Ethereum y de la máquina
virtual, que son altamente complejas, en constante cambio, y
que introducen modificaciones a través de hard forks incompatibles con las
versiones anteriores.

Las inscripciones, en cambio, dependen de la blockchain de Bitcoin, que es
relativamente simple y conservadora, e introduce modificaciones
a través de soft forks compatibles con versiones anteriores.

*Las inscripciones son más seguras.*

Las inscripciones heredan el modelo de transacción de Bitcoin, que permite
a un usuario ver exactamente qué inscripciones son transferidas por una
transacción antes de que la firme. Las inscripciones pueden ser ofrecidas en venta
utilizando transacciones parcialmente firmadas, que no requieren permitir a un tercero,
como un exchange o un marketplace, transferirlas en nombre del usuario.

Por el contrario, las NFT de Ethereum están plagadas de vulnerabilidades de seguridad para el usuario final.
Es común firmar transacciones a ciegas, otorgar a aplicaciones de terceros
permisos ilimitados sobre las NFT de un usuario, e interactuar con contratos
inteligentes complejos e impredecibles. Esto crea un campo minado de peligros para los usuarios
de NFT de Ethereum que simplemente no son una preocupación para los teóricos ordinales.

*Las inscripciones son más escasas.*

Las inscripciones requieren bitcoin para ser acuñadas, transferidas y almacenadas.
Esto puede parecer un inconveniente en la superficie,
pero la raison d'etre de los artefactos digitales es ser escasos y por lo tanto valiosos.

Las NFT de Ethereum, por otro lado, pueden ser acuñadas en cantidades
prácticamente ilimitadas con una sola transacción,
haciéndolas intrínsecamente menos escasas, y por lo tanto, potencialmente menos valiosas.

*Las inscripciones no pretenden soportar regalías en la cadena.*

Las regalías en la cadena son una buena idea en teoría pero no en la práctica.
El pago de regalías no puede ser aplicado en la cadena sin
restricciones complejas e invasivas. El ecosistema de NFT de Ethereum está
actualmente enfrentando confusión en torno a las regalías, y está
colectivamente enfrentándose a la realidad de que las regalías en la cadena,
que se presentaron a los artistas como una ventaja de las NFT,
no son posibles, mientras que las plataformas compiten al por menor y
eliminan el soporte a las regalías.

Las inscripciones evitan completamente esta situación al no hacer falsas
promesas de soportar regalías en la cadena, evitando así la
confusión, el caos y la negatividad de la situación de las NFT de
Ethereum.

*Las inscripciones desbloquean nuevos mercados.*

La capitalización de mercado y la liquidez de Bitcoin son mayores que
las de Ethereum por un amplio margen. Mucha de esta liquidez
no está disponible para los NFT de Ethereum, ya que muchos
Bitcoiners prefieren no interactuar con el ecosistema Ethereum por
preocupaciones relacionadas con la simplicidad, seguridad y
descentralización.

Estos Bitcoiners podrían estar más interesados en las inscripciones que en los
NFT de Ethereum, desbloqueando nuevas clases de coleccionistas.

*Las inscripciones tienen un modelo de datos más rico.*

Las inscripciones consisten en un tipo de contenido, también conocido como tipo MIME,
y contenido, que es una cadena de bytes arbitraria. Este es el mismo modelo de datos
utilizado por la web, y permite que el contenido de la inscripción evolucione con la web,
y soporte cualquier tipo de contenido soportado por los navegadores web,
sin requerir cambios en el protocolo subyacente.


### ¿Los activos RGB y Taro?

RGB y Taro son ambos protocolos de activos de segundo nivel construidos sobre Bitcoin.
En comparación con las inscripciones, son mucho más complejos, pero también mucho más ricos en funciones.

La teoría ordinal fue diseñada desde cero para los artefactos digitales,
mientras que el uso principal de RGB y Taro son los tokens fungibles,
por lo que la experiencia de usuario para las inscripciones probablemente será más simple y refinada
que la experiencia de usuario para los NFT de RGB y Taro.

RGB y Taro almacenan ambos contenidos off-chain, que requieren infraestructuras adicionales,
y que pueden perderse. En cambio, el contenido de la inscripción
se almacena on-chain y no puede perderse.

La teoría ordinal, RGB, y Taro son todos muy prematuros, así que esto es especulación,
pero el enfoque de la teoría ordinal podría darle una ventaja en términos de funcionalidad
para los artefactos digitales, incluyendo un mejor modelo de contenido,
y características como símbolos globalmente únicos.

### ¿Activo Counterparty?

Counterparty tiene su propio token, XCP, que es necesario para algunas funciones,
lo que hace que la mayoría de los bitcoiners lo vean como un altcoin,
y no como una extensión o un segundo nivel para Bitcoin.

La teoría ordinal fue diseñada desde cero para los artefactos digitales,
mientras que Counterparty fue diseñada principalmente para la emisión de tokens financieros.

Inscripciones para…
-----------------

### Artistas

*Las inscripciones están en Bitcoin.* Bitcoin es la moneda digital con el estatus más alto
y la mayor probabilidad de supervivencia a largo plazo.
Si quieres garantizar que tu arte sobreviva en el futuro,
no hay mejor manera de publicarlo que como inscripciones.

*Almacenamiento on-chain más económico.* A $20.000 por BTC y la comisión mínima
de envío de 1 satoshi por vbyte,
publicar el contenido de la inscripción cuesta $50 por 1 millón de bytes.

*¡Las inscripciones están en su fase inicial!* Las inscripciones todavía están en desarrollo y aún no se han lanzado
en la red principal. Esto te da la oportunidad de ser pionero y
explorar el medio mientras evoluciona.

*¡Las inscripciones son simples!* Las inscripciones no requieren la escritura
o la comprensión de los contratos inteligentes.

*Las inscripciones desbloquean nueva liquidez.* Las inscripciones son más accesibles y
atractivas para los poseedores de bitcoin, desbloqueando una clase de coleccionistas completamente nueva.

*Las inscripciones están diseñadas para artefactos digitales.* Las inscripciones están diseñadas
desde cero para soportar los NFT, y presentan un mejor modelo de datos,
y características como símbolos globalmente únicos y una mejor proveniencia.

*Las inscripciones no soportan los derechos de autor on-chain.* Esto es negativo,
pero solo dependiendo de cómo lo mires. Los derechos
de autor on-chain han sido una bendición para los creadores, pero también han
creado una gran cantidad de confusión en el ecosistema Ethereum
NFT. El ecosistema ahora enfrenta este problema, y se ha comprometido en
una carrera hacia el fondo, hacia un futuro opcional para los derechos de autor. Las
inscripciones no tienen soporte para los derechos de autor on-chain, porque
son técnicamente irrealizables. Si eliges crear inscripciones, hay
muchas formas de sortear este límite: retener una parte de tus
inscripciones para la venta futura, para beneficiarte del
apreciamiento futuro, o quizás ofrecer beneficios para los usuarios que respeten los derechos
de autor opcionales.

### Coleccionistas

*Las inscripciones son simples, claras y no tienen sorpresas.* Siempre son
inmutables y on-chain, sin necesidad de diligencias especiales.

*Las inscripciones están en Bitcoin.* Puedes verificar la ubicación
y las propiedades de las inscripciones fácilmente con un nodo completo de Bitcoin que controles.

### Los Bitcoiners

Permíteme comenzar esta sección diciendo: la función más importante que
la red Bitcoin realiza es descentralizar el dinero. Todos los demás usos
son secundarios, incluyendo la teoría ordinal. Los desarrolladores de la teoría
ordinal comprenden y reconocen esto, y creen que la teoría
ordinal ayuda, al menos en pequeña parte, a la misión principal de Bitcoin.

A diferencia de muchas otras cosas en el espacio altcoin, los artefactos digitales
tienen mérito. Hay, por supuesto, un gran número de NFT que son
feos, tontos y fraudulentos. Sin embargo, hay muchos que son
fantásticamente creativos, y la creación y colección de arte ha sido
parte de la historia humana desde el principio, e incluso precede al
comercio y al dinero, que también son tecnologías antiguas.

Bitcoin proporciona una plataforma extraordinaria para la creación y la
colección de artefactos digitales de forma segura y descentralizada, que
protege a los usuarios y a los artistas de la misma manera que proporciona una
plataforma extraordinaria para enviar y recibir valor, y por todas
las mismas razones.

Los ordinales e inscripciones aumentan la demanda de espacio en los bloques
de Bitcoin, lo que a su vez incrementa el presupuesto de seguridad de Bitcoin, que es esencial para
salvaguardar la transición de Bitcoin hacia un modelo de seguridad
dependiente de las tarifas, a medida que la recompensa
por bloque se reduce a la mitad hasta volverse insignificante.

El contenido de la inscripción se guarda en la cadena de bloques (on-chain), por lo que la
demanda de espacio en los bloques para usar en las inscripciones es ilimitada. Esto
crea un comprador de último recurso para todo el espacio en los bloques de Bitcoin.
Esto ayudará a sostener un robusto mercado de tarifas, que
asegura que Bitcoin permanezca seguro.

Las inscripciones también desafían la narrativa de que Bitcoin
no puede ser extendido o utilizado para nuevos casos de uso. Si sigues
proyectos como DLC, Fedimint, Lightning, Taro y RGB, sabes que esta
narrativa es falsa, pero las inscripciones proporcionan un contra-argumento fácil
de entender, que aborda un caso de uso popular y probado, los NFT,
haciéndolo muy accesible.

Si las inscripciones resultan, como esperan sus creadores, ser artefactos digitales
muy demandados con una rica historia, servirán como un potente imán
para la adopción de Bitcoin: ven por el arte divertido y valioso,
quedate por el dinero digital descentralizado.

Las inscripciones son una fuente extremadamente benigna de demanda de
espacio en el bloque. A diferencia, por ejemplo, de las stablecoins, que
podrían dar a los grandes emisores de stablecoins influencia sobre el futuro
del desarrollo de Bitcoin, o DeFi, que podría centralizar la minería
introduciendo oportunidades para MEV, el arte digital y los coleccionables en
Bitcoin son improbables de producir entidades individuales con suficiente
poder para corromper Bitcoin. El arte es descentralizado.

Los usuarios de inscripciones y los proveedores de servicios están incentivados a
ejecutar nodos completos de Bitcoin, para publicar y rastrear las
inscripciones, y por lo tanto arrojar su peso económico detrás de la cadena
honesta.

La teoría ordinal y las inscripciones no afectan significativamente la
fungibilidad de Bitcoin. Los usuarios de Bitcoin pueden ignorar ambos y no
verse afectados.

Esperamos que la teoría ordinal fortalezca y enriquezca a Bitcoin, y le dé
otra dimensión de atractivo y funcionalidad, permitiéndole servir
más efectivamente su caso de uso primario como depósito de valor
descentralizado de la humanidad.
