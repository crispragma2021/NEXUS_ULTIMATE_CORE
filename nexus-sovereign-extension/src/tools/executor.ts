// ============================================================================
// 🔧 NEXUS TOOL EXECUTOR
// ============================================================================
// Ejecuta herramientas nativas SIN depender de MCP servers ni Roo Code.
// Cada tool se implementa con APIs nativas de Node/VS Code.

import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';
import { exec, execSync, ChildProcess } from 'child_process';
import * as readline from 'readline';

export type ToolResult = {
  success: boolean;
  output: string;
  error?: string;
};

export type ToolCallback = (
  args: Record<string, any>,
  context: vscode.ExtensionContext
) => Promise<ToolResult>;

/**
 * Registry de ejecutores de herramientas.
 * Cada tool tiene su implementación nativa aquí.
 */
export const toolExecutors: Record<string, ToolCallback> = {
  /**
   * Ejecuta un comando shell en el sistema.
   */
  execute_command: async (args, _context) => {
    const command = args.command as string;
    const cwd = (args.cwd as string) || vscode.workspace.workspaceFolders?.[0]?.uri?.fsPath || process.cwd();
    const timeout = Math.min((args.timeout as number) || 30, 120);

    if (!command || command.trim().length === 0) {
      return { success: false, output: '', error: 'Comando vacío' };
    }

    return new Promise((resolve) => {
      const child = exec(command, { cwd, timeout: timeout * 1000, maxBuffer: 10 * 1024 * 1024 }, (error, stdout, stderr) => {
        let output = '';
        if (stdout) output += stdout;
        if (stderr) output += '\n' + stderr;
        
        if (error) {
          // Timeout o error
          const errorMsg = error.killed
            ? `⚠️ Timeout (${timeout}s) - comando fue asesinado`
            : `⚠️ Error: ${error.message}`;
          resolve({ success: true, output: output || errorMsg, error: error.message });
        } else {
          resolve({ success: true, output: output || '(comando ejecutado sin output)' });
        }
      });
    });
  },

  /**
   * Lee el contenido de un archivo.
   */
  read_file: async (args) => {
    const filePath = resolvePath(args.path as string);
    try {
      if (!fs.existsSync(filePath)) {
        return { success: false, output: '', error: `Archivo no encontrado: ${filePath}` };
      }
      const content = fs.readFileSync(filePath, 'utf-8');
      const lines = content.split('\n');
      const offset = (args.offset as number) || 1;
      const limit = (args.limit as number) || 2000;
      const start = Math.max(0, offset - 1);
      const end = Math.min(lines.length, start + limit);
      const selected = lines.slice(start, end);
      const numbered = selected.map((line, i) => `${start + i + 1} | ${line}`).join('\n');
      
      let result = numbered;
      if (start > 0) result = `... (omitting ${start} lines before)\n${result}`;
      if (end < lines.length) result = `${result}\n... (${lines.length - end} more lines after)`;
      
      return { success: true, output: result };
    } catch (err: any) {
      return { success: false, output: '', error: `Error leyendo archivo: ${err.message}` };
    }
  },

  /**
   * Escribe/sobreescribe un archivo completo.
   */
  write_to_file: async (args) => {
    const filePath = resolvePath(args.path as string);
    const content = args.content as string;
    
    if (!content) {
      return { success: false, output: '', error: 'Contenido vacío' };
    }

    try {
      const dir = path.dirname(filePath);
      if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
      }
      fs.writeFileSync(filePath, content, 'utf-8');
      return { success: true, output: `✅ Archivo escrito: ${filePath} (${Buffer.byteLength(content, 'utf-8')} bytes)` };
    } catch (err: any) {
      return { success: false, output: '', error: `Error escribiendo archivo: ${err.message}` };
    }
  },

  /**
   * Aplica cambios quirúrgicos SEARCH/REPLACE.
   */
  apply_diff: async (args) => {
    const filePath = resolvePath(args.path as string);
    const diff = args.diff as string;
    
    try {
      if (!fs.existsSync(filePath)) {
        return { success: false, output: '', error: `Archivo no encontrado: ${filePath}` };
      }
      
      const content = fs.readFileSync(filePath, 'utf-8');
      
      // Parsear bloques SEARCH/REPLACE
      const blocks = diff.split('<<<<<<< SEARCH');
      let modified = content;
      let changesApplied = 0;
      
      for (const block of blocks) {
        if (!block.includes('=======')) continue;
        
        // Extraer search y replace dentro del bloque actual
        const eqIdx = block.indexOf('=======');
        const searchText = block.substring(0, eqIdx).trim();
        let afterEq = block.substring(eqIdx + '======='.length);
        if (afterEq.includes('>>>>>>> REPLACE')) {
          afterEq = afterEq.split('>>>>>>> REPLACE')[0];
        }
        const replaceText = afterEq.trim();
        
        // Extraer línea de inicio si existe
        let searchContent = searchText;
        const lineMatch = searchContent.match(/^:start_line:\[?(\d+)\]?\n/);
        if (lineMatch) {
          searchContent = searchContent.replace(/^:start_line:\[?\d+\]?\n/, '');
        }
        
        if (modified.includes(searchContent)) {
          modified = modified.replace(searchContent, replaceText);
          changesApplied++;
        } else if (lineMatch) {
          // Buscar por línea específica
          const targetLine = parseInt(lineMatch[1]);
          const lines = modified.split('\n');
          if (targetLine <= lines.length) {
            lines[targetLine - 1] = replaceText;
            modified = lines.join('\n');
            changesApplied++;
          }
        }
      }
      
      if (changesApplied === 0) {
        return { success: false, output: '', error: 'No se encontró el contenido a reemplazar en el archivo. Verifica que el SEARCH coincida exactamente.' };
      }
      
      fs.writeFileSync(filePath, modified, 'utf-8');
      return { success: true, output: `✅ Aplicados ${changesApplied} cambio(s) en ${filePath}` };
    } catch (err: any) {
      return { success: false, output: '', error: `Error aplicando diff: ${err.message}` };
    }
  },

  /**
   * Busca archivos por regex.
   */
  search_files: async (args) => {
    const dirPath = resolvePath(args.path as string);
    const regex = args.regex as string;
    const filePattern = args.file_pattern as string || '*';
    
    try {
      if (!fs.existsSync(dirPath)) {
        return { success: false, output: '', error: `Directorio no encontrado: ${dirPath}` };
      }
      
      // Usar grep si está disponible
      const grepCmd = `grep -rn "${regex}" ${dirPath} --include="${filePattern}" 2>/dev/null | head -200`;
      
      return new Promise((resolve) => {
        exec(grepCmd, { maxBuffer: 5 * 1024 * 1024 }, (error, stdout, stderr) => {
          if (error && !stdout) {
            resolve({ success: true, output: 'No se encontraron resultados.' });
            return;
          }
          const lines = stdout.split('\n').filter(l => l.trim()).length;
          resolve({ success: true, output: stdout || 'No se encontraron resultados.', error: undefined });
        });
      });
    } catch (err: any) {
      return { success: false, output: '', error: `Error en búsqueda: ${err.message}` };
    }
  },

  /**
   * Lista archivos en un directorio.
   */
  list_files: async (args) => {
    const dirPath = resolvePath(args.path as string);
    const recursive = args.recursive === true;
    
    try {
      if (!fs.existsSync(dirPath)) {
        return { success: false, output: '', error: `Directorio no encontrado: ${dirPath}` };
      }
      
      if (recursive) {
        const files: string[] = [];
        function walk(dir: string, prefix: string = '') {
          const entries = fs.readdirSync(dir, { withFileTypes: true });
          for (const entry of entries) {
            const relPath = prefix ? `${prefix}/${entry.name}` : entry.name;
            if (entry.isDirectory()) {
              files.push(`${relPath}/`);
              walk(path.join(dir, entry.name), relPath);
            } else {
              files.push(relPath);
            }
          }
        }
        walk(dirPath);
        return { success: true, output: files.join('\n') };
      } else {
        const entries = fs.readdirSync(dirPath, { withFileTypes: true });
        const listing = entries.map(e => {
          const type = e.isDirectory() ? '📁' : '📄';
          return `${type} ${e.name}`;
        });
        return { success: true, output: listing.join('\n') };
      }
    } catch (err: any) {
      return { success: false, output: '', error: `Error listando: ${err.message}` };
    }
  },

  /**
   * Pregunta al usuario (envía a la UI del webview).
   */
  ask_question: async (args) => {
    const question = args.question as string;
    const options = (args.options as string)?.split('|') || ['OK'];
    
    // Esto se maneja a través del webview - devolvemos placeholder
    return { 
      success: true, 
      output: `[QUESTION] ${question}\nOpciones: ${options.join(', ')}\n(Esperando respuesta del usuario por la UI...)` 
    };
  },

  /**
   * Finaliza la tarea.
   */
  attempt_completion: async (args) => {
    const result = args.result as string;
    return { success: true, output: `[COMPLETED] ✅ ${result}` };
  },

  /**
   * Consulta SQLite.
   */
  database_query: async (args) => {
    const query = args.query as string;
    const dbPath = path.join(vscode.workspace.workspaceFolders?.[0]?.uri?.fsPath || process.cwd(), 'nexus_memoria.db');
    
    if (!query.toLowerCase().startsWith('select')) {
      return { success: false, output: '', error: 'Solo consultas SELECT permitidas' };
    }

    try {
      const result = execSync(`sqlite3 "${dbPath}" -header -column "${query.replace(/"/g, '\\"')}" 2>&1`, {
        timeout: 5000,
        encoding: 'utf-8'
      });
      return { success: true, output: result || '(empty result set)' };
    } catch (err: any) {
      return { success: false, output: '', error: `Error DB query: ${err.message}` };
    }
  },

  /**
   * Busqueda en memoria FTS5.
   */
  memory_search: async (args) => {
    const query = args.query as string;
    const limit = (args.limit as number) || 5;
    const dbPath = path.join(vscode.workspace.workspaceFolders?.[0]?.uri?.fsPath || process.cwd(), 'nexus_memoria.db');
    
    try {
      const escaped = query.replace(/'/g, "''");
      const sql = `SELECT text, rank FROM memoria_fts WHERE memoria_fts MATCH '${escaped}' ORDER BY rank LIMIT ${limit}`;
      const result = execSync(`sqlite3 "${dbPath}" -header -column "${sql}" 2>&1`, {
        timeout: 5000,
        encoding: 'utf-8'
      });
      return { success: true, output: result || '(no results)' };
    } catch (err: any) {
      return { success: false, output: '', error: `Error memory search: ${err.message}` };
    }
  },
  
  /**
   * Lee output de comando truncado (simulado con archivo temporal).
   */
  read_terminal_output: async (args) => {
    const artifactId = args.artifact_id as string;
    const tmpPath = `/tmp/${artifactId}`;
    
    try {
      if (!fs.existsSync(tmpPath)) {
        return { success: false, output: '', error: `Artifact no encontrado: ${tmpPath}` };
      }
      const content = fs.readFileSync(tmpPath, 'utf-8');
      const search = args.search as string | undefined;
      const offset = (args.offset as number) || 0;
      const limit = (args.limit as number) || content.length;
      
      let result = content.slice(offset, offset + limit);
      if (search) {
        const lines = result.split('\n').filter(l => l.toLowerCase().includes(search.toLowerCase()));
        result = lines.join('\n');
      }
      
      return { success: true, output: result || '(empty output)' };
    } catch (err: any) {
      return { success: false, output: '', error: `Error: ${err.message}` };
    }
  }
};

/**
 * Resuelve rutas relativas o absolutas.
 */
function resolvePath(inputPath: string): string {
  if (path.isAbsolute(inputPath)) {
    return inputPath;
  }
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri?.fsPath;
  if (workspaceRoot) {
    return path.join(workspaceRoot, inputPath);
  }
  return path.resolve(inputPath);
}
