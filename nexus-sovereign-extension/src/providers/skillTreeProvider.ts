// ============================================================================
// 🔱 NEXUS — SkillTreeProvider: Árbol de skills NEXUS
// ============================================================================
// NUEVO (no existía en Antigravity): Escanea .agent/skills/ y lee SKILL.md
// de cada skill para mostrar su nombre y descripción en el árbol.
// ============================================================================

import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';

// ---------------------------------------------------------------------------
// SkillItem — Cada skill en el árbol
// ---------------------------------------------------------------------------
export class SkillItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly skillPath: string,
        private descriptionText: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState
    ) {
        super(label, collapsibleState);

        this.tooltip = `🧩 Skill: ${label}\n${descriptionText}\nRuta: ${skillPath}`;
        this.description = descriptionText.length > 40
            ? descriptionText.substring(0, 40) + '…'
            : descriptionText;

        // Icono de skill
        this.iconPath = new vscode.ThemeIcon('symbol-miscellaneous');
    }
}

// ---------------------------------------------------------------------------
// SkillTreeProvider — Proveedor de datos para el árbol de skills
// ---------------------------------------------------------------------------
export class SkillTreeProvider implements vscode.TreeDataProvider<SkillItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<SkillItem | undefined | null | void> =
        new vscode.EventEmitter<SkillItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<SkillItem | undefined | null | void> =
        this._onDidChangeTreeData.event;

    constructor(private context: vscode.ExtensionContext) {}

    /** Refrescar el árbol */
    refresh(): void {
        this._onDidChangeTreeData.fire(undefined);
    }

    getTreeItem(element: SkillItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: SkillItem): Thenable<SkillItem[]> {
        if (!element) {
            return this.getSkills();
        }
        return Promise.resolve([]);
    }

    /** Escanear el directorio .agent/skills/ del workspace */
    private getSkills(): Promise<SkillItem[]> {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders) {
            return Promise.resolve([]);
        }

        const skillsDir = path.join(
            workspaceFolders[0].uri.fsPath,
            '.agent',
            'skills'
        );

        return new Promise(resolve => {
            if (!fs.existsSync(skillsDir)) {
                resolve([]);
                return;
            }

            try {
                const entries = fs.readdirSync(skillsDir);
                const skills = entries
                    .filter(entry => {
                        const fullPath = path.join(skillsDir, entry);
                        return fs.statSync(fullPath).isDirectory();
                    })
                    .map(skillName => {
                        const skillPath = path.join(skillsDir, skillName);
                        const description = this.readSkillDescription(skillPath, skillName);
                        return new SkillItem(
                            skillName,
                            skillPath,
                            description,
                            vscode.TreeItemCollapsibleState.None
                        );
                    });

                resolve(skills);
            } catch (e) {
                console.error('❌ [NEXUS] Error leyendo skills:', e);
                resolve([]);
            }
        });
    }

    /** Leer SKILL.md o index.md dentro del directorio de la skill */
    private readSkillDescription(skillPath: string, fallback: string): string {
        const candidates = ['SKILL.md', 'skill.md', 'README.md', 'readme.md', 'index.md'];
        for (const candidate of candidates) {
            const candidatePath = path.join(skillPath, candidate);
            if (fs.existsSync(candidatePath)) {
                try {
                    const content = fs.readFileSync(candidatePath, 'utf-8').trim();
                    // Extraer primera línea no vacía que no sea comentario
                    const lines = content.split('\n');
                    for (const line of lines) {
                        const stripped = line.replace(/^#+\s*/, '').replace(/^\s*\*+\s*/, '').trim();
                        if (stripped.length > 0 && !stripped.startsWith('[')) {
                            return stripped.length > 100
                                ? stripped.substring(0, 100) + '…'
                                : stripped;
                        }
                    }
                } catch {
                    // fallback
                }
            }
        }
        return `Skill: ${fallback}`;
    }
}
