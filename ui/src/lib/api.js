/**
 * Cliente API del orquestador NEXUS.
 *
 * El backend Rust (orquestador) expone:
 *   GET  /api/projects            → lista de proyectos con estado y puerto
 *   POST /api/projects            → registrar proyecto (asigna puerto inmutable)
 *   GET  /api/projects/:id/status → healthcheck en vivo de un proyecto
 *   POST /api/messages/resolve    → ScopeMapper: contexto aislado por proyecto
 */

const BASE = '/api'

async function request(path, options) {
  const res = await fetch(`${BASE}${path}`, options)
  if (!res.ok) throw new Error(`HTTP ${res.status}`)
  return res.json()
}

/** Lista proyectos con estado + puerto (para el grid de alta densidad). */
export function fetchProjects() {
  return request('/projects')
}

/** Healthcheck en vivo de un proyecto. */
export function fetchProjectStatus(id) {
  return request(`/projects/${encodeURIComponent(id)}/status`)
}

/** Registra un proyecto; el backend le asigna un puerto inmutable (8000-8999). */
export function registerProject(payload) {
  return request('/projects', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload)
  })
}

/** ScopeMapper: resuelve el contexto aislado para un mensaje. */
export function resolveContext(message) {
  return request('/messages/resolve', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ message })
  })
}
