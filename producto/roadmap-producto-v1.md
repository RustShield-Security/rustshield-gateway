# Roadmap: De MVP 0.1 a Producto Industrial (V1.0)

> Estado actualizado: la ruta ejecutable recomendada está definida en
> `producto/ruta-a-producto.md`. Este documento conserva la visión V1.0, pero no
> debe leerse como orden inmediato de implementación.

## 0. Orden Recomendado

La prioridad experta para avanzar hacia producto es:

1. V0.2 Producto piloto: signing `audit` validado, catálogo crítico inicial,
   fuzzing/routing ampliado y evidencia de latencia SITL/QGroundControl
   documentada.
2. V0.3 Operable de laboratorio: ADR/implementación controlada de `enforce`,
   gestión de claves de laboratorio y observabilidad read-only.
3. V0.4 Hardware-lab: ADR Serial/Radio, transporte Serial y HITL de banco sin
   vuelo.
4. V1.0 Industrial piloto: API autenticada, dashboard/operación, empaquetado,
   compatibilidad ampliada y evaluación fail-safe.

Serial, KMS/HSM, dashboard mutable, bypass y certificación formal quedan
condicionados a gates previos. Tras la fase 0017, el siguiente paso recomendado
es decidir por ADR el alcance de `enforce` para MAVLink signing en comandos
críticos.

Este documento detalla la brecha técnica y operativa entre el estado actual (MVP 0.1, ~20% del producto) y una solución comercial competitiva para el sector de drones profesionales y defensa.

## 1. Brecha Funcional (Hardware y Protocolo)

Para superar la fase de simulación y ser útil en el mundo real, el producto debe expandirse en:

- [ ] **Soporte Serial/Radio (UART):** Implementar el transporte por puerto serie usando `tokio-serial`. Los drones reales no suelen exponer telemetría por UDP de forma nativa en el aire.
- [x] **Catálogo de Comandos Críticos Inicial:** Filtrado inicial para misión, parámetros, modo, `REPOSITION`, RC/manual override y reboot en alcance UDP/SITL.
- [ ] **Catálogo de Comandos Extendido por Dialecto:** Ampliar cobertura a payloads, dialectos de fabricante y políticas específicas por autopiloto.
- [ ] **Dialectos Adicionales:** Soporte oficial y validado para PX4 y dialectos específicos de fabricantes (ej. DJI MAVLink bridge).
- [ ] **Manejo de MAVLink 1.0:** Aunque el objetivo es MAVLink 2.0, el producto debe ser capaz de manejar o descartar tráfico 1.0 de forma segura sin romper el flujo.

## 2. Hardening de Seguridad

El MVP 0.1 utiliza controles "débiles" (IP allowlist). El producto final requiere:

- [ ] **MAVLink 2.0 Signing:** Validación criptográfica real de las firmas de los paquetes. El Gateway debe ser el "Secret Keeper".
- [ ] **Protección Anti-Replay:** Gestión de ventanas de secuencia y timestamps para evitar que comandos antiguos autorizados sean reutilizados por un atacante.
- [ ] **Gestión de Claves (KMS):** Integración con módulos de seguridad (HSM) o sistemas de gestión de claves para evitar el almacenamiento de secretos en archivos de configuración.
- [ ] **Handshake de Identidad:** Implementar un flujo de autenticación inicial entre la GCS y el Gateway antes de permitir el tráfico de comandos.

## 3. Operatividad y Gestión

Un producto industrial no puede depender exclusivamente de archivos `.toml` y logs de consola.

- [ ] **API de Configuración (REST/gRPC):** Permitir cambios de políticas en caliente (hot-reload) sin reiniciar el gateway.
- [ ] **Dashboard de Estado:** Interfaz visual (web o plugin de GCS) para ver métricas, bloqueos en tiempo real y salud del enlace.
- [ ] **Logs de Auditoría Remotos:** Exportación de eventos a sistemas externos (SIEM, syslog, ELK) para monitorización centralizada de flotas.
- [ ] **Modo Fail-Safe / Bypass:** Implementar un mecanismo de protección que permita el flujo de datos si el gateway detecta un fallo crítico interno (para evitar la pérdida del vehículo).

## 4. Calidad y Certificación

Para vender en sectores regulados, el "proceso" es tan importante como el "código".

- [ ] **Cobertura MC/DC:** Alcanzar el 100% de cobertura de código en tests unitarios y de integración.
- [ ] **Mapeo Formal de Estándares:** Completar la matriz de trazabilidad alineada con los objetivos de la norma **DO-178C** y **ED-203A**.
- [ ] **Qualified Toolchain:** Migrar el pipeline de CI/CD para usar compiladores calificados (ej. Ferrocene).
- [ ] **HITL (Hardware-in-the-Loop):** Validar el binario en la plataforma de ejecución final (Raspberry Pi, i.MX8, etc.) con hardware real conectado.

## 6. Escalabilidad Industrial (Beyond Drones)

La arquitectura de este producto ha sido diseñada bajo el paradigma de **Stateful Semantic Shielding**, lo que permite su portabilidad a otros sectores de robótica y automatización. 

Para más detalles, consultar el documento: [Portabilidad de la Arquitectura a otros Sectores Industriales](portabilidad-industrial.md).
