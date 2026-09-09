# PLAN DE REDISEÑO: AUTOPUBLICADOR SOBERANO v2.0

## 🎨 Concepto: Nexus Cyber-Financial
Transformación del panel actual (básico) a una interfaz de nivel operativo profesional, inspirada en terminales de trading (Bloomberg/Binance) pero con estética Cyberpunk.

## 🛠️ Stack Tecnológico
- **Frontend**: HTML5 + Tailwind CSS (vía CDN para portabilidad)
- **Iconografía**: Lucide Icons / FontAwesome
- **Tipografía**: Outfit (UI) + Fira Code (Data)
- **Backend**: Express.js (ya implementado en `web_panel.cjs`)

## 🔑 Activación de Modelos (Vertex AI $300)
Para usar Gemini 1.5 Pro multimodal con tus créditos:
1. Ve a: [Vertex AI Model Garden](https://console.cloud.google.com/vertex-ai/model-garden?project=project-26e94ab7-4257-4475-ade)
2. Busca "Gemini" y selecciona **Gemini 1.5 Pro**.
3. Haz clic en **Habilitar** (Enable).

---

## 📐 Estructura de Componentes (v2.0 Implementada)

### 1. Sidebar Soberano (Navegación)
- Botones de acceso rápido: Dashboard, Programador, Historial, Configuración de Sesión.
- Indicador de salud de la sesión de Facebook en tiempo real.

### 2. Panel de Métricas (Hero)
- **Total Posts**: Alcance acumulado.
- **Tasa de Éxito**: % de publicaciones sin detección.
- **Días Activo**: Tiempo de vida de la identidad actual (Gabriel).
- **Cola**: Cantidad de posts pendientes.

### 3. Composer Pro (Creación)
- Área de texto con auto-resize.
- Selector de Estilo (Informativo, Provocador, Storytelling).
- **Preview Real**: Una ventana que muestra cómo se verá el post en Facebook antes de enviarlo.

### 4. Tabla de Operaciones (Cola)
- Filas con efecto hover esmeralda.
- Badges de estado: `PENDIENTE` (Amarillo), `PUBLICANDO` (Cian Animado), `COMPLETADO` (Esmeralda), `FALLIDO` (Rojo).
- Miniatura del screenshot de la publicación exitosa.

## 💎 Design System (Tailwind Config)
```javascript
{
  colors: {
    'nexus-bg': '#020617',
    'nexus-card': '#0f172a',
    'nexus-emerald': '#10b981',
    'nexus-cyan': '#06b6d4',
    'nexus-border': '#1e293b'
  }
}
```

## 🚀 Próximos Pasos (MODO CÓDIGO)
1. Integrar Tailwind en `web_panel.cjs`.
2. Re-escribir el generador de HTML en el endpoint `/`.
3. Añadir lógica de WebSockets o Polling optimizado para estados vivos.
4. **Figma Integration**: Si el Arquitecto provee un link de archivo, extraeré los SVGs exactos.

¿Aprueba este esquema para proceder a la implementación?
