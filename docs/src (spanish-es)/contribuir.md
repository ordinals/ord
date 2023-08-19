Contribuir a `ord`
=====================

Pasos Sugeridos
---------------

1. Encuentra un problema en el que quieras trabajar.
2. Comprende cuál sería un buen primer paso para resolver el problema. Esto
   podría ser en forma de código, investigación, una propuesta, o sugerir que se
   cierre, si está obsoleto o no es una buena idea en primer lugar.
3. Comenta el problema con una descripción de tu primer paso sugerido y
   pide retroalimentación. Por supuesto, puedes sumergirte y comenzar a escribir código o
   pruebas inmediatamente, pero esto evita esfuerzos potencialmente desperdiciados, si el problema
   está obsoleto, no está claramente especificado, está bloqueado por algo más, o
   de cualquier forma no está listo para ser implementado.
4. Si el problema requiere una modificación del código o una corrección de errores, abre un PR en borrador con las pruebas,
   y pide retroalimentación. Esto asegura que todos estén en la misma página
   respecto a qué se debe hacer, o cuál debería ser el primer paso para resolver el problema.
   Además, dado que las pruebas son requeridas, escribir primero las pruebas facilita confirmar que la modificación puede ser fácilmente probada.
5. Golpea el teclado al azar hasta que las pruebas pasen, y refactoriza hasta que el código
   esté listo para ser enviado.
6. Marca el PR como listo para revisión.
7. Revisa el PR si es necesario.
8. ¡Y finalmente haz el merge!

Comienza con pequeños pasos
-----------

Las pequeñas modificaciones te permitirán tener un impacto
rápido, y si tomas el camino equivocado, no habrás desperdiciado mucho tiempo.

Ideas para problemas pequeños:
- Añade una nueva prueba o caso de prueba que aumente la cobertura de las pruebas
- Añade o mejora la documentación
- Encuentra un problema que requiera más investigación y realiza esa investigación y resume
   en un comentario
- Encuentra un problema obsoleto y comenta que puede ser cerrado
- Encuentra un problema que no debería hacerse y proporciona retroalimentación constructiva
   explicando por qué crees que es así

Haz merge temprano y a menudo
---------------------

Divide tareas grandes en pasos más pequeños que individualmente hagan
progreso. Si hay un bug, puedes abrir un PR que añade una prueba ignorada fallida.
Esto puede ser fusionado, y el siguiente paso puede ser corregir el bug y dejar de ignorar la prueba.
Realiza investigaciones o pruebas, e informa tus resultados. Divide una funcionalidad en
subfuncionalidades más pequeñas, e impleméntalas una a la vez.

Entender cómo dividir un PR más grande en PR más pequeños que pueden ser
fusionados es una forma de arte que vale la pena practicar. La parte difícil es que cada PR debe
por sí mismo representar una mejora.

Me esfuerzo por seguir este consejo yo mismo, y siempre me va mejor cuando lo hago.

Las pequeñas modificaciones son rápidas de escribir, de revisar y de fusionar, lo que es mucho más divertido que
trabajar en un solo PR gigante que tarda una eternidad en ser escrito, revisado y fusionado.
Las pequeñas modificaciones no requieren mucho tiempo, así que si tienes que dejar de trabajar en una pequeña
modificación, no habrás desperdiciado mucho tiempo en comparación con una modificación más grande que
representa muchas horas de trabajo. Hacer entrar rápidamente un PR mejora el proyecto un
poco de inmediato, en lugar de tener que esperar mucho tiempo para una mejora más grande.
Las pequeñas modificaciones son menos propensas a acumular conflictos de merge.
Como decían los atenienses: *Los rápidos cometen lo que quieren, los lentos fusionan lo que deben.*

Pide ayuda
--------

Si te quedas atascado por más de 15 minutos, pide ayuda, como en un Discord de Rust,
Stack Exchange, o en un problema de proyecto o discusión.

Practica la depuración guiada por hipótesis
------------------------------------

Formula una hipótesis sobre qué está causando el problema. Encuentra la manera de
probar esa hipótesis. Realiza esa prueba. Si funciona, genial,
has resuelto el problema o ahora sabes cómo resolver el problema.
Si no, repite con una nueva hipótesis.

Presta atención a los mensajes de error
-------------------------------

Lee todos los mensajes de error y no toleres advertencias.
