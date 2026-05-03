# Especificación de Observabilidad

## 1. Objetivo

Definir logs, métricas y trazas mínimas para operar y auditar el gateway sin exponer secretos ni generar ruido que degrade latencia.

## 2. Logs Estructurados

Formato recomendado: JSON o key-value estructurado mediante `tracing`.

Eventos mínimos:

- `gateway.start`;
- `gateway.shutdown`;
- `transport.opened`;
- `transport.error`;
- `mavlink.parse_error`;
- `mavlink.signed_observed`;
- `mavlink.signing_validated`;
- `mavlink.signing_rejected`;
- `mavlink.setup_signing_observed`;
- `flight_state.mode_changed`;
- `security.command_observed`;
- `security.command_blocked`;
- `security.audit_only`;
- `crypto.error`;
- `config.validation_error`.

## 3. Campos de Seguridad

Campos recomendados:

- `timestamp`;
- `event`;
- `rule_id`;
- `decision`;
- `transport`;
- `source_ip`;
- `source_port`;
- `message_id`;
- `command`;
- `severity`;
- `system_id`;
- `component_id`;
- `target_system`;
- `target_component`;
- `flight_mode`;
- `reason`.

Campos prohibidos:

- claves;
- payload completo por defecto;
- material derivado de claves;
- datos sensibles innecesarios;
- variables de entorno completas.

## 4. Métricas

Contadores:

- `packets_received_total`;
- `packets_forwarded_total`;
- `packets_blocked_total`;
- `packets_parse_error_total`;
- `packets_signed_observed_total`;
- `packets_signed_valid_total`;
- `packets_signed_invalid_total`;
- `packets_unsigned_rejected_total`;
- `signing_replay_rejected_total`;
- `setup_signing_observed_total`;
- `packets_crypto_error_total`;
- `commands_critical_observed_total`;

Histogramas:

- `processing_latency_ms`;
- `parse_latency_ms`;
- `policy_latency_ms`;
- `crypto_latency_ms`;

Gauges:

- `channels_depth`;
- `last_heartbeat_age_ms`;

## 4.1 Endpoint Read-Only de Laboratorio

Cuando `metrics.enabled = true` y `metrics.readonly_bind` está configurado, el
gateway puede abrir un endpoint HTTP read-only para laboratorio.

Rutas:

- `GET /healthz`: JSON mínimo con estado, `read_only=true`, uptime y contadores
  agregados principales.
- `GET /metrics`: texto tipo Prometheus con contadores y latencias acumuladas.

Restricciones:

- no acepta `POST`, `PUT`, `PATCH` ni `DELETE`;
- no modifica configuración, políticas, claves, estado de vuelo ni transporte;
- no expone payload MAVLink, claves, material derivado, firma completa ni
  contenido de `SETUP_SIGNING`;
- por defecto no se abre socket si `metrics.readonly_bind` no está configurado;
- en esta fase solo se aceptan direcciones loopback.

Eventos adicionales:

- `observability.readonly_opened`;
- `observability.readonly_error`.

## 4.2 Consola Read-Only de Laboratorio

La consola local read-only consume `/healthz` y `/metrics` desde navegador para
visualizar salud, tráfico, seguridad, signing y latencia interna.

Restricciones:

- no envía comandos MAVLink;
- no cambia configuración, políticas, claves, transporte, misiones, modos ni
  parámetros;
- no muestra payload completo, claves, material derivado, firma completa ni
  contenido de `SETUP_SIGNING`;
- puede funcionar con fixtures locales cuando el gateway no está arrancado;
- debe distinguir paquetes firmados observados de autenticación validada.

## 5. Niveles de Log

- `error`: fallo crítico o configuración inválida.
- `warn`: bloqueo, origen inesperado, parse error repetido.
- `info`: arranque, shutdown, cambios de modo, resumen operativo.
- `debug`: detalles de clasificación sin payload completo.
- `trace`: reservado para desarrollo local, nunca con secretos.

## 6. Requisitos de Rendimiento

- Logging no debe bloquear el pipeline principal.
- Eventos de alto volumen deben tener rate limiting o agregación.
- Métricas deben tener coste bajo y cardinalidad controlada.
