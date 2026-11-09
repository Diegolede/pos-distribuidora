# Evaluación de Frameworks GUI para POS en Rust

## Fecha: 2026-11-09
## Objetivo: Elegir framework GUI para POS de escritorio multiplataforma

---

## Criterios de Evaluación

1. **Rendimiento con 5,000+ productos** (virtualización, búsqueda rápida)
2. **Integración nativa Linux** (Wayland/X11, theming, atajos)
3. **Libertad visual** (diseño personalizado tipo Hyprland, píldoras, bordes redondeados)
4. **Reutilización real Windows/Android** (sin reescritura)
5. **Ecosistema Rust** (documentación, comunidad, estabilidad)
6. **Facilidad de desarrollo** (curva de aprendizaje, hot reload)

---

## Alternativas Evaluadas

### 1. **Tauri v2** (Web stack: HTML/CSS/JS + Rust backend)
- ✅ Excelente libertad visual (CSS completo, componentes web)
- ✅ Desarrollo rápido con frameworks JS (React/Vue/Svelte)
- ✅ Virtualización madura (react-window, react-virtualized)
- ✅ Windows y Linux soportados oficialmente
- ⚠️  Android en beta (Tauri Mobile alpha), no producción aún
- ⚠️  Tecnología web (no nativa, mayor overhead memoria)
- ⚠️  Webview dependiente del sistema (WebKitGTK en Linux)
- 📊 **Rendimiento:** Excelente con virtualización web
- 📊 **Tamaño:** ~10-15 MB (bundle + webview)

### 2. **Slint** (Toolkit declarativo nativo)
- ✅ Lenguaje `.slint` declarativo similar a QML
- ✅ Renderizado nativo con Skia/FemtoVG
- ✅ Diseño responsive con flexbox-like layout
- ✅ Linux (Wayland/X11), Windows soportados
- ⚠️  Android experimental (renderer software)
- ⚠️  Libertad visual limitada vs CSS (custom painting complejo)
- ⚠️  Virtualización manual (no hay componente ListView virtual integrado)
- ⚠️  Comunidad más pequeña
- 📊 **Rendimiento:** Bueno, pero listas grandes requieren trabajo manual
- 📊 **Tamaño:** ~5-8 MB

### 3. **egui** (Immediate mode GUI)
- ✅ Código Rust puro, sin DSL separado
- ✅ Excelente para herramientas y debug UIs
- ✅ Multiplataforma (wgpu backend)
- ⚠️  Immediate mode: redibuja todo cada frame (menos eficiente)
- ⚠️  Estilos limitados, look consistente pero no muy personalizable
- ⚠️  No adecuado para Android (touch, gestos)
- ⚠️  Virtualización manual, no trivial con 5k items
- 📊 **Rendimiento:** Aceptable hasta ~500 items visibles, 5k requiere optimización pesada
- 📊 **Aspecto:** Funcional pero no "profesional/moderno" sin trabajo extra

### 4. **Iced** (Elm-inspired, reactive)
- ✅ Arquitectura limpia (Elm pattern)
- ✅ Multiplataforma con wgpu
- ✅ Más declarativo que egui
- ⚠️  Ecosistema de widgets limitado
- ⚠️  Virtualización no incluida, implementación manual
- ⚠️  Personalización visual moderada
- ⚠️  Android no soportado oficialmente
- 📊 **Rendimiento:** Bueno con optimización manual

### 5. **GTK4-rs** (Bindings a GTK 4)
- ✅ Integración Linux perfecta (nativo GNOME)
- ✅ Widgets maduros (ListView con factory pattern)
- ✅ Theming con CSS GTK
- ✅ Virtualización integrada (GtkListView)
- ⚠️  Windows requiere runtime GTK
- ❌ Android no soportado (GTK es desktop-only)
- ⚠️  Curva de aprendizaje (GObject model)
- ⚠️  Look GTK reconocible, libertad visual moderada
- 📊 **Rendimiento:** Excelente con listas virtualizadas
- 📊 **Tamaño:** ~40-60 MB (incluyendo librerías GTK)

---

## Decisión: **Tauri v2 + React + TypeScript**

### Justificación

**Elegido:** Tauri con frontend React/TypeScript porque:

1. **Rendimiento probado:** react-window/react-virtualized manejan 10k+ items fluidamente
2. **Libertad visual total:** CSS/Tailwind permite implementar el diseño píldoras/Hyprland exactamente
3. **Desarrollo rápido:** componentes React reutilizables, hot reload, depuración con DevTools
4. **Linux funcional:** WebKitGTK sólido, integración con Wayland/X11
5. **Windows viable:** mismo código frontend, Webview2 nativo
6. **Separación núcleo/UI:** lógica Rust independiente reutilizable

### Compromiso Consciente: Android

**ANDROID NO ES COMPATIBLE EN ESTA IMPLEMENTACIÓN.**

- Tauri Mobile está en **alpha/beta** (no estable para producción)
- Requiere reescritura significativa de UI para touch/navegación móvil
- Pruebas en dispositivo Android no disponibles en este entorno

**Camino hacia Android (futuro):**
1. El núcleo Rust (`pos-core`, `pos-catalog`, etc.) es reutilizable
2. UI Android nativa separada con Jetpack Compose + JNI/FFI al core
3. O esperar madurez de Tauri Mobile + adaptar layouts responsive

### Limitaciones Reconocidas

1. **Overhead web:** ~100-150 MB RAM más que GUI nativa (aceptable en desktop 2026)
2. **Dependencia webview:** requiere WebKitGTK en Linux (normalmente preinstalado)
3. **No es "nativo" visualmente:** pero permite diseño exacto sin restricciones de toolkit
4. **Bundle mayor:** ~15-20 MB vs ~5 MB nativo (aceptable para app de negocio)

### Comparación Final

| Criterio              | Tauri | GTK4-rs | Slint | egui | Iced |
|-----------------------|-------|---------|-------|------|------|
| Rendimiento 5k items  | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ |
| Libertad visual       | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ |
| Integración Linux     | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |
| Soporte Windows       | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| Soporte Android       | ⭐⭐ (beta) | ❌ | ⭐⭐ (exp) | ⭐ | ❌ |
| Velocidad desarrollo  | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| Ecosistema/docs       | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |

---

## Stack Técnico Final

- **Backend:** Rust (Tauri commands)
- **Frontend:** React 18 + TypeScript
- **Estilos:** Tailwind CSS + tokens personalizados
- **Virtualización:** @tanstack/react-virtual (moderno, mantenido)
- **Estado:** Zustand (ligero) o React Context
- **Build:** Vite (incluido con Tauri)
- **Empaquetado Linux:** AppImage, .deb
- **Empaquetado Windows:** .msi, .exe instalador

---

## Nota sobre Tecnologías Web

**SÍ, ESTA IMPLEMENTACIÓN USA TECNOLOGÍAS WEB (HTML/CSS/JavaScript)** para la UI.

Esto no es una limitación sino una elección estratégica:
- Permite cumplir todos los requisitos de diseño visual exactamente
- Rendimiento suficiente para el caso de uso (no es un videojuego)
- Toda la lógica crítica está en Rust (DB, cálculos, importación, etc.)
- La UI web es solo la "cara" del sistema

---

**Conclusión:** Tauri v2 permite entregar un POS profesional, rápido y visualmente exacto para Linux y Windows, con el núcleo Rust preparado para una futura app Android nativa.
