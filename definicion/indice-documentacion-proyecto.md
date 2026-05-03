# Índice de Documentación del Proyecto

## 1. Objetivo

Este índice organiza los documentos de definición, arquitectura, seguridad, validación y soporte de MAVLink-Rust Shield Gateway. Debe usarse como punto de entrada antes de implementar código o tomar decisiones de diseño.

## 2. Documentos Canónicos Originales

- `requerimiento-tecnico-mavlink-rust-shield-gateway.md`: requerimiento técnico base.
- `documentacion-recomendada-arquitectura-producto.md`: mapa de fuentes y documentación externa recomendada.
- `investigacion-mcp-servers-para-mavlink-rust-shield-gateway.md`: investigación de MCP servers.
- `investigacion-skills-codex-para-mavlink-rust-shield-gateway.md`: investigación de skills Codex.
- `paquete-codex-proyecto.md`: configuración de conocimiento para Codex.
- `configuracion-recomendada-codex-mcp-skills.md`: configuración recomendada de skills/MCP.

## 3. Definición de Producto y Arquitectura

- `prd-mavlink-rust-shield-gateway.md`: problema, usuarios, objetivos, no objetivos, escenarios, requisitos y criterios de aceptación.
- `mvp-0.1-alcance-y-criterios.md`: alcance cerrado de la primera versión implementable.
- `trazabilidad-mvp-0.1.md`: trazabilidad requisito-amenaza-regla-test-log-métrica.
- `arquitectura-arc42-mavlink-rust-shield-gateway.md`: arquitectura con estructura arc42.
- `modelo-amenazas-mavlink-rust-shield-gateway.md`: threat model y riesgos de seguridad.
- `registro-riesgos.md`: registro vivo de riesgos técnicos, de seguridad y compatibilidad.
- `definition-of-done.md`: criterios de finalización para documentos, decisiones, reglas y código futuro.
- `herramientas-implementacion-mvp-0.1.md`: herramientas locales, MCP servers y skills recomendadas para implementar MVP 0.1.
- `verificacion-entorno-pruebas-mvp-0.1.md`: estado verificado del entorno de pruebas MVP 0.1.

## 4. Seguridad y Configuración

- `especificacion-politicas-seguridad.md`: modelo de decisión `allow/block/audit_only/drop_invalid`.
- `especificacion-arm-auto-001.md`: regla `ARM-AUTO-001` convertida en especificación testeable.
- `validacion-modo-ardupilot-sitl.md`: validación de `HEARTBEAT.custom_mode` para ArduPilot Copter SITL.
- `catalogo-comandos-mavlink-criticos.md`: mensajes y comandos MAVLink críticos.
- `especificacion-configuracion.md`: estructura inicial de `config.toml` y validaciones.
- `estrategia-gestion-claves.md`: claves, nonces, rotación futura y restricciones de logging.
- `especificacion-observabilidad.md`: logs, métricas, niveles y campos prohibidos.
- `recomendacion-autenticacion-signing-post-mvp.md`: recomendación inicial de autenticación/signing posterior al MVP.
- `especificacion-autenticacion-signing.md`: política futura `observe/audit/enforce`, estados `authenticated`, eventos, métricas y fixtures.

## 5. Compatibilidad y Validación

- `matriz-compatibilidad-gcs-autopilotos.md`: compatibilidad GCS/autopiloto/transporte/signing/cifrado.
- `plan-validacion-sitl.md`: escenarios de validación en simulación.
- `plan-benchmark-latencia.md`: metodología para objetivo de latencia inferior a 1 ms.
- `plan-fuzzing-mavlink.md`: fuzzing de parser, filtro, configuración y wrapper cifrado.

## 6. Apoyo Terminológico

- `glosario-mavlink-seguridad.md`: términos clave MAVLink, seguridad y criptografía.

## 7. ADRs

- `adr/0001-usar-rust-2021.md`
- `adr/0002-usar-mavlink-2.md`
- `adr/0003-usar-tokio-para-concurrencia.md`
- `adr/0004-cifrado-vs-compatibilidad-gcs.md`
- `adr/0005-politica-ante-modo-desconocido.md`
- `adr/0006-identidad-ip-vs-autenticacion-fuerte.md`
- `adr/0007-gateway-transparente-por-defecto.md`
- `adr/0008-comportamiento-ante-mavlink-signing.md`
- `adr/0009-autenticacion-y-mavlink-signing-post-mvp.md`
- `adr/0010-enforce-signing-comandos-criticos.md`

## 8. Orden Recomendado de Lectura

1. `requerimiento-tecnico-mavlink-rust-shield-gateway.md`
2. `prd-mavlink-rust-shield-gateway.md`
3. `mvp-0.1-alcance-y-criterios.md`
4. `modelo-amenazas-mavlink-rust-shield-gateway.md`
5. `arquitectura-arc42-mavlink-rust-shield-gateway.md`
6. ADRs iniciales.
7. `trazabilidad-mvp-0.1.md`
8. `matriz-compatibilidad-gcs-autopilotos.md`
9. `catalogo-comandos-mavlink-criticos.md`
10. `especificacion-politicas-seguridad.md`
11. `especificacion-arm-auto-001.md`
12. `validacion-modo-ardupilot-sitl.md`
13. `plan-validacion-sitl.md`
14. `plan-benchmark-latencia.md`
15. `plan-fuzzing-mavlink.md`

## 9. Regla de Mantenimiento

Toda decisión que afecte seguridad, compatibilidad, latencia u operación debe actualizar al menos uno de estos documentos y, si cambia arquitectura o comportamiento crítico, debe generar o modificar un ADR.
