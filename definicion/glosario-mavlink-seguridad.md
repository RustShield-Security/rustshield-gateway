# Glosario MAVLink y Seguridad

## GCS

Ground Control Station. Aplicación usada para monitorizar y controlar un vehículo, por ejemplo QGroundControl o Mission Planner.

## MAVLink

Protocolo binario usado para comunicación entre vehículos, componentes, GCS y sistemas asociados.

## MAVLink 2

Versión del protocolo con `message_id` extendido, flags de compatibilidad, extensiones y soporte de signing.

## HEARTBEAT

Mensaje periódico que permite observar presencia, tipo, autopiloto, modo y estado del sistema.

## COMMAND_LONG

Mensaje MAVLink usado para enviar comandos con parámetros en coma flotante.

## COMMAND_INT

Mensaje MAVLink usado para comandos con campos enteros, especialmente útil para coordenadas.

## MAV_CMD_COMPONENT_ARM_DISARM

Comando MAVLink usado para armar o desarmar un vehículo o componente.

## system_id

Identificador del sistema MAVLink emisor.

## component_id

Identificador del componente MAVLink emisor dentro de un sistema.

## target_system

Sistema MAVLink destinatario de un mensaje, cuando el mensaje incluye destino.

## target_component

Componente MAVLink destinatario, cuando aplica.

## Signing

Mecanismo de MAVLink 2 para autenticar mensajes y mitigar manipulación/replay bajo ciertas condiciones.

## AEAD

Authenticated Encryption with Associated Data. Cifrado que proporciona confidencialidad e integridad.

## ChaCha20-Poly1305

Algoritmo AEAD recomendado para cifrado ligero y robusto en software.

## SITL

Software-in-the-loop. Simulación del autopiloto para pruebas sin hardware real.

## Gateway Transparente

Modo en el que el gateway reenvía mensajes preservando semántica y evitando modificaciones salvo decisiones explícitas.

## Wrapper Cifrado

Formato externo que encapsula datos MAVLink cifrados junto con metadatos como nonce y tag de autenticación.

