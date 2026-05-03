# Estrategia de Gestión de Claves

## 1. Objetivo

Definir controles mínimos para claves usadas por ChaCha20-Poly1305 u otros mecanismos de autenticación/cifrado. El objetivo es evitar una implementación criptográfica técnicamente correcta pero operacionalmente débil.

## 2. Principios

- No versionar claves.
- No imprimir claves.
- No aceptar claves débiles o de longitud incorrecta.
- No reutilizar nonce con la misma clave.
- Fallar cerrado ante configuración criptográfica inválida.
- Separar cifrado externo de MAVLink signing.

## 3. Fuente de Clave Inicial

Prioridad recomendada:

1. Variable de entorno en entornos de desarrollo controlados.
2. Fichero de secreto con permisos restrictivos.
3. Integración futura con secret manager.

No recomendado:

- claves literales en `config.toml`;
- claves en repositorio;
- claves en argumentos CLI visibles en histórico o lista de procesos.

## 4. Validación

La clave debe:

- tener longitud adecuada para el algoritmo;
- estar codificada de forma inequívoca, por ejemplo base64;
- validarse al arranque si el cifrado está habilitado;
- no aparecer en mensajes de error.

Para MAVLink signing de laboratorio, la codificación aceptada es hexadecimal:
32 bytes como 64 caracteres ASCII. El archivo local debe estar fuera de
del worktree Git actual del gateway, no ser symlink y, en Unix, pertenecer al
usuario efectivo del gateway con permisos sin acceso de grupo ni mundo. Este
backend local no sustituye un KMS/HSM futuro para operación de flota.

## 5. Nonces

ChaCha20-Poly1305 exige que no se reutilice nonce con la misma clave.

Opciones:

- contador monotónico por sesión con identificador de sesión aleatorio;
- nonce aleatorio con análisis de colisiones;
- derivación por dirección de flujo y contador.

La primera implementación debe documentar y testear:

- unicidad por flujo;
- comportamiento tras reinicio;
- comportamiento con concurrencia;
- límite de mensajes por clave.

## 6. Rotación

La rotación automática queda fuera de la primera versión, pero debe dejarse preparada:

- identificador de clave no secreto;
- métrica de mensajes cifrados por clave;
- error claro ante claves desincronizadas;
- procedimiento manual de reemplazo.

La rotación manual de laboratorio para signing requiere parada controlada,
generación de nueva clave fuera del repositorio, actualización explícita de
`signing.key_path` o reemplazo controlado del archivo, reinicio del emisor
firmante y reinicio del gateway. No se admite fail-open automático.

## 7. Logs y Errores

Permitido:

- algoritmo;
- estado habilitado/deshabilitado;
- fuente de clave como tipo, no valor;
- key id no secreto si existe.

Prohibido:

- clave;
- material derivado;
- dumps de estructuras criptográficas;
- payload descifrado en logs normales.

## 8. Riesgos

- nonce repetido;
- clave compartida por demasiados entornos;
- clave filtrada por logs;
- falsa seguridad si la GCS no descifra;
- pérdida de disponibilidad por desincronización de claves.
