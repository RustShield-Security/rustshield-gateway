# Plan de Benchmark de Latencia

## 1. Objetivo

Medir si el gateway cumple el objetivo de procesamiento interno inferior a 1 ms por paquete en condiciones nominales, separando latencia propia del gateway de latencias externas.

## 2. Qué Se Mide

- tiempo de recepción a decisión;
- tiempo de parseo;
- tiempo de evaluación de política;
- tiempo de actualización de estado;
- tiempo de cifrado, si aplica;
- tiempo total antes de envío.

## 3. Qué No Se Incluye

- latencia de red externa;
- latencia de la GCS;
- latencia del simulador;
- latencia física del enlace serial;
- scheduling externo no atribuible al proceso, salvo que afecte consistentemente.

## 4. Métricas

- p50;
- p95;
- p99;
- máximo observado;
- mensajes por segundo;
- CPU;
- memoria;
- errores;
- drops.

## 5. Escenarios

| Escenario | Descripción |
|---|---|
| B-001 | UDP sin cifrado, telemetría nominal SITL |
| B-002 | UDP con comandos mezclados |
| B-003 | UDP con tráfico inválido bajo volumen |
| B-004 | UDP con alto volumen |
| B-005 | Cifrado habilitado con test harness |
| B-006 | Logging `info` vs logging detallado |

## 6. Criterios Iniciales

- p95 < 1 ms en escenario nominal.
- p99 documentado aunque supere 1 ms en escenarios adversos.
- Sin crecimiento no acotado de memoria.
- Sin bloqueo del flujo opuesto por operación lenta.

## 7. Herramientas Recomendadas

- `criterion` para microbenchmarks de clasificación, parseo y cifrado.
- métricas internas para pipeline completo.
- capturas de tráfico SITL para corpus reproducible.

## 8. Riesgos

- Medir demasiado pronto sin código estable.
- Confundir latencia de red con latencia interna.
- Logging síncrono distorsiona resultados.
- Hardware objetivo no definido.

