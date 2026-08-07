// ============================================================================
// 🔱 NEXUS — ContextDetector: Analiza el workspace abierto automáticamente
// ============================================================================
// Extrae: estructura del proyecto, archivos abiertos, lenguajes, dependencias
// ============================================================================

import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';

export interface ProjectContext {
    name: string;
    root: string;
    languages: string[];
    totalFiles: number;
    openFiles: string[];
    fileTree: string;
    manifestInfo: Record<string, string>;
    gitBranch?: string;
}

const MANIFEST_FILES = ['package.json', 'Cargo.toml', 'pyproject.toml', 'go.mod', 'CMakeLists.txt', 'Makefile', 'Gemfile', 'composer.json', 'build.gradle', 'pom.xml'];

export class ContextDetector {
    private cachedContext: ProjectContext | null = null;
    private lastScan = 0;
    private readonly CACHE_TTL = 30000; // 30s

    async detect(forceRefresh = false): Promise<ProjectContext> {
        const now = Date.now();
        if (this.cachedContext && !forceRefresh && (now - this.lastScan) < this.CACHE_TTL) {
            return this.cachedContext;
        }

        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders || workspaceFolders.length === 0) {
            return this.emptyContext();
        }

        const root = workspaceFolders[0].uri.fsPath;
        const name = path.basename(root);
        const openFiles = vscode.window.tabGroups.all
            .flatMap(g => g.tabs)
            .filter(t => t.input instanceof vscode.TabInputText)
            .map(t => vscode.workspace.asRelativePath((t.input as vscode.TabInputText).uri));

        // Lenguajes detectados por extensiones de archivos abiertos
        const extMap: Record<string, string> = {
            '.ts': 'TypeScript', '.tsx': 'TypeScript/React', '.js': 'JavaScript', '.jsx': 'JavaScript/React',
            '.rs': 'Rust', '.py': 'Python', '.go': 'Go', '.c': 'C', '.cpp': 'C++',
            '.css': 'CSS', '.scss': 'SCSS', '.html': 'HTML',
            '.json': 'JSON', '.yaml': 'YAML', '.yml': 'YAML', '.md': 'Markdown',
            '.toml': 'TOML', '.sh': 'Shell', '.bash': 'Shell', '.nix': 'Nix',
        };

        const languages = new Set<string>();
        const fileTreeLines: string[] = [];

        // Escaneo rápido del proyecto (solo 2 niveles de profundidad para no saturar)
        try {
            const entries = fs.readdirSync(root, { withFileTypes: true });
            for (const entry of entries) {
                if (entry.name.startsWith('.') || entry.name === 'node_modules' || entry.name === 'target') continue;
                if (entry.isFile()) {
                    const ext = path.extname(entry.name);
                    if (extMap[ext]) languages.add(extMap[ext]);
                    fileTreeLines.push(`📄 ${entry.name}`);
                } else if (entry.isDirectory()) {
                    fileTreeLines.push(`📁 ${entry.name}/`);
                    try {
                        const sub = fs.readdirSync(path.join(root, entry.name), { withFileTypes: true });
                        for (const subEntry of sub.slice(0, 10)) {
                            if (subEntry.name.startsWith('.')) continue;
                            fileTreeLines.push(`  ${subEntry.isDirectory() ? '📁' : '📄'} ${subEntry.name}`);
                        }
                        if (sub.length > 10) fileTreeLines.push(`  ... (+${sub.length - 10} items)`);
                    } catch { /* no problem */ }
                }
            }
        } catch { /* no problem */ }

        // Manifest info
        const manifestInfo: Record<string, string> = {};
        for (const mf of MANIFEST_FILES) {
            const mfPath = path.join(root, mf);
            if (fs.existsSync(mfPath)) {
                try {
                    const content = fs.readFileSync(mfPath, 'utf-8').slice(0, 500);
                    manifestInfo[mf] = content;
                } catch { /* no problem */ }
            }
        }

        // Git branch (rápido, sin await)
        let gitBranch: string | undefined;
        try {
            const gitHead = path.join(root, '.git', 'HEAD');
            if (fs.existsSync(gitHead)) {
                const ref = fs.readFileSync(gitHead, 'utf-8').trim();
                if (ref.startsWith('ref: ')) {
                    gitBranch = ref.replace('ref: refs/heads/', '');
                }
            }
        } catch { /* no problem */ }

        this.cachedContext = {
            name,
            root,
            languages: Array.from(languages),
            totalFiles: entriesCount(root),
            openFiles,
            fileTree: fileTreeLines.join('\n'),
            manifestInfo,
            gitBranch,
        };
        this.lastScan = now;
        return this.cachedContext;
    }

    getContextSummary(context: ProjectContext): string {
        const parts: string[] = [
            `📁 **Proyecto**: ${context.name}`,
            `📍 **Ruta**: ${context.root}`,
            `📊 **Archivos**: ~${context.totalFiles}`,
            `🔤 **Lenguajes**: ${context.languages.join(', ') || 'no detectados'}`,
        ];
        if (context.gitBranch) parts.push(`🌿 **Branch**: ${context.gitBranch}`);
        if (context.openFiles.length > 0) {
            parts.push(`📂 **Archivos abiertos**:\n  ${context.openFiles.map(f => `- \`${f}\``).join('\n  ')}`);
        }
        if (context.fileTree) {
            parts.push(`🌳 **Estructura**:\n${context.fileTree}`);
        }
        return parts.join('\n');
    }

    private emptyContext(): ProjectContext {
        return {
            name: '(sin proyecto)',
            root: '',
            languages: [],
            totalFiles: 0,
            openFiles: [],
            fileTree: '',
            manifestInfo: {},
        };
    }
}

function entriesCount(dir: string, maxScan = 2000): number {
    let count = 0;
    try {
        const stack = [dir];
        while (stack.length > 0 && count < maxScan) {
            const current = stack.pop()!;
            const entries = fs.readdirSync(current, { withFileTypes: true });
            for (const e of entries) {
                if (e.name.startsWith('.') || e.name === 'node_modules' || e.name === 'target') continue;
                count++;
                if (e.isDirectory() && count < maxScan) {
                    stack.push(path.join(current, e.name));
                }
            }
        }
    } catch { /* no problem */ }
    return count;
}
