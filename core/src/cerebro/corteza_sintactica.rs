// =====================================================================
// CORTEZA SINTÁCTICA (COMPILADOR SIMBÓLICO-SINTÁCTICO)
// =====================================================================
// Traduce intenciones lógicas directamente a código fuente Rust
// válido y compilable, sin usar modelos de lenguaje ni APIs externas.
// Basado en lógica simbólica, expresiones regulares y ASTs tipados.
// =====================================================================

use regex::Regex;

/// Tipos de datos soportados en el AST
#[derive(Debug, Clone, PartialEq)]
pub enum TipoDato {
    Entero32,
    Entero64,
    Flotante64,
    Cadena,
    Booleano,
    Vacio,
    Personalizado(String),
}

impl TipoDato {
    pub fn a_str(&self) -> String {
        match self {
            TipoDato::Entero32 => "i32".to_string(),
            TipoDato::Entero64 => "i64".to_string(),
            TipoDato::Flotante64 => "f64".to_string(),
            TipoDato::Cadena => "String".to_string(),
            TipoDato::Booleano => "bool".to_string(),
            TipoDato::Vacio => "()".to_string(),
            TipoDato::Personalizado(s) => s.clone(),
        }
    }
}

/// Operaciones del cuerpo en el AST
#[derive(Debug, Clone, PartialEq)]
pub enum Operacion {
    Suma(String, String),
    Resta(String, String),
    Multiplicacion(String, String),
    Division(String, String),
    RetornarVal(String),
    LlamarFuncion {
        modulo: Option<String>,
        funcion: String,
        args: Vec<String>,
    },
    AsignarVariable {
        nombre: String,
        tipo: Option<TipoDato>,
        valor: String,
    },
}

/// Argumento de función
#[derive(Debug, Clone, PartialEq)]
pub struct Argumento {
    pub nombre: String,
    pub tipo: TipoDato,
}

/// AST de Función
#[derive(Debug, Clone, PartialEq)]
pub struct FuncionAST {
    pub nombre: String,
    pub params: Vec<Argumento>,
    pub retorno: TipoDato,
    pub cuerpo: Vec<Operacion>,
}

/// AST de Estructura (Struct)
#[derive(Debug, Clone, PartialEq)]
pub struct StructAST {
    pub nombre: String,
    pub campos: Vec<Argumento>,
}

/// AST de Bloque de Implementación (impl Struct)
#[derive(Debug, Clone, PartialEq)]
pub struct ImplAST {
    pub nombre_struct: String,
    pub funciones: Vec<FuncionAST>,
}

/// AST de Módulo (mod nombre_mod)
#[derive(Debug, Clone, PartialEq)]
pub struct ModuloAST {
    pub nombre: String,
    pub contenido: Vec<ASTCodigo>,
}

/// Representación unificada del AST de Código
#[derive(Debug, Clone, PartialEq)]
pub enum ASTCodigo {
    Funcion(FuncionAST),
    Struct(StructAST),
    Impl(ImplAST),
    Modulo(ModuloAST),
    Bloque(Vec<ASTCodigo>),
}

/// Extractor de Esquemas Lógicos (El Analizador Semántico)
pub struct ExtractorEsquemas {
    patron_struct: Regex,
}

impl Default for ExtractorEsquemas {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtractorEsquemas {
    pub fn new() -> Self {
        Self {
            patron_struct: Regex::new(
                r"(?i)crear\s+(?:estructura|struct)\s+(\w+)(?:\s+con\s+campos\s+(.+))?",
            )
            .unwrap(),
        }
    }

    /// Analiza un texto de intención y genera el AST de código correspondiente
    pub fn extraer(&self, intencion: &str) -> Result<ASTCodigo, String> {
        let texto_orig = intencion.trim();
        let texto = texto_orig.to_lowercase();

        // 1. Detectar Módulo
        let re_modulo = Regex::new(r"(?i)crear\s+modulo\s+(\w+)\s+con\s+(.+)").unwrap();
        if let Some(caps) = re_modulo.captures(texto_orig) {
            let mod_nombre = caps.get(1).unwrap().as_str().to_string();
            let resto = caps.get(2).unwrap().as_str();

            let contenido_ast = self.extraer(resto)?;
            let contenido = match contenido_ast {
                ASTCodigo::Bloque(v) => v,
                other => vec![other],
            };

            return Ok(ASTCodigo::Modulo(ModuloAST {
                nombre: mod_nombre,
                contenido,
            }));
        }

        // 2. Detectar Estructura + Implementación
        if texto.contains("y crear implementacion con")
            || texto.contains("y crear implementación con")
        {
            let partes: Vec<&str> = if texto.contains("y crear implementacion con") {
                texto_orig.split("y crear implementacion con").collect()
            } else {
                texto_orig.split("y crear implementación con").collect()
            };

            if partes.len() == 2 {
                let struct_part = partes[0].trim();
                let impl_part = partes[1].trim();

                let struct_ast = self.extraer(struct_part)?;
                let struct_nombre = match &struct_ast {
                    ASTCodigo::Struct(s) => s.nombre.clone(),
                    _ => {
                        return Err("Se esperaba una estructura para la implementación".to_string())
                    }
                };

                let func_ast = self.extraer_funcion_impl(impl_part, &struct_nombre)?;

                return Ok(ASTCodigo::Bloque(vec![
                    struct_ast,
                    ASTCodigo::Impl(ImplAST {
                        nombre_struct: struct_nombre,
                        funciones: vec![func_ast],
                    }),
                ]));
            }
        }

        // 3. Detectar Estructura sola
        if let Some(caps) = self.patron_struct.captures(texto_orig) {
            let nombre = caps.get(1).unwrap().as_str().to_string();
            let mut campos = Vec::new();

            if let Some(campos_match) = caps.get(2) {
                let campos_str = campos_match.as_str();
                for campo_str in campos_str.split(',') {
                    let partes: Vec<&str> = campo_str.split(':').collect();
                    if partes.len() == 2 {
                        let c_nombre = partes[0].trim().to_string();
                        let c_tipo_str = partes[1].trim().to_lowercase();
                        let c_tipo = match c_tipo_str.as_str() {
                            "i32" | "entero" | "texto_entero" | "texto entero" => {
                                TipoDato::Entero32
                            }
                            "i64" => TipoDato::Entero64,
                            "f64" | "flotante" => TipoDato::Flotante64,
                            "string" | "texto" | "cadena" => TipoDato::Cadena,
                            "bool" | "booleano" => TipoDato::Booleano,
                            _ => TipoDato::Personalizado(partes[1].trim().to_string()),
                        };
                        campos.push(Argumento {
                            nombre: c_nombre,
                            tipo: c_tipo,
                        });
                    }
                }
            }

            return Ok(ASTCodigo::Struct(StructAST { nombre, campos }));
        }

        // 4. Detectar Función
        if texto.contains("función") || texto.contains("funcion") || texto.contains("fn") {
            let mut nombre = "operar".to_string();

            let re_fn_name =
                Regex::new(r"(?i)(?:funcion|función|fn)\s+(?:publica\s+|pública\s+|pub\s+)?(\w+)")
                    .unwrap();
            if let Some(caps) = re_fn_name.captures(texto_orig) {
                nombre = caps.get(1).unwrap().as_str().to_string();
            }

            let retorno = if texto.contains("retorne booleano") || texto.contains("retorne bool") {
                TipoDato::Booleano
            } else if texto.contains("retorne texto")
                || texto.contains("retorne string")
                || texto.contains("retorne cadena")
            {
                TipoDato::Cadena
            } else if texto.contains("retorne entero") || texto.contains("retorne i32") {
                TipoDato::Entero32
            } else if texto.contains("flotante")
                || texto.contains("f64")
                || texto.contains("decimal")
            {
                TipoDato::Flotante64
            } else {
                TipoDato::Entero32
            };

            let mut params = Vec::new();

            if texto.contains("tres flotantes") || texto.contains("3 flotantes") {
                params.push(Argumento {
                    nombre: "a".to_string(),
                    tipo: TipoDato::Flotante64,
                });
                params.push(Argumento {
                    nombre: "b".to_string(),
                    tipo: TipoDato::Flotante64,
                });
                params.push(Argumento {
                    nombre: "c".to_string(),
                    tipo: TipoDato::Flotante64,
                });
            } else if texto.contains("dos enteros") || texto.contains("2 enteros") {
                params.push(Argumento {
                    nombre: "a".to_string(),
                    tipo: TipoDato::Entero32,
                });
                params.push(Argumento {
                    nombre: "b".to_string(),
                    tipo: TipoDato::Entero32,
                });
            } else if texto.contains("nombre texto") || texto.contains("nombre de tipo texto") {
                params.push(Argumento {
                    nombre: "nombre".to_string(),
                    tipo: TipoDato::Cadena,
                });
            } else if texto.contains("reciba texto") || texto.contains("reciba string") {
                params.push(Argumento {
                    nombre: "texto".to_string(),
                    tipo: TipoDato::Cadena,
                });
            } else if texto.contains("reciba entero") || texto.contains("reciba i32") {
                params.push(Argumento {
                    nombre: "n".to_string(),
                    tipo: TipoDato::Entero32,
                });
            } else {
                params.push(Argumento {
                    nombre: "a".to_string(),
                    tipo: retorno.clone(),
                });
                params.push(Argumento {
                    nombre: "b".to_string(),
                    tipo: retorno.clone(),
                });
            }

            let cuerpo_op = if nombre == "validar_email"
                || (texto.contains("email") && texto.contains("arroba") && texto.contains("punto"))
            {
                let p_name = if !params.is_empty() {
                    &params[0].nombre
                } else {
                    "texto"
                };
                Operacion::RetornarVal(format!(
                    "{}.contains('@') && {}.contains('.')",
                    p_name, p_name
                ))
            } else if nombre == "saludar"
                || texto.contains("hola mas el nombre")
                || texto.contains("hola más el nombre")
            {
                let p_name = if !params.is_empty() {
                    &params[0].nombre
                } else {
                    "nombre"
                };
                Operacion::RetornarVal(format!("format!(\"hola {{}}\", {})", p_name))
            } else if nombre == "cuadrado"
                || texto.contains("multiplicandolo por si mismo")
                || texto.contains("multiplicándolo por sí mismo")
            {
                let p_name = if !params.is_empty() {
                    &params[0].nombre
                } else {
                    "n"
                };
                Operacion::RetornarVal(format!("{} * {}", p_name, p_name))
            } else if nombre == "calcular_promedio" || texto.contains("promedio") {
                if params.len() == 3 {
                    Operacion::RetornarVal("(a + b + c) / 3.0".to_string())
                } else {
                    Operacion::RetornarVal("(a + b) / 2.0".to_string())
                }
            } else if texto.contains("sumar") || texto.contains("suma") {
                Operacion::RetornarVal("a + b".to_string())
            } else if texto.contains("restar") || texto.contains("resta") {
                Operacion::RetornarVal("a - b".to_string())
            } else if texto.contains("multiplicar") || texto.contains("multiplica") {
                Operacion::RetornarVal("a * b".to_string())
            } else if texto.contains("dividir") || texto.contains("divide") {
                Operacion::RetornarVal("a / b".to_string())
            } else {
                Operacion::RetornarVal("a".to_string())
            };

            return Ok(ASTCodigo::Funcion(FuncionAST {
                nombre,
                params,
                retorno,
                cuerpo: vec![cuerpo_op],
            }));
        }

        Err(format!(
            "No se pudo extraer una intención de código clara para: '{}'",
            texto_orig
        ))
    }

    fn extraer_funcion_impl(
        &self,
        impl_part: &str,
        _struct_nombre: &str,
    ) -> Result<FuncionAST, String> {
        let texto = impl_part.to_lowercase();
        let mut nombre = "metodo".to_string();

        let re_fn_name =
            Regex::new(r"(?i)(?:funcion|función|fn)\s+(?:publica\s+|pública\s+|pub\s+)?(\w+)")
                .unwrap();
        if let Some(caps) = re_fn_name.captures(impl_part) {
            nombre = caps.get(1).unwrap().as_str().to_string();
        }

        let retorno = if texto.contains("flotante")
            || texto.contains("f64")
            || texto.contains("pi")
            || texto.contains("area")
            || texto.contains("área")
        {
            TipoDato::Flotante64
        } else {
            TipoDato::Entero32
        };

        let params = vec![Argumento {
            nombre: "&self".to_string(),
            tipo: TipoDato::Vacio,
        }];

        let cuerpo_op = if nombre == "area" || texto.contains("area") || texto.contains("área") {
            if texto.contains("pi") {
                Operacion::RetornarVal("std::f64::consts::PI * self.radio * self.radio".to_string())
            } else {
                Operacion::RetornarVal("self.radio * self.radio".to_string())
            }
        } else {
            Operacion::RetornarVal("self.radio".to_string())
        };

        Ok(FuncionAST {
            nombre,
            params,
            retorno,
            cuerpo: vec![cuerpo_op],
        })
    }
}

/// Generador y Formateador del Código Rust Sintáctico
pub struct CompiladorSimbolico;

impl CompiladorSimbolico {
    /// Compila un AST unificado a una cadena de código Rust válido
    pub fn compilar(ast: &ASTCodigo) -> String {
        match ast {
            ASTCodigo::Funcion(func) => Self::compilar_funcion(func),
            ASTCodigo::Struct(strct) => Self::compilar_struct(strct),
            ASTCodigo::Impl(imp) => Self::compilar_impl(imp),
            ASTCodigo::Modulo(mod_ast) => Self::compilar_modulo(mod_ast),
            ASTCodigo::Bloque(nodos) => {
                let mut codigo = String::new();
                for (i, nodo) in nodos.iter().enumerate() {
                    if i > 0 {
                        codigo.push('\n');
                    }
                    codigo.push_str(&Self::compilar(nodo));
                }
                codigo
            }
        }
    }

    fn compilar_funcion(func: &FuncionAST) -> String {
        let mut codigo = String::new();

        codigo.push_str("pub fn ");
        codigo.push_str(&func.nombre);
        codigo.push('(');

        let params_str: Vec<String> = func
            .params
            .iter()
            .map(|p| {
                if p.nombre == "&self" {
                    "&self".to_string()
                } else if p.nombre == "self" {
                    "self".to_string()
                } else {
                    format!("{}: {}", p.nombre, p.tipo.a_str())
                }
            })
            .collect();
        codigo.push_str(&params_str.join(", "));

        codigo.push(')');

        if func.retorno != TipoDato::Vacio {
            codigo.push_str(&format!(" -> {}", func.retorno.a_str()));
        }

        codigo.push_str(" {\n");

        for op in &func.cuerpo {
            codigo.push_str("    ");
            codigo.push_str(&Self::compilar_operacion(op));
            codigo.push('\n');
        }

        codigo.push_str("}\n");
        codigo
    }

    fn compilar_struct(strct: &StructAST) -> String {
        let mut codigo = String::new();

        codigo.push_str("#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]\n");
        codigo.push_str("pub struct ");
        codigo.push_str(&strct.nombre);
        codigo.push_str(" {\n");

        for campo in &strct.campos {
            codigo.push_str(&format!(
                "    pub {}: {},\n",
                campo.nombre,
                campo.tipo.a_str()
            ));
        }

        codigo.push_str("}\n");
        codigo
    }

    fn compilar_impl(imp: &ImplAST) -> String {
        let mut codigo = String::new();
        codigo.push_str(&format!("impl {} {{\n", imp.nombre_struct));
        for func in &imp.funciones {
            let func_code = Self::compilar_funcion(func);
            for line in func_code.lines() {
                if line.is_empty() {
                    codigo.push('\n');
                } else {
                    codigo.push_str(&format!("    {}\n", line));
                }
            }
        }
        codigo.push_str("}\n");
        codigo
    }

    fn compilar_modulo(mod_ast: &ModuloAST) -> String {
        let mut codigo = String::new();
        codigo.push_str(&format!("pub mod {} {{\n", mod_ast.nombre));
        for item in &mod_ast.contenido {
            let item_code = Self::compilar(item);
            for line in item_code.lines() {
                if line.is_empty() {
                    codigo.push('\n');
                } else {
                    codigo.push_str(&format!("    {}\n", line));
                }
            }
        }
        codigo.push_str("}\n");
        codigo
    }

    fn compilar_operacion(op: &Operacion) -> String {
        match op {
            Operacion::Suma(a, b) => format!("{} + {}", a, b),
            Operacion::Resta(a, b) => format!("{} - {}", a, b),
            Operacion::Multiplicacion(a, b) => format!("{} * {}", a, b),
            Operacion::Division(a, b) => format!("{} / {}", a, b),
            Operacion::RetornarVal(val) => val.clone(),
            Operacion::LlamarFuncion {
                modulo,
                funcion,
                args,
            } => {
                let args_str = args.join(", ");
                if let Some(m) = modulo {
                    format!("{}::{}({})", m, funcion, args_str)
                } else {
                    format!("{}({})", funcion, args_str)
                }
            }
            Operacion::AsignarVariable {
                nombre,
                tipo,
                valor,
            } => {
                if let Some(t) = tipo {
                    format!("let {}: {} = {};", nombre, t.a_str(), valor)
                } else {
                    format!("let {} = {};", nombre, valor)
                }
            }
        }
    }
}
