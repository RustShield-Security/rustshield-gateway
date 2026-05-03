# Whitepaper: Mitigación de Inyección de Comandos en Sistemas UAV mediante Filtrado Semántico en Rust

**Fecha:** Mayo 2026  
**Autor:** RustShield Labs  
**Categoría:** Ciberseguridad Aeroespacial / Robótica

## 1. Resumen Ejecutivo
A pesar de la madurez de los ecosistemas ArduPilot y PX4, el protocolo MAVLink sigue presentando retos críticos de seguridad en su capa de transporte. La falta de autenticación y cifrado por defecto permite ataques de inyección de comandos que pueden resultar en la pérdida total del vehículo. Este documento analiza la mitigación mediante **filtrado semántico** y **auditoría criptográfica** en modo audit mediante una solución de **RustShield Gateway** desarrollada en Rust con latencia interna objetivo inferior a 1 ms en condiciones nominales.

## 2. El Problema: Vulnerabilidad Estructural en MAVLink
El protocolo MAVLink (Micro Air Vehicle Link) fue diseñado para la eficiencia, no para la seguridad. Aunque MAVLink 2.0 introdujo la firma de paquetes (signing), su adopción en flotas comerciales es limitada. Sin una pasarela de validación, el sistema es vulnerable a ataques de **inyección de paquetes**, **manipulación de datos** y **ataques de repetición (Replay)**.

### Casos Críticos de Ataque
1.  **ARM-AUTO-001:** Inyección de un comando `MAV_CMD_COMPONENT_ARM_DISARM` en modo `AUTO`.
2.  **Replay Attack:** Captura y reenvío de comandos legítimos para forzar estados del vehículo en momentos no autorizados.
3.  **Signature Tampering:** Modificación de parámetros en paquetes firmados para desviar misiones.

## 3. La Solución: Filtrado Semántico con RustShield Gateway
En lugar de modificar el firmware del autopiloto (proceso costoso y que requiere recertificación), proponemos la inserción de una **Pasarela de Seguridad Transparente** situada entre el Datalink y el Autopiloto.

### Arquitectura del Sistema
El sistema actúa como un proxy bidireccional que realiza una **inspección profunda de paquetes (DPI)** y validación criptográfica en tiempo real:
*   **Observación de Estado:** El gateway mantiene un gemelo digital del estado de vuelo (`FlightState`) monitorizando los mensajes `HEARTBEAT`.
*   **Filtrado Semántico:** Valida el contexto operacional de cada comando recibido.
*   **Auditoría Criptográfica:** Observa firmas MAVLink 2.0, firmas inválidas y replay en modo `audit`; el modo `enforce` queda pendiente de ADR y validación controlada.
*   **Modo Auditoría (Non-Breaking):** Permite validar la seguridad de la red sin interrumpir el flujo operativo, reportando anomalías antes de activar el bloqueo.

## 4. Por qué Rust?
La elección de Rust para este componente garantiza seguridad y performance sin compromisos:
*   **Memory Safety:** Eliminación de vulnerabilidades de desbordamiento en el parseo de protocolos binarios.
*   **Latencia Medida:** Microbenchmarks internos y evidencias SITL/QGroundControl mantienen el procesamiento del gateway bajo 1 ms en la corrida documentada, separando latencia interna de GCS, simulador y sistema operativo.
*   **Concurrencia Segura:** Uso de `Tokio` para manejar flujos asíncronos bidireccionales con alta disponibilidad.

## 5. Resultados y Validación
El sistema ha sido validado en entornos de simulación de alta fidelidad (SITL) y en laboratorios de auditoría criptográfica dedicados:
*   **Eficacia:** Bloqueo de los casos cubiertos por `ARM-AUTO-001` y del catálogo crítico inicial dentro del alcance validado.
*   **Performance:** `latency_benchmark` cubre microbenchmarks internos; la evidencia SITL registra `processing_avg_us=60.769` y `processing_latency_max_us=246` en la corrida documentada.
*   **Seguridad de Red:** Pruebas de **Fuzzing** (`cargo-fuzz`) y corpus MAVLink ampliado sin crash lógico en la campaña documentada.
*   **Trazabilidad:** Generación de ADRs, requisitos, pruebas y evidencias preparatorias para una futura estrategia de assurance.

## 6. Conclusión
La ciberseguridad en drones ya no es una opción, es un requisito regulatorio y operativo. El uso de gateways de seguridad externos permite elevar el nivel de protección de flotas existentes con una ruta incremental: simulación, laboratorio controlado, hardware de banco y, solo después, una estrategia formal de assurance.
