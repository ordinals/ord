Colección
==========

Actualmente, [ord](https://github.com/ordinals/ord/) es la única cartera que soporta
control-sat y selección-sat, los cuales son necesarios para almacenar y enviar de manera segura
sats raros e inscripciones, denominados a continuación como ordinales.

La forma recomendada de enviar, recibir y almacenar ordinales es con `ord`, pero si
eres cuidadoso, es posible almacenar de forma segura, y en algunos casos enviar,
ordinales con otras carteras.

Como nota general, recibir ordinales en una cartera no soportada no es
peligroso. Los ordinales pueden ser enviados a cualquier dirección de bitcoin y son seguros siempre que
el UTXO que los contiene no se gaste. Sin embargo, si posteriormente esa cartera se utiliza
para enviar bitcoins, podría seleccionar el UTXO que contiene el ordinal como entrada y
enviar la inscripción o gastarla en comisiones.

Una [guía](./collecting/sparrow-wallet.md) para crear una cartera compatible con `ord` usando [Sparrow Wallet] está disponible
en este manual.

Ten en cuenta que si sigues esta guía, no deberías usar la cartera que
crees para enviar BTC, a menos que realices una selección manual de monedas para evitar enviar
ordinales.
