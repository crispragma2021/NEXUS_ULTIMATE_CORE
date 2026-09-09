import * as vscode from 'vscode';
import { toolExecutors } from './executor'; // Asumo que executor.ts ya expone toolExecutors
import { ToolResult } from './executor';

// Definir los argumentos para consultar_memoria
interface ConsultarMemoriaArgs {
    query: string;
    modo?: 'snapshot' | 'search' | 'status';
}

// Definir los argumentos para ejecutar_comando
interface EjecutarComandoArgs {
    command: string;
    cwd?: string;
    timeout?: number;
}

/**
 * Invoca la tool mcp__nexus_claws_mcp__consultar_memoria.
 */
export async function mcp__nexus_claws_mcp__consultar_memoria(args: ConsultarMemoriaArgs, context: vscode.ExtensionContext): Promise<ToolResult> {
    return toolExecutors['mcp__nexus_claws_mcp__consultar_memoria'](args, context);
}

/**
 * Invoca la tool mcp__nexus_claws_mcp__ejecutar_comando.
 */
export async function mcp__nexus_claws_mcp__ejecutar_comando(args: EjecutarComandoArgs, context: vscode.ExtensionContext): Promise<ToolResult> {
    return toolExecutors['mcp__nexus_claws_mcp__ejecutar_comando'](args, context);
}
