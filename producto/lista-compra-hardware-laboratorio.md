# Lista de Compra: Laboratorio de Validación Hardware (V1.0)

Esta lista detalla los componentes necesarios para trasladar el **RustShield Gateway** de la simulación (SITL) a un entorno de hardware real (HITL/Prototipado).

## 1. Unidades de Procesamiento (Gateways)

*   **1x Raspberry Pi 4 Model B (2GB o 4GB RAM):**
    *   *Uso:* Plataforma de desarrollo principal y gateway para estaciones de tierra (GCS).
    *   *Razón:* Estándar industrial, excelente soporte de Rust y múltiples puertos serie/USB.
*   **1x Raspberry Pi Zero 2 W:**
    *   *Uso:* Validación de bajo consumo y tamaño reducido.
    *   *Razón:* Es el factor de forma ideal para ir embarcado en drones de tamaño pequeño/medio. Si el software es eficiente aquí, el producto es viable para OEMs.

## 2. Almacenamiento y Energía

*   **2x Tarjetas MicroSD (16GB o 32GB) Clase 10 / U3:**
    *   *Recomendación:* SanDisk Extreme o similar (necesaria alta velocidad para evitar cuellos de botella en los logs).
*   **2x Fuentes de Alimentación Oficiales (USB-C para RPi 4, Micro-USB para Zero 2):**
    *   *Importante:* Usar fuentes oficiales para evitar caídas de tensión que puedan corromper el sistema de archivos durante los tests.

## 3. Conectividad y Telemetría

*   **2x Adaptadores USB a TTL Serial (FTDI / CP2102):**
    *   *Uso:* Crucial para probar la comunicación `transport.rs` vía puerto serie entre las Raspberries o hacia el PC.
*   **1x Cable Ethernet (Cat 5e o 6):**
    *   *Uso:* Conexión estable para la GCS durante los primeros tests de red.

## 4. Hardware de Vuelo (Opcional - Fase Avanzada)

Si se desea validar la transparencia total con un autopiloto físico:

*   **Autopiloto (Pixhawk 6C o Cube Orange+):** 
    *   *Coste:* **250€ - 450€** (dependiendo del modelo y si incluye GPS).
*   **Datalink (Set de Telemetría SiK Radio 433/915MHz):**
    *   *Coste:* **50€ - 80€**.
    *   *Uso:* Probar el Shield en un enlace de radio real con pérdidas de paquetes y latencias variables.

## 5. Herramientas de Medición (Opcional)

*   **Analizador Lógico Barato (tipo Saleae clone 24MHz/8ch):**
    *   *Coste:* **15€ - 25€**.
    *   *Uso:* Para medir latencias físicas reales en los pines GPIO (UART) y confirmar que nuestro benchmark interno es honesto.

---
**Nota de Presupuesto:**
*   **Kit Básico (Secciones 1, 2 y 3):** Aprox. **120€ - 150€**. (Suficiente para el 90% de la validación).
*   **Laboratorio Completo (Todos los puntos):** Aprox. **450€ - 700€**.
