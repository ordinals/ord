Contribuyendo a `ord`
====================

Pasos Sugeridos
---------------

1. Encuentra un problema en el que quieras trabajar.
2. Determina cuál sería el primer paso adecuado para resolver el problema. Esto podría ser en forma de código, investigación, una propuesta o sugerir que se cierre, si está desactualizado o no es una buena idea en primer lugar.
3. Comenta en el problema con un esbozo de tu primer paso sugerido y pide comentarios. Por supuesto, puedes sumergirte y comenzar a escribir código o pruebas de inmediato, pero esto evita un esfuerzo potencialmente desperdiciado si el problema está desactualizado, no está claramente especificado, está bloqueado por algo más o de alguna otra manera no está listo para implementarse.
4. Si el problema requiere un cambio de código o una corrección de errores, abre un PR (Pull Request) en modo borrador con pruebas y pide comentarios. Esto asegura que todos estén en la misma página sobre lo que debe hacerse o cuál debería ser el primer paso para resolver el problema. Además, dado que se requieren pruebas, escribir las pruebas primero facilita confirmar que el cambio se puede probar fácilmente.
5. Escribe código hasta que las pruebas pasen y luego refactoriza hasta que el código esté listo para enviar.
6. Marca el PR como listo para revisar.
7. Revisa el PR según sea necesario.
8. ¡Y finalmente, fusiona!

Comienza con cosas pequeñas
---------------------------

Los cambios pequeños te permitirán tener un impacto rápidamente y, si tomas un enfoque equivocado, no habrás perdido mucho tiempo.

Ideas para problemas pequeños:
- Agregar una nueva prueba o caso de prueba que aumente la cobertura de pruebas
- Agregar o mejorar la documentación
- Encontrar un problema que requiere más investigación y realizar esa investigación y resumirla en un comentario
- Encontrar un problema desactualizado y comentar que se puede cerrar
- Encontrar un problema que no deba realizarse y proporcionar comentarios constructivos detallando por qué crees que ese es el caso

Fusiona temprano y a menudo
--------------------------

Divide las tareas grandes en varios pasos más pequeños que progresen individualmente. Si hay un error, puedes abrir un PR que agregue una prueba fallida ignorada. Esto se puede fusionar y el siguiente paso puede ser corregir el error y dejar de ignorar la prueba. Investiga o realiza pruebas y reporta tus resultados. Divide una característica en subcaracterísticas pequeñas e impleméntalas una a la vez.

Descubrir cómo dividir un PR más grande en PR más pequeños que se puedan fusionar es una forma de arte que vale la pena practicar. La parte difícil es que cada PR en sí debe ser una mejora.

Siempre trato de seguir este consejo y siempre me va mejor cuando lo hago.

Los cambios pequeños se escriben, revisan y fusionan rápidamente, lo que es mucho más divertido que trabajar en un solo PR gigante que lleva mucho tiempo escribir, revisar y fusionar. Los cambios pequeños no requieren mucho tiempo, por lo que si necesitas dejar de trabajar en un cambio pequeño, no habrás perdido mucho tiempo en comparación con un cambio más grande que representa muchas horas de trabajo. Obtener un PR rápidamente mejora el proyecto un poco de inmediato, en lugar de tener que esperar mucho tiempo para una mejora más grande. Los cambios pequeños son menos propensos a acumular conflictos de fusión. Como dijeron los atenienses: *Los rápidos hacen lo que quieren, los lentos hacen lo que deben*.

Obtén ayuda
----------

Si te quedas atascado durante más de 15 minutos, pide ayuda, como en un servidor de Discord de Rust, Stack Exchange o en un problema o discusión del proyecto.

Practica la depuración basada en hipótesis
----------------------------------------

Formula una hipótesis sobre lo que está causando el problema. Descubre cómo probar esa hipótesis. Realiza esas pruebas. Si funciona, genial, resolviste el problema o ahora sabes cómo hacerlo. Si no funciona, repite con una nueva hipótesis.

Presta atención a los mensajes de error
------------------------------------

Lee todos los mensajes de error y no toleres las advertencias.
