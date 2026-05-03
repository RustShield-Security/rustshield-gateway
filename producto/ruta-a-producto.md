# Ruta a Producto: MAVLink-Rust Shield Gateway

## 1. Diagnóstico Ejecutivo

El proyecto ya superó la etapa de MVP técnico para UDP + SITL: existe proxy
bidireccional, parser MAVLink, estado de vuelo, reglas iniciales, observabilidad,
fuzzing inicial, benchmark, validación SITL y signing en modo `audit`.

La brecha hacia producto no es "añadir hardware cuanto antes". La brecha real es
convertir el núcleo de seguridad en una cadena verificable:

```text
autenticación -> autorización semántica -> evidencias -> operación controlada
```

Por tanto, la ruta recomendada prioriza:

1. validar signing audit contra escenarios reproducibles;
2. ampliar política de comandos críticos;
3. diseñar y validar `enforce` de forma controlada;
4. introducir operación y hardware solo cuando existan gates de seguridad.

## 2. Principios de Ruta

- No operar hardware real por defecto.
- No activar `enforce` sin ADR, fixtures, recuperación y validación SITL/lab.
- No introducir API de cambios en caliente antes de autenticación y auditoría de
  configuración.
- No inventar un handshake propio si MAVLink signing o un canal autenticado
  estándar cubren el caso.
- No prometer certificación; hablar de "evidencias preparatorias" hasta definir
  alcance regulatorio.
- Separar disponibilidad y seguridad: cualquier bypass requiere ADR porque puede
  permitir tráfico no filtrado.

## 3. Producto Piloto: V0.2

Objetivo: demostrar valor de seguridad con evidencias reproducibles, sin hardware
real y sin afirmar protección completa.

Fases:

1. `0014-validacion-signing-audit-lab.md`
   - Validar runtime `audit` con paquetes firmados válidos, inválidos, replay y
     comandos críticos sin firma.
   - Evidencia: logs, métricas, no exposición de claves y no bloqueo por signing.

2. `0015-catalogo-politicas-comandos-criticos.md`
   - Activar reglas para `PARAM_SET`, misión, cambio de modo, reposition,
     RC/manual override y reboot.
   - Evidencia: tests unitarios, fixtures MAVLink y trazabilidad regla -> riesgo.

3. `0016-endurecimiento-routing-fuzzing.md`
   - Ampliar corpus, casos de routing/transparencia, MAVLink v1/v2 y paquetes
     firmados.
   - Evidencia: campaña fuzzing mayor, no reserialización accidental, métricas de
     parse error.

4. `0017-latencia-end-to-end-sitl.md`
   - Medir latencia end-to-end con QGroundControl + ArduPilot SITL y gateway.
   - Evidencia: `metrics.snapshot`, capturas, logs y comparación con
     microbenchmark.

Gate de salida V0.2:

- signing audit validado;
- catálogo crítico inicial cubierto;
- campaña fuzzing ampliada sin crash;
- latencia SITL/QGroundControl documentada con límites explícitos de medición
  end-to-end;
- claims comerciales corregidos a lo demostrado.

## 4. Producto Operable de Laboratorio: V0.3

Objetivo: permitir pruebas de laboratorio controlado con autenticación fuerte y
operación observable, todavía sin despliegue real.

Fases:

1. `0018-adr-enforce-signing.md`
   - ADR para `enforce`: qué bloquea, por dirección, por tipo de mensaje y cómo
     se recupera ante clave/timestamp desincronizado.

2. `0019-enforce-signing-comandos-criticos.md`
   - Implementar `enforce` solo para comandos críticos GCS -> vehículo.
   - Telemetría no firmada se trata con política explícita separada.

3. `0020-gestion-claves-laboratorio.md`
   - Carga local segura, rotación en laboratorio, redacción de logs y permisos.
   - KMS/HSM queda diseñado, no obligatorio.

4. `0021-observabilidad-operacional-readonly.md`
   - Endpoint local o export de métricas/logs sin cambios de política en caliente.
   - Integración syslog/JSON/ELK como salida, no dashboard complejo.

Gate de salida V0.3:

- `enforce` validado solo en simulación/lab;
- claves tratadas como secreto real;
- observabilidad consumible por herramientas externas;
- procedimiento de recuperación documentado.

## 5. Producto Hardware-Lab: V0.4

Objetivo: validar transportes y plataforma física sin operación aérea.

Fases:

1. `0022-adr-serial-radio-y-hardware-lab.md`
   - Decidir topologías UART/SiK/USB, límites de seguridad y no-go para vuelo.

2. `0023-transporte-serial-laboratorio.md`
   - Implementar `tokio-serial` con tests loopback y MAVLink fixtures.

3. `0024-hitl-sin-vuelo.md`
   - Raspberry Pi / autopiloto en banco, sin hélices ni misión real.
   - Medición de latencia física y pérdida de paquetes.

Gate de salida V0.4:

- Serial validado en banco;
- comportamiento ante pérdida/ruido documentado;
- no hay vuelo ni comandos reales fuera de procedimiento explícito.

## 6. Producto Industrial V1.0

Objetivo: base vendible/transferible para piloto industrial, no certificación
formal completa.

Bloques:

- API de configuración autenticada y auditada, con hot-reload limitado.
- Dashboard o plugin read-mostly para salud, bloqueos y métricas.
- Integración SIEM/syslog/ELK.
- Empaquetado reproducible para plataforma objetivo.
- Evaluación de bypass/watchdog con ADR específico.
- Matriz de compatibilidad ArduPilot/PX4/GCS ampliada.

Gate V1.0:

- instalación reproducible;
- operación documentada;
- telemetría y alertas externas;
- políticas críticas cubiertas;
- evidencia de laboratorio físico;
- modelo de amenazas actualizado.

## 7. Ruta Certificable

La certificación no debe aparecer como promesa implícita de V1.0. Debe tratarse
como línea separada:

- definir estándar objetivo y nivel de assurance;
- identificar frontera certificable;
- congelar requisitos;
- establecer trazabilidad formal;
- decidir toolchain calificado;
- diseñar cobertura estructural, incluida MC/DC si aplica.

Hasta entonces, la formulación correcta es "certification-ready evidence pack",
no "certificado" ni "listo para DO-178C".

## 8. Ajustes Recomendados en Narrativa de Producto

- Cambiar "latencia <400ns por paquete" por "procesamiento interno sub-ms en la
  corrida SITL/QGroundControl documentada; latencia externa de GCS, simulador,
  sistema operativo y red fuera de esa métrica".
- Cambiar "bloqueo del 100% de comandos críticos" por "bloqueo del 100% de los
  casos cubiertos por ARM-AUTO-001 en el alcance validado".
- Cambiar "MAVLink Signing compatible" por "transparente por defecto y validado
  en modo audit; enforce pendiente de fase controlada".
- Cambiar "listas para certificación DO-178C/ED-203A" por "trazabilidad y ADRs
  preparatorios para una estrategia de assurance".
