# Recomendación Post-MVP: Autenticación y MAVLink Signing

## 1. Objetivo

Definir la recomendación inicial para evolucionar desde controles débiles por IP hacia autenticación criptográfica sin romper la transparencia del gateway ni afirmar seguridad que no se valida.

## 2. Alcance

Incluye:

- priorizar MAVLink signing como primera ruta de autenticación compatible con MAVLink 2;
- mantener observabilidad explícita de paquetes firmados;
- preparar ADR posterior antes de validar, rechazar o exigir firmas.

No incluye:

- implementar validación criptográfica en esta fase;
- gestionar claves operativas;
- activar signing obligatorio;
- modificar o reserializar paquetes firmados.

## 3. Recomendación

La siguiente iteración debe preparar un ADR específico para autenticación/signing con esta dirección:

- conservar gateway transparente por defecto;
- observar paquetes firmados con `mavlink.signed_observed`;
- registrar `authenticated=false` mientras no exista validación criptográfica;
- no usar IP allowlist como prueba de identidad fuerte;
- definir política futura por enlace: permitir sin firma, auditar sin firma o exigir firma;
- validar timestamp por `(SystemID, ComponentID, LinkID)` si se implementa signing;
- documentar impacto de replay, rotación de claves, pérdida de sincronía y compatibilidad con QGroundControl/ArduPilot.

La decisión formal queda recogida en
`definicion/adr/0009-autenticacion-y-mavlink-signing-post-mvp.md`.

## 4. Criterios Antes de Implementar

- ADR aceptado para la política de signing.
- Fixtures MAVLink 2 firmados y no firmados.
- Tests de no modificación de bytes para paquetes firmados permitidos.
- Tests negativos de firma inválida, timestamp antiguo y link ID desconocido.
- Plan de gestión de claves separado de configuración insegura por defecto.

## 5. Riesgos Relacionados

- R-002: IP allowlist se interpreta como autenticación fuerte.
- R-007: signing MAVLink roto.
- R-006: routing MAVLink alterado.
