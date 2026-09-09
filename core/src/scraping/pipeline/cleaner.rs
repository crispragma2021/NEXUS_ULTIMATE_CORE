//! Limpieza determinista HTML → Markdown (F1.2).
//!
//! 100% código puro (sin LLM). Estrategia:
//! 1. Recoge los ids de nodos que coinciden con selectores de exclusión
//!    (`script`, `style`, `nav`, `footer`, `header`, `svg`, `iframe`,
//!    `noscript`, `form` + selectores CSS extra).
//! 2. Recorre el DOM saltando (skip) los subárboles de esos nodos y convierte
//!    el resto a Markdown plano (`h1`-`h6`, `p`, `a`, `ul`/`ol`/`li`,
//!    `pre`/`code`, `blockquote`, `img` alt).
//!
//! Usa solo la API pública y estable de `scraper`: `ElementRef::id()` y
//! `document.select()`.

use scraper::{Html, Node, Selector};
use std::collections::HashSet;

/// Selectores de nodos a eliminar por completo.
const STRIP_SELECTORS: [&str; 9] = [
    "script", "style", "nav", "footer", "header", "svg", "iframe", "noscript", "form",
];

/// Limpia el HTML crudo y devuelve Markdown plano.
///
/// - Elimina los nodos definidos en `STRIP_SELECTORS`.
/// - Aplica selectores de exclusión extra si se proveen (spec §2.1).
/// - Colapsa espacios en blanco múltiples y líneas vacías redundantes.
pub fn clean(html: &str, exclude_selectors: &[String]) -> String {
    let document = Html::parse_document(html);

    // 1. Construir lista de selectores de exclusión: base + extra.
    let mut strip: Vec<Selector> =
        Vec::with_capacity(STRIP_SELECTORS.len() + exclude_selectors.len());
    for sel in STRIP_SELECTORS.iter() {
        if let Ok(s) = Selector::parse(sel) {
            strip.push(s);
        }
    }
    for sel in exclude_selectors.iter() {
        if let Ok(s) = Selector::parse(sel) {
            strip.push(s);
        }
    }

    // 2. Identificar ids de nodos a omitir.
    let mut remove_ids: HashSet<_> = HashSet::new();
    for sel in &strip {
        for el in document.select(sel) {
            remove_ids.insert(el.id());
        }
    }

    // 3. Renderizar el árbol saltando los nodos excluidos.
    let root = document.root_element();
    let mut out = String::new();
    render_element(root, &remove_ids, &mut out);
    collapse_whitespace(&mut out);
    out
}

/// Renderiza un elemento y sus descendientes a Markdown, saltando los nodos
/// excluidos (sus subárboles completos se omiten).
fn render_element(
    el: scraper::ElementRef<'_>,
    remove_ids: &HashSet<ego_tree::NodeId>,
    out: &mut String,
) {
    // Si este nodo está en la lista de exclusión, omitir todo su subárbol.
    if remove_ids.contains(&el.id()) {
        return;
    }

    let tag = el.value().name();
    // Elementos de bloque: el salto de línea se añade tras renderizar hijos.
    let is_block = matches!(
        tag,
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "li" | "blockquote" | "pre"
    );

    match tag {
        "h1" => out.push_str("# "),
        "h2" => out.push_str("## "),
        "h3" => out.push_str("### "),
        "h4" => out.push_str("#### "),
        "h5" => out.push_str("##### "),
        "h6" => out.push_str("###### "),
        "p" => out.push_str(""),
        "li" => out.push_str("- "),
        "blockquote" => out.push_str("> "),
        "pre" => out.push_str("```\n"),
        "br" => out.push('\n'),
        "hr" => out.push_str("\n---\n"),
        "img" => {
            let alt = el.value().attr("alt").unwrap_or("");
            let src = el.value().attr("src").unwrap_or("");
            if !alt.is_empty() {
                out.push_str(&format!("![{alt}]({src}) "));
            }
        }
        "a" => {
            let href = el.value().attr("href").unwrap_or("").trim();
            let text = el.text().collect::<String>();
            if !text.trim().is_empty() {
                if href.starts_with("http") {
                    out.push_str(&format!("[{}]({})", text.trim(), href));
                } else {
                    out.push_str(text.trim());
                }
            }
        }
        _ => {}
    }

    // Recorrer hijos.
    for child in el.children() {
        match child.value() {
            Node::Element(_) => {
                if let Some(child_el) = scraper::ElementRef::wrap(child) {
                    render_element(child_el, remove_ids, out);
                }
            }
            Node::Text(t) => {
                let text: &str = &t.text;
                if !text.trim().is_empty() {
                    out.push_str(text.trim());
                    out.push(' ');
                }
            }
            _ => {}
        }
    }

    // Cerrar el bloque con un salto de línea.
    if is_block {
        out.push('\n');
    }
}

/// Colapsa espacios/líneas redundantes: normaliza a bloques de texto legibles.
fn collapse_whitespace(out: &mut String) {
    // Reemplazar secuencias de espacios por uno solo.
    let mut prev_space = false;
    let mut result = String::with_capacity(out.len());
    for ch in out.chars() {
        if ch == ' ' {
            if prev_space {
                continue;
            }
            prev_space = true;
            result.push(' ');
        } else {
            prev_space = false;
            result.push(ch);
        }
    }
    // Reducir 3+ saltos de línea a 2.
    let lines: Vec<&str> = result.lines().collect();
    let mut final_out = String::new();
    let mut blank = 0;
    for line in lines {
        if line.trim().is_empty() {
            blank += 1;
            if blank <= 1 {
                final_out.push('\n');
            }
        } else {
            blank = 0;
            final_out.push_str(line.trim());
            final_out.push('\n');
        }
    }
    *out = final_out.trim().to_string();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elimina_script_style_y_nav() {
        let html = r#"
        <html><body>
            <script>alert("hack");</script>
            <style>.x{color:red}</style>
            <nav><a href="/menu">Menu</a></nav>
            <h1>Titulo</h1>
            <p>Parrafo <b>importante</b>.</p>
        </body></html>"#;
        let md = clean(html, &[]);
        assert!(!md.contains("hack"));
        assert!(!md.contains("color:red"));
        assert!(!md.contains("Menu"));
        assert!(md.contains("Titulo"));
        assert!(md.contains("importante"));
    }

    #[test]
    fn respeta_selectores_de_exclusion_extra() {
        let html = r#"<html><body><div class="ad">Publicidad</div><p>Contenido</p></body></html>"#;
        let md = clean(html, &[".ad".to_string()]);
        assert!(!md.contains("Publicidad"));
        assert!(md.contains("Contenido"));
    }

    #[test]
    fn convierte_encabezados_a_markdown() {
        let html =
            r#"<html><body><h1>A</h1><h2>B</h2><ul><li>Uno</li><li>Dos</li></ul></body></html>"#;
        let md = clean(html, &[]);
        assert!(md.contains("# A"));
        assert!(md.contains("## B"));
        assert!(md.contains("- Uno"));
        assert!(md.contains("- Dos"));
    }

    #[test]
    fn reduce_tamano_significativamente() {
        let html = format!(
            "<html><head><style>{}</style></head><body>{}</body></html>",
            "a".repeat(5000),
            "<p>".to_string() + &"b".repeat(2000) + "</p>"
        );
        let md = clean(&html, &[]);
        assert!(md.len() < 3000);
    }
}
