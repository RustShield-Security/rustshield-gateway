# Matriz de Compatibilidad: GCS, Autopilotos y MAVLink

## 1. Objetivo

Definir cómo se evaluará la compatibilidad del gateway con estaciones de control, autopilotos, dialectos MAVLink, transportes, signing y cifrado. Este documento evita que una decisión de seguridad rompa silenciosamente la operación esperada.

## 2. Principios

- Compatibilidad no significa permitir todo: significa entender qué se permite, qué se bloquea y por qué.
- QGroundControl y autopilotos SITL deben ser el primer banco de pruebas.
- El gateway debe preservar semántica MAVLink por defecto.
- El cifrado externo no es transparente para una GCS estándar.
- MAVLink signing y cifrado externo resuelven problemas distintos.

## 3. Matriz Inicial

| Combinación | Estado objetivo | Riesgo principal | Validación mínima |
|---|---|---|---|
| QGroundControl + ArduPilot SITL + UDP + MAVLink 2 | Prioridad alta | Interpretación de modo y comandos ArduPilot | Conexión, telemetría, bloqueo ARM |
| QGroundControl + PX4 SITL + UDP + MAVLink 2 | Prioridad alta | Diferencias de modo y perfiles de streaming | Conexión, HEARTBEAT, comandos críticos |
| Mission Planner + ArduPilot SITL + UDP | Prioridad media | Semántica específica de ArduPilot | Telemetría y ARM/DISARM |
| GCS estándar + flujo cifrado directo | No compatible por defecto | La GCS espera MAVLink plano | Debe fallar con mensaje claro de configuración |
| Gateway-to-gateway + cifrado | Compatible si se implementa receptor | Nonce, claves, pérdida de paquetes | Test de ida y vuelta cifrado |
| MAVLink signing activo + gateway transparente | Limitado en MVP 0.1 | No validar firma pero tampoco romperla | ADR 0008 y prueba de transparencia |
| MAVLink signing activo + política `observe` | Post-MVP diseño | Falsa autenticación si se interpreta el flag como identidad | ADR 0009, `mavlink.signed_observed`, `authenticated=false` |
| MAVLink signing activo + política `audit` | Laboratorio post-MVP | Falsa seguridad si se interpreta audit como bloqueo | ADR 0009, evidencia signing audit y `authenticated=true` solo tras validación |
| MAVLink signing activo + política `enforce` para comandos críticos | Implementado para laboratorio | Incompatibilidad si GCS/autopiloto no comparten clave, `link_id` o timestamp | ADR 0010; tests `transport::signing_enforce_*`; pendiente SITL/QGroundControl con claves de laboratorio |
| MAVLink signing activo + reserialización | Riesgo alto | Firma inválida | ADR antes de implementar |
| Serial autopiloto real + GCS UDP | Futuro | Seguridad física y operación real | Solo laboratorio controlado |

## 4. Criterios por Dimensión

### 4.1 GCS

Para cada GCS se documentará:

- versión;
- protocolo MAVLink configurado;
- system id usado por la GCS;
- comportamiento de heartbeat;
- soporte o no de signing;
- capacidad de recibir tráfico cifrado externo;
- configuración de enlaces UDP.

### 4.2 Autopiloto

Para cada autopiloto se documentará:

- versión;
- dialecto MAVLink relevante;
- mapping de modos de vuelo;
- comportamiento de `HEARTBEAT`;
- comandos soportados;
- routing;
- perfiles de streaming.

### 4.3 Transporte

UDP:

- permite identificación débil por IP y puerto;
- susceptible a spoofing y replay;
- adecuado para SITL y pruebas locales.

Serial:

- no aporta IP de origen;
- requiere política distinta de identidad;
- mayor sensibilidad a baud rate, timeouts y desconexiones.

## 5. Pruebas Obligatorias de Compatibilidad

- La GCS muestra telemetría básica a través del gateway.
- El gateway recibe `HEARTBEAT` y actualiza estado.
- Un comando permitido se reenvía.
- Un comando `ARM` no certificado en modo automático se bloquea.
- Los logs explican la decisión.
- El modo cifrado no se activa contra GCS estándar sin receptor compatible.
- El gateway conserva `sysid`, `compid`, `target_system` y `target_component` salvo decisión documentada.

## 6. Decisiones Pendientes

- GCS primaria para la primera release.
- Autopiloto primario para la primera release.
- Política ante MAVLink 1.
- Política posterior para validación completa de paquetes MAVLink 2 firmados.
- Modo de despliegue cifrado: directo, plugin, gateway-to-gateway o deshabilitado en release inicial.
