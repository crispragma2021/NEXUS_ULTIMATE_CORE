// ============================================================================
// 🧩 RAG SHADCN/UI — CATÁLOGO DE COMPONENTES
// ============================================================================
// Índice estático de componentes shadcn/ui que Gemini usa como contexto para
// planificar y generar. Es un catálogo finito y conocido (no requiere
// embeddings ni búsqueda semántica): se carga una vez y se consulta por
// nombre/categoría.
//
// Cada componente registra:
//   - nombre, categoría, descripción
//   - dependencias (@radix-ui/*, utilidades)
//   - variantes principales
//   - fragmento de ejemplo de uso
//   - primitivas base (componentes shadcn que requiere)
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Un componente del catálogo shadcn/ui.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponenteShadcn {
    pub nombre: String,
    pub categoria: String,
    pub descripcion: String,
    #[serde(default)]
    pub dependencias: Vec<String>,
    #[serde(default)]
    pub primitivas: Vec<String>,
    #[serde(default)]
    pub variantes: Vec<String>,
    pub ejemplo: String,
}

/// Índice de componentes shadcn/ui.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CatalogoShadcn {
    pub componentes: HashMap<String, ComponenteShadcn>,
}

impl CatalogoShadcn {
    /// Catálogo estático de componentes shadcn/ui. Fuente de verdad embebida.
    pub fn estandar() -> Self {
        let mut c = HashMap::new();
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "button".into(),
                categoria: "primitivas".into(),
                descripcion: "Botón con variantes de estilo y tamaño via class-variance-authority.".into(),
                dependencias: vec!["class-variance-authority".into(), "clsx".into(), "tailwind-merge".into(), "lucide-react".into()],
                primitivas: vec!["@radix-ui/react-slot".into()],
                variantes: vec!["default".into(), "destructive".into(), "outline".into(), "secondary".into(), "ghost".into(), "link".into()],
                ejemplo: "<Button variant=\"outline\">Outline</Button>".into(),
            },
        );
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "card".into(),
                categoria: "layout".into(),
                descripcion: "Contenedor con header, contenido y footer para agrupar información.".into(),
                dependencias: vec!["clsx".into(), "tailwind-merge".into()],
                primitivas: vec![],
                variantes: vec![],
                ejemplo: "<Card><CardHeader><CardTitle>Título</CardTitle></CardHeader><CardContent>...</CardContent></Card>".into(),
            },
        );
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "input".into(),
                categoria: "form".into(),
                descripcion: "Campo de texto de entrada de datos.".into(),
                dependencias: vec!["clsx".into(), "tailwind-merge".into()],
                primitivas: vec![],
                variantes: vec![],
                ejemplo: "<Input placeholder=\"Email\" />".into(),
            },
        );
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "label".into(),
                categoria: "form".into(),
                descripcion: "Etiqueta accesible para campos de formulario.".into(),
                dependencias: vec!["clsx".into()],
                primitivas: vec!["@radix-ui/react-label".into()],
                variantes: vec![],
                ejemplo: "<Label htmlFor=\"email\">Email</Label>".into(),
            },
        );
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "select".into(),
                categoria: "form".into(),
                descripcion: "Selector desplegable accesible.".into(),
                dependencias: vec!["clsx".into(), "tailwind-merge".into()],
                primitivas: vec!["@radix-ui/react-select".into()],
                variantes: vec![],
                ejemplo: "<Select><SelectTrigger><SelectValue /></SelectTrigger><SelectContent>...</SelectContent></Select>".into(),
            },
        );
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "dialog".into(),
                categoria: "overlay".into(),
                descripcion: "Diálogo modal accesible con superposición.".into(),
                dependencias: vec!["clsx".into(), "tailwind-merge".into()],
                primitivas: vec!["@radix-ui/react-dialog".into()],
                variantes: vec![],
                ejemplo: "<Dialog><DialogTrigger>Open</DialogTrigger><DialogContent>...</DialogContent></Dialog>".into(),
            },
        );
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "dropdown-menu".into(),
                categoria: "overlay".into(),
                descripcion: "Menú desplegable accesible con items, separadores y shortcuts.".into(),
                dependencias: vec!["clsx".into(), "tailwind-merge".into()],
                primitivas: vec!["@radix-ui/react-dropdown-menu".into()],
                variantes: vec![],
                ejemplo: "<DropdownMenu><DropdownMenuTrigger>...</DropdownMenuTrigger><DropdownMenuContent>...</DropdownMenuContent></DropdownMenu>".into(),
            },
        );
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "tabs".into(),
                categoria: "navigation".into(),
                descripcion: "Pestañas para alternar entre vistas de contenido.".into(),
                dependencias: vec!["clsx".into(), "tailwind-merge".into()],
                primitivas: vec!["@radix-ui/react-tabs".into()],
                variantes: vec![],
                ejemplo: "<Tabs><TabsList><TabsTrigger value=\"a\">A</TabsTrigger></TabsList><TabsContent value=\"a\">...</TabsContent></Tabs>".into(),
            },
        );
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "table".into(),
                categoria: "data-display".into(),
                descripcion: "Tabla de datos con header, body, fila y celda.".into(),
                dependencias: vec!["clsx".into(), "tailwind-merge".into()],
                primitivas: vec![],
                variantes: vec![],
                ejemplo: "<Table><TableHeader><TableRow><TableHead>Nombre</TableHead></TableRow></TableHeader><TableBody>...</TableBody></Table>".into(),
            },
        );
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "badge".into(),
                categoria: "data-display".into(),
                descripcion: "Etiqueta pequeña para estados, contadores o clasificación.".into(),
                dependencias: vec!["class-variance-authority".into(), "clsx".into(), "tailwind-merge".into()],
                primitivas: vec![],
                variantes: vec!["default".into(), "secondary".into(), "destructive".into(), "outline".into()],
                ejemplo: "<Badge variant=\"secondary\">Nuevo</Badge>".into(),
            },
        );
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "avatar".into(),
                categoria: "data-display".into(),
                descripcion: "Avatar de usuario con fallback de iniciales.".into(),
                dependencias: vec!["clsx".into()],
                primitivas: vec!["@radix-ui/react-avatar".into()],
                variantes: vec![],
                ejemplo: "<Avatar><AvatarImage src=\"...\" /><AvatarFallback>NX</AvatarFallback></Avatar>".into(),
            },
        );
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "progress".into(),
                categoria: "feedback".into(),
                descripcion: "Barra de progreso accesible.".into(),
                dependencias: vec!["clsx".into()],
                primitivas: vec!["@radix-ui/react-progress".into()],
                variantes: vec![],
                ejemplo: "<Progress value={33} />".into(),
            },
        );
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "separator".into(),
                categoria: "layout".into(),
                descripcion: "Separador visual horizontal o vertical.".into(),
                dependencias: vec!["clsx".into()],
                primitivas: vec!["@radix-ui/react-separator".into()],
                variantes: vec![],
                ejemplo: "<Separator />".into(),
            },
        );
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "switch".into(),
                categoria: "form".into(),
                descripcion: "Interruptor de encendido/apagado accesible.".into(),
                dependencias: vec!["clsx".into()],
                primitivas: vec!["@radix-ui/react-switch".into()],
                variantes: vec![],
                ejemplo: "<Switch checked={on} onCheckedChange={setOn} />".into(),
            },
        );
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "checkbox".into(),
                categoria: "form".into(),
                descripcion: "Casilla de verificación accesible.".into(),
                dependencias: vec!["clsx".into()],
                primitivas: vec!["@radix-ui/react-checkbox".into()],
                variantes: vec![],
                ejemplo: "<Checkbox checked={x} onCheckedChange={setX} />".into(),
            },
        );
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "tooltip".into(),
                categoria: "overlay".into(),
                descripcion: "Tooltip accesible que aparece al hacer hover.".into(),
                dependencias: vec!["clsx".into()],
                primitivas: vec!["@radix-ui/react-tooltip".into()],
                variantes: vec![],
                ejemplo: "<Tooltip><TooltipTrigger>Hover</TooltipTrigger><TooltipContent>Info</TooltipContent></Tooltip>".into(),
            },
        );
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "toast".into(),
                categoria: "feedback".into(),
                descripcion: "Notificaciones toast para feedback efímero.".into(),
                dependencias: vec!["clsx".into()],
                primitivas: vec!["@radix-ui/react-toast".into()],
                variantes: vec![],
                ejemplo: "<Toast><ToastTitle>Título</ToastTitle><ToastDescription>...</ToastDescription></Toast>".into(),
            },
        );
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "alert-dialog".into(),
                categoria: "overlay".into(),
                descripcion: "Diálogo de confirmación de acciones destructivas.".into(),
                dependencias: vec!["clsx".into()],
                primitivas: vec!["@radix-ui/react-alert-dialog".into()],
                variantes: vec![],
                ejemplo: "<AlertDialog><AlertDialogTrigger>Delete</AlertDialogTrigger><AlertDialogContent>...</AlertDialogContent></AlertDialog>".into(),
            },
        );
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "scroll-area".into(),
                categoria: "layout".into(),
                descripcion: "Área con scrollbar personalizada y accesible.".into(),
                dependencias: vec!["clsx".into()],
                primitivas: vec!["@radix-ui/react-scroll-area".into()],
                variantes: vec![],
                ejemplo: "<ScrollArea className=\"h-40\">...</ScrollArea>".into(),
            },
        );
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "calendar".into(),
                categoria: "form".into(),
                descripcion: "Calendario de selección de fechas.".into(),
                dependencias: vec!["react-day-picker".into(), "date-fns".into(), "clsx".into()],
                primitivas: vec![],
                variantes: vec![],
                ejemplo: "<Calendar mode=\"single\" selected={date} onSelect={setDate} />".into(),
            },
        );
        Self::insertar(
            &mut c,
            ComponenteShadcn {
                nombre: "sonner".into(),
                categoria: "feedback".into(),
                descripcion: "Sistema de toasts moderno y ligero (sonner).".into(),
                dependencias: vec!["sonner".into()],
                primitivas: vec![],
                variantes: vec![],
                ejemplo: "<Toaster />".into(),
            },
        );

        CatalogoShadcn { componentes: c }
    }

    fn insertar(c: &mut HashMap<String, ComponenteShadcn>, comp: ComponenteShadcn) {
        c.insert(comp.nombre.clone(), comp);
    }

    /// Busca un componente por nombre (case-insensitive, admite prefijo "ui/").
    pub fn buscar(&self, nombre: &str) -> Option<&ComponenteShadcn> {
        let limpio = nombre
            .trim()
            .trim_start_matches("ui/")
            .trim_start_matches("@/components/ui/")
            .trim_start_matches("components/ui/");
        self.componentes.get(limpio).or_else(|| {
            self.componentes
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(limpio))
                .map(|(_, v)| v)
        })
    }

    /// Devuelve componentes de una categoría concreta.
    pub fn por_categoria(&self, categoria: &str) -> Vec<&ComponenteShadcn> {
        self.componentes
            .values()
            .filter(|c| c.categoria == categoria)
            .collect()
    }

    /// Devuelve los nombres de todos los componentes, ordenados.
    pub fn nombres(&self) -> Vec<String> {
        let mut v: Vec<String> = self.componentes.keys().cloned().collect();
        v.sort();
        v
    }

    /// Categorías presentes en el catálogo.
    pub fn categorias(&self) -> Vec<String> {
        let mut set: Vec<String> = self
            .componentes
            .values()
            .map(|c| c.categoria.clone())
            .collect();
        set.sort();
        set.dedup();
        set
    }

    /// Serializa el catálogo a JSON para inyectar como contexto a Gemini.
    pub fn a_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".into())
    }

    /// Número total de componentes indexados.
    pub fn len(&self) -> usize {
        self.componentes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.componentes.is_empty()
    }
}

impl Default for CatalogoShadcn {
    fn default() -> Self {
        Self::estandar()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalogo_estandar_carga() {
        let c = CatalogoShadcn::estandar();
        assert!(c.len() >= 20);
        assert!(c.buscar("button").is_some());
        assert!(c.buscar("dialog").is_some());
        assert!(c.buscar("card").is_some());
    }

    #[test]
    fn test_buscar_case_insensitive_y_prefijos() {
        let c = CatalogoShadcn::estandar();
        assert!(c.buscar("Button").is_some());
        assert!(c.buscar("ui/button").is_some());
        assert!(c.buscar("@/components/ui/button").is_some());
        assert!(c.buscar("components/ui/button").is_some());
    }

    #[test]
    fn test_buscar_inexistente() {
        let c = CatalogoShadcn::estandar();
        assert!(c.buscar("no-existe").is_none());
    }

    #[test]
    fn test_por_categoria() {
        let c = CatalogoShadcn::estandar();
        let form = c.por_categoria("form");
        assert!(!form.is_empty());
        assert!(form.iter().all(|x| x.categoria == "form"));
        assert!(form.iter().any(|x| x.nombre == "select"));
    }

    #[test]
    fn test_nombres_ordenados_y_categorias() {
        let c = CatalogoShadcn::estandar();
        let nombres = c.nombres();
        assert!(nombres.windows(2).all(|w| w[0] <= w[1]));
        let cats = c.categorias();
        assert!(cats.contains(&"overlay".to_string()));
        assert!(cats.contains(&"form".to_string()));
    }

    #[test]
    fn test_a_json_roundtrip() {
        let c = CatalogoShadcn::estandar();
        let json = c.a_json();
        let c2: CatalogoShadcn = serde_json::from_str(&json).unwrap();
        assert_eq!(c, c2);
    }

    #[test]
    fn test_dependencias_radix_consistentes() {
        let c = CatalogoShadcn::estandar();
        let dialog = c.buscar("dialog").unwrap();
        assert!(dialog.primitivas.contains(&"@radix-ui/react-dialog".to_string()));
        let button = c.buscar("button").unwrap();
        assert!(button.dependencias.contains(&"class-variance-authority".to_string()));
        assert_eq!(button.variantes.len(), 6);
    }
}
