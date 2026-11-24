# language: es
Característica: Operación local-first y sincronización de la terminal de portería
  Como vigilante de portería
  Quiero que la terminal funcione sin internet y se sincronice en horas fuera de pico
  Para no detener la operación y mantener al servidor central actualizado

  Antecedentes:
    Dado que el vigilante inició sesión en la terminal

  Escenario: Registrar datos sin conexión
    Dado que el servidor no está disponible
    Cuando el vigilante registra un nuevo residente
    Entonces el residente aparece en la lista inmediatamente
    Y queda guardado en "estado_local.json"
    Y queda un cambio pendiente en "cola_envio.json"

  Escenario: Los datos sobreviven al cierre de la aplicación
    Dado que el vigilante dio salida a la visita "Carlos Andrés Ramírez"
    Cuando la aplicación se cierra y se vuelve a abrir
    Entonces la visita "Carlos Andrés Ramírez" ya no aparece como activa

  Esquema del escenario: Sincronizar en las ventanas fuera de hora pico
    Dado que el reloj de la terminal marca "<antes>"
    Cuando el reloj avanza a "<despues>"
    Entonces la terminal <accion> una sincronización

    Ejemplos:
      | antes | despues | accion     |
      | 09:59 | 10:00   | inicia     |
      | 14:59 | 15:01   | inicia     |
      | 21:59 | 22:00   | inicia     |
      | 02:59 | 03:00   | inicia     |
      | 10:01 | 14:59   | no inicia  |

  Escenario: Indicador durante la sincronización
    Cuando comienza una sincronización
    Entonces a la derecha del nombre del usuario se muestra "SINCRONIZANDO…"
    Cuando la sincronización termina con éxito
    Entonces se muestra "ÚLTIMA SINCRONIZACIÓN" con la fecha y hora actual

  Escenario: Sincronización fallida
    Dado que existe el archivo "SIN_CONEXION" en la carpeta del servidor simulado
    Cuando llega una ventana de sincronización
    Entonces el indicador muestra "SIN CONEXIÓN" con la fecha de la última sincronización exitosa
    Y la cola de envío conserva todos sus cambios

  Escenario: Cambios de otra terminal llegan en la siguiente ventana
    Dado que la terminal B registró el paquete de "Amazon" para "Torre 1 - Apto 101" y ya sincronizó
    Cuando la terminal A sincroniza
    Entonces el paquete aparece en la lista de paquetería pendiente de la terminal A

  Escenario: Gana la terminal ante un conflicto
    Dado que la terminal A tiene pendiente de envío un cambio del residente "r1"
    Y el servidor tiene un cambio distinto del residente "r1" hecho por la terminal B
    Cuando la terminal A sincroniza
    Entonces el residente "r1" de la terminal A conserva su versión local
    Y el servidor queda con la versión de la terminal A

  Escenario: Sincronización manual con F9 o el botón SINCRONIZAR
    Dado que el vigilante acaba de iniciar sesión
    Cuando pulsa la tecla "F9"
    Entonces el indicador muestra "SINCRONIZANDO…" y el botón queda deshabilitado
    Y al terminar muestra "ÚLTIMA SINCRONIZACIÓN" con la hora actual

  Escenario: Ignorar una sincronización manual si ya hay una en curso
    Dado que hay una sincronización en curso
    Cuando el vigilante pulsa "F9"
    Entonces no se inicia otra sincronización
    Y la barra de estado indica "Ya hay una sincronización en curso."
