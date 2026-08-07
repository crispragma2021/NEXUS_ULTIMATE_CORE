// ============================================================================
// 🔱 NEXUS — Central de Servicios Singleton
// ============================================================================
// Punto único de acceso a SessionStore, ContextDetector, DiffPreview.
// Rompe dependencias circulares entre extension.ts ↔ agenticLoop.ts
// ============================================================================

import * as vscode from 'vscode';
import { SessionStore } from './persistence/SessionStore';
import { ContextDetector } from './tools/ContextDetector';
import { DiffPreview } from './panels/DiffPreview';

let _sessionStore: SessionStore;
let _contextDetector: ContextDetector;
let _diffPreview: DiffPreview;

/**
 * Inicializa todos los servicios. Llamar UNA VEZ desde extension.ts activate().
 */
export function initializeServices(context: vscode.ExtensionContext): void {
  _sessionStore = SessionStore.getInstance(context);
  _contextDetector = new ContextDetector();
  _diffPreview = new DiffPreview();
}

export function getSessionStore(): SessionStore {
  return _sessionStore;
}

export function getContextDetector(): ContextDetector {
  return _contextDetector;
}

export function getDiffPreview(): DiffPreview {
  return _diffPreview;
}
