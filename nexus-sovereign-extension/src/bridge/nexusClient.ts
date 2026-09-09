// ============================================================================
// 🔱 NEXUS — Cliente HTTP para comunicación con NEXUS Core
// ============================================================================
// ABSORBIDO de Antigravity (agentControlPanel.ts: healthCheck + consultarNexus)
// y MEJORADO: fetch nativo, timeout configurable, tipos fuertes,
// endpoints dinámicos desde constants.ts
// ============================================================================

import { NEXUS_API_BASE, NEXUS_ENDPOINTS } from '../constants';

// ---------------------------------------------------------------------------
// Interfaces de respuesta
// ---------------------------------------------------------------------------
export interface NexusHealthResponse {
    status: string;
    version?: string;
    uptime?: number;
    [key: string]: unknown;
}

export interface NexusConsultResponse {
    respuesta?: string;
    response?: string;
    error?: string;
    modo?: string;
    [key: string]: unknown;
}

export interface NexusMonologueResponse {
    pensamiento?: string;
    thought?: string;
    [key: string]: unknown;
}

// ---------------------------------------------------------------------------
// Opciones de consulta
// ---------------------------------------------------------------------------
export interface ConsultOptions {
    prompt: string;
    modelo?: string;
    modo?: string;
    timeout?: number;
}

// ---------------------------------------------------------------------------
// NexusClient — Singleton HTTP client
// ---------------------------------------------------------------------------
class NexusClient {
    private baseUrl: string;

    constructor(baseUrl: string = NEXUS_API_BASE) {
        this.baseUrl = baseUrl;
    }

    /** Cambiar URL base (ej: para conectar a engine-puro en otro puerto) */
    setBaseUrl(url: string): void {
        this.baseUrl = url;
    }

    /** Obtener URL base actual */
    getBaseUrl(): string {
        return this.baseUrl;
    }

    // ------------------------------------------------------------------
    // Health Check
    // ------------------------------------------------------------------
    /** Verificar si NEXUS Core responde */
    async healthCheck(timeoutMs: number = 3000): Promise<boolean> {
        try {
            const url = `${this.baseUrl}${NEXUS_ENDPOINTS.HEALTH}`;
            const controller = new AbortController();
            const timeout = setTimeout(() => controller.abort(), timeoutMs);

            const response = await fetch(url, {
                method: 'GET',
                signal: controller.signal,
            });
            clearTimeout(timeout);
            return response.ok;
        } catch {
            return Promise.resolve(false);
        }
    }

    /** Obtener payload completo de salud */
    async getHealth(timeoutMs: number = 3000): Promise<NexusHealthResponse | null> {
        try {
            const url = `${this.baseUrl}${NEXUS_ENDPOINTS.HEALTH}`;
            const controller = new AbortController();
            const timeout = setTimeout(() => controller.abort(), timeoutMs);

            const response = await fetch(url, {
                method: 'GET',
                signal: controller.signal,
            });
            clearTimeout(timeout);

            if (response.ok) {
                return (await response.json()) as NexusHealthResponse;
            }
            return null;
        } catch {
            return null;
        }
    }

    // ------------------------------------------------------------------
    // Consultar NEXUS (chat/completions style)
    // ------------------------------------------------------------------
    /** Enviar consulta a NEXUS Core */
    async consultar(
        options: ConsultOptions
    ): Promise<NexusConsultResponse> {
        const { prompt, modelo = 'orquestador', modo, timeout = 15000 } = options;

        const url = `${this.baseUrl}${NEXUS_ENDPOINTS.CONSULTAR}`;
        const body: Record<string, unknown> = { prompt, modelo };
        if (modo) body.modo = modo;

        const controller = new AbortController();
        const timer = setTimeout(() => controller.abort(), timeout);

        try {
            const response = await fetch(url, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(body),
                signal: controller.signal,
            });
            clearTimeout(timer);

            if (!response.ok) {
                const errorText = await response.text().catch(() => 'Unknown error');
                throw new Error(
                    `NEXUS respondió ${response.status}: ${errorText}`
                );
            }

            const data = (await response.json()) as NexusConsultResponse;
            return data;
        } catch (err) {
            clearTimeout(timer);
            if (err instanceof Error && err.name === 'AbortError') {
                throw new Error('Timeout consultando NEXUS Core');
            }
            throw err;
        }
    }

    // ------------------------------------------------------------------
    // Monólogo interno de NEXUS
    // ------------------------------------------------------------------
    /** Obtener pensamiento interno del motor puro */
    async getMonologue(timeoutMs: number = 5000): Promise<string | null> {
        try {
            const url = `${this.baseUrl}${NEXUS_ENDPOINTS.MONOLOGUE}`;
            const controller = new AbortController();
            const timeout = setTimeout(() => controller.abort(), timeoutMs);

            const response = await fetch(url, {
                method: 'GET',
                signal: controller.signal,
            });
            clearTimeout(timeout);

            if (response.ok) {
                const data = (await response.json()) as NexusMonologueResponse;
                return data.pensamiento || data.thought || null;
            }
            return null;
        } catch {
            return null;
        }
    }
}

// Exportar singleton
export const nexusClient = new NexusClient();
export default NexusClient;
