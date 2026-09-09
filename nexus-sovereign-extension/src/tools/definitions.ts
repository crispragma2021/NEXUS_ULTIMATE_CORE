// ============================================================================
// 🔧 NEXUS TOOL DEFINITIONS
// ============================================================================
// Define herramientas como OpenAI-compatible function calling
// para que OpenRouter/Gemini pueda invocarlas desde el LLM.

export interface ToolParameter {
  type: string;
  description: string;
  enum?: string[];
  items?: { type: string };
  default?: any;
}

export interface ToolDefinition {
  name: string;
  description: string;
  parameters: {
    type: 'object';
    properties: Record<string, ToolParameter>;
    required: string[];
  };
}

/**
 * Catálogo de herramientas que NEXUS puede invocar.
 * Clonadas y mejoradas de las tools nativas de Roo Code.
 */
export const TOOL_DEFINITIONS: ToolDefinition[] = [
  {
    name: 'execute_command',
    description: 'Ejecuta un comando shell en el sistema. Devuelve stdout/stderr. Timeout por defecto: 30s.',
    parameters: {
      type: 'object',
      properties: {
        command: {
          type: 'string',
          description: 'Comando shell a ejecutar (ej: "ls -la", "cargo build", "git status")'
        },
        cwd: {
          type: 'string',
          description: 'Directorio de trabajo (opcional, default: workspace root)'
        },
        timeout: {
          type: 'number',
          description: 'Timeout en segundos (opcional, default: 30, max: 120)'
        }
      },
      required: ['command']
    }
  },
  {
    name: 'read_file',
    description: 'Lee el contenido de un archivo. Devuelve el texto con líneas numeradas.',
    parameters: {
      type: 'object',
      properties: {
        path: {
          type: 'string',
          description: 'Ruta del archivo (relativa al workspace o absoluta)'
        },
        offset: {
          type: 'number',
          description: 'Línea inicial (1-based, opcional)'
        },
        limit: {
          type: 'number',
          description: 'Máximo de líneas a leer (opcional, default: 2000)'
        }
      },
      required: ['path']
    }
  },
  {
    name: 'write_to_file',
    description: 'ESCRIBE o SOBREESCRIBE un archivo completo con contenido nuevo. Crea directorios si no existen. USAR CON CUIDADO - sobreescribe completamente.',
    parameters: {
      type: 'object',
      properties: {
        path: {
          type: 'string',
          description: 'Ruta del archivo a escribir'
        },
        content: {
          type: 'string',
          description: 'Contenido COMPLETO del archivo (no truncar, incluir TODO el contenido)'
        }
      },
      required: ['path', 'content']
    }
  },
  {
    name: 'apply_diff',
    description: 'APLICA cambios quirúrgicos a un archivo existente usando bloques SEARCH/REPLACE. MÁS SEGURO que write_to_file para modificaciones parciales.',
    parameters: {
      type: 'object',
      properties: {
        path: {
          type: 'string',
          description: 'Ruta del archivo a modificar'
        },
        diff: {
          type: 'string',
          description: 'Bloques SEARCH/REPLACE. Formato:\n<<<<<<< SEARCH\n:código_exacto_a_buscar\n=======\n:código_nuevo\n>>>>>>> REPLACE'
        }
      },
      required: ['path', 'diff']
    }
  },
  {
    name: 'search_files',
    description: 'Busca archivos por expresión regular. Similar a grep -r. Devuelve contexto alrededor de cada match.',
    parameters: {
      type: 'object',
      properties: {
        path: {
          type: 'string',
          description: 'Directorio raíz para la búsqueda'
        },
        regex: {
          type: 'string',
          description: 'Patrón de expresión regular a buscar (Rust regex syntax)'
        },
        file_pattern: {
          type: 'string',
          description: 'Glob pattern para filtrar archivos (ej: "*.ts", "*.rs", "*.md")'
        }
      },
      required: ['path', 'regex']
    }
  },
  {
    name: 'list_files',
    description: 'Lista archivos y directorios en una ruta. Opcionalmente recursivo.',
    parameters: {
      type: 'object',
      properties: {
        path: {
          type: 'string',
          description: 'Ruta del directorio a listar'
        },
        recursive: {
          type: 'boolean',
          description: 'Listar recursivamente (opcional, default: false)'
        }
      },
      required: ['path']
    }
  },
  {
    name: 'ask_question',
    description: 'PREGUNTA al usuario. Usa cuando necesites información, confirmación o decisión del humano.',
    parameters: {
      type: 'object',
      properties: {
        question: {
          type: 'string',
          description: 'Pregunta clara para el usuario'
        },
        options: {
          type: 'string',
          description: 'Opciones sugeridas separadas por | (ej: "Sí|No|Cancelar")'
        }
      },
      required: ['question']
    }
  },
  {
    name: 'attempt_completion',
    description: 'FINALIZA la tarea actual con un mensaje de resultado. Úsala cuando hayas completado exitosamente el objetivo.',
    parameters: {
      type: 'object',
      properties: {
        result: {
          type: 'string',
          description: 'Mensaje final describiendo lo completado'
        }
      },
      required: ['result']
    }
  },
  {
    name: 'read_terminal_output',
    description: 'Lee output completo de un comando previo truncado. Útil cuando execute_command devuelve output truncado.',
    parameters: {
      type: 'object',
      properties: {
        artifact_id: {
          type: 'string',
          description: 'ID del artifact (ej: "cmd-1706119234567.txt")'
        },
        search: {
          type: 'string',
          description: 'Patrón opcional para filtrar líneas (grep)' 
        },
        offset: {
          type: 'number',
          description: 'Byte offset para paginación'
        },
        limit: {
          type: 'number',
          description: 'Máximo de bytes a retornar'
        }
      },
      required: ['artifact_id']
    }
  },
  {
    name: 'database_query',
    description: 'Ejecuta consultas SQL en la base de datos NEXUS (nexus_memoria.db SQLite). Solo SELECT.',
    parameters: {
      type: 'object',
      properties: {
        query: {
          type: 'string',
          description: 'Consulta SQL SELECT a ejecutar'
        }
      },
      required: ['query']
    }
  },
  {
    name: 'memory_search',
    description: 'Busca en la memoria semántica de NEXUS (FTS5 y vectores). Recupera experiencias, decisiones, contexto.',
    parameters: {
      type: 'object',
      properties: {
        query: {
          type: 'string',
          description: 'Texto de búsqueda semántica'
        },
        limit: {
          type: 'number',
          description: 'Máximo de resultados (default: 5)'
        }
      },
      required: ['query']
    }
  }
];
