// ═══════════════════════════════════════════════════════════════════════════
// NEXUS TR — Panel de Identidades (Sembrador OMEGA)
// ═══════════════════════════════════════════════════════════════════════════
// Gestión de identidades sintéticas: crear, listar, monitorear
// ═══════════════════════════════════════════════════════════════════════════

const API_BASE = '/api';

// ─── Estado ──────────────────────────────────────────────────────────────
const STATE_ID = {
  identities: [],
  report: null,
  selectedType: 'temporal',
};

// ─── DOM ─────────────────────────────────────────────────────────────────
const DOM_ID = {
  panel: document.getElementById('identityPanel'),
  btnCreate: document.getElementById('btnCreateIdentity'),
  selectType: document.getElementById('identityTypeSelect'),
  identityList: document.getElementById('identityList'),
  identityStats: document.getElementById('identityStats'),
  identityReport: document.getElementById('identityReport'),
};

// ─── Crear identidades ───────────────────────────────────────────────────
async function crearIdentidades() {
  const tipo = DOM_ID.selectType ? DOM_ID.selectType.value : 'temporal';
  const cantidad = 1;

  DOM_ID.btnCreate.disabled = true;
  DOM_ID.btnCreate.textContent = '⏳ Sembrando...';

  try {
    const res = await fetch(`${API_BASE}/identidades/sembrar?tipo=${tipo}&cantidad=${cantidad}`);
    const data = await res.json();

    if (data.status === 'ok' || data.status === 'parcial') {
      console.log(`🧬 [IDENTIDADES] ${data.sembradas} sembradas, ${data.errores} errores`);
      // Agregar al inicio de la lista
      if (data.identidades && data.identidades.length > 0) {
        STATE_ID.identities.unshift(...data.identidades);
        if (STATE_ID.identities.length > 200) {
          STATE_ID.identities = STATE_ID.identities.slice(0, 200);
        }
      }
      renderIdentityList();
      cargarReporte();
    } else {
      console.error('❌ [IDENTIDADES] Error:', data.mensaje);
      addTerminalOutput?.(`❌ Error al sembrar: ${data.mensaje}`, 'error');
    }
  } catch (e) {
    console.error('❌ [IDENTIDADES] Error de red:', e);
    addTerminalOutput?.('❌ Error de red al sembrar', 'error');
  } finally {
    DOM_ID.btnCreate.disabled = false;
    DOM_ID.btnCreate.textContent = '🌱 Sembrar';
  }
}

// ─── Cargar lista de identidades ─────────────────────────────────────────
async function cargarIdentidades() {
  try {
    const res = await fetch(`${API_BASE}/identidades?cantidad=100`);
    const data = await res.json();
    if (data.status === 'ok') {
      STATE_ID.identities = data.identidades || [];
      renderIdentityList();
    }
  } catch (e) {
    console.error('❌ [IDENTIDADES] Error al cargar lista:', e);
  }
}

// ─── Renderizar lista ────────────────────────────────────────────────────
function renderIdentityList() {
  if (!DOM_ID.identityList) return;

  if (STATE_ID.identities.length === 0) {
    DOM_ID.identityList.innerHTML = `<div class="empty-identities">
      <div class="empty-icon">🧬</div>
      <div>Ninguna identidad sembrada aún</div>
      <div class="empty-hint">Usa "Sembrar" para generar tu primera identidad</div>
    </div>`;
    return;
  }

  DOM_ID.identityList.innerHTML = STATE_ID.identities.slice(0, 50).map(id => {
    const estadoClass = getEstadoClass(id.estado);
    const tipoIcon = getTipoIcon(id.tipo);
    return `<div class="identity-card" onclick="verIdentidad('${id.email}')">
      <div class="identity-avatar">${tipoIcon}</div>
      <div class="identity-info">
        <div class="identity-name">${id.nombre || '—'}</div>
        <div class="identity-email">${id.email}</div>
        <div class="identity-meta">
          <span class="identity-type">${id.tipo}</span>
          <span class="identity-country">${id.pais || '—'}</span>
        </div>
      </div>
      <div class="identity-status ${estadoClass}">${id.estado}</div>
    </div>`;
  }).join('');
}

function getEstadoClass(estado) {
  const map = {
    'creada': 'status-created',
    'email_verificado': 'status-verified',
    'activa': 'status-active',
    'bloqueada': 'status-blocked',
    'en_proceso': 'status-pending',
    'fallida': 'status-failed',
  };
  return map[estado] || 'status-unknown';
}

function getTipoIcon(tipo) {
  const icons = {
    'temporal': '⏳',
    'gmail': '📧',
    'proton': '🔒',
    'facebook': '📘',
    'twitter': '🐦',
    'sintetico': '🧬',
  };
  return icons[tipo] || '📋';
}

// ─── Ver detalle de identidad ────────────────────────────────────────────
async function verIdentidad(email) {
  try {
    const res = await fetch(`${API_BASE}/identidades/${encodeURIComponent(email)}`);
    const data = await res.json();
    if (data.status === 'ok' && data.identidad) {
      const id = data.identidad;
      mostrarDetalle(id);
    }
  } catch (e) {
    console.error('❌ [IDENTIDADES] Error al obtener detalle:', e);
  }
}

function mostrarDetalle(id) {
  const overlay = document.createElement('div');
  overlay.className = 'identity-overlay';
  overlay.innerHTML = `
    <div class="identity-modal">
      <div class="modal-header">
        <span>🧬 ${id.nombre} ${id.apellido}</span>
        <button onclick="this.parentElement.parentElement.parentElement.remove()" class="modal-close">✕</button>
      </div>
      <div class="modal-body">
        <div class="detail-grid">
          <div class="detail-item">
            <label>Email</label>
            <span class="detail-value mono">${id.email}</span>
          </div>
          <div class="detail-item">
            <label>Contraseña</label>
            <span class="detail-value mono">${id.password}</span>
          </div>
          <div class="detail-item">
            <label>Recovery</label>
            <span class="detail-value">${id.recovery_email || '—'}</span>
          </div>
          <div class="detail-item">
            <label>Fecha Nac.</label>
            <span class="detail-value">${id.fecha_nacimiento || '—'}</span>
          </div>
          <div class="detail-item">
            <label>País / Ciudad</label>
            <span class="detail-value">${id.pais || '—'} / ${id.ciudad || '—'}</span>
          </div>
          <div class="detail-item">
            <label>Género</label>
            <span class="detail-value">${id.genero || '—'}</span>
          </div>
          <div class="detail-item">
            <label>Tipo</label>
            <span class="detail-value">${id.tipo}</span>
          </div>
          <div class="detail-item">
            <label>Estado</label>
            <span class="detail-value ${getEstadoClass(id.estado)}">${id.estado}</span>
          </div>
          <div class="detail-item">
            <label>Proveedor</label>
            <span class="detail-value">${id.email_provider || '—'}</span>
          </div>
          <div class="detail-item">
            <label>Creado</label>
            <span class="detail-value">${id.creado_en || '—'}</span>
          </div>
        </div>
        <div class="modal-actions">
          <button class="btn-nexus" onclick="copiarAlPortapapeles('${id.email}:${id.password}')">📋 Copiar credenciales</button>
          <button class="btn-nexus secondary" onclick="this.parentElement.parentElement.parentElement.remove()">Cerrar</button>
        </div>
      </div>
    </div>
  `;
  document.body.appendChild(overlay);
}

function copiarAlPortapapeles(texto) {
  navigator.clipboard.writeText(texto).then(() => {
    addTerminalOutput?.('📋 Credenciales copiadas al portapapeles', 'success');
  });
}

// ─── Cargar reporte ──────────────────────────────────────────────────────
async function cargarReporte() {
  try {
    const res = await fetch(`${API_BASE}/identidades/reporte`);
    const data = await res.json();
    if (data.status === 'ok' && data.reporte) {
      STATE_ID.report = data.reporte;
      renderReport();
    }
  } catch (e) {
    console.error('❌ [IDENTIDADES] Error al cargar reporte:', e);
  }
}

function renderReport() {
  if (!DOM_ID.identityStats || !STATE_ID.report) return;
  const r = STATE_ID.report;

  DOM_ID.identityStats.innerHTML = `
    <div class="stat-card">
      <div class="stat-value">${r.total || 0}</div>
      <div class="stat-label">Total</div>
    </div>
    <div class="stat-card active">
      <div class="stat-value">${r.activas || 0}</div>
      <div class="stat-label">Activas</div>
    </div>
    <div class="stat-card">
      <div class="stat-value">${r.por_tipo ? Object.keys(r.por_tipo).length : 0}</div>
      <div class="stat-label">Tipos</div>
    </div>
  `;

  if (DOM_ID.identityReport) {
    DOM_ID.identityReport.textContent = JSON.stringify(r, null, 2);
  }
}

// ─── Inicializar ─────────────────────────────────────────────────────────
async function initIdentidades() {
  // Event listeners
  if (DOM_ID.btnCreate) {
    DOM_ID.btnCreate.addEventListener('click', crearIdentidades);
  }

  // Cargar datos iniciales
  await Promise.all([
    cargarIdentidades(),
    cargarReporte(),
  ]);

  console.log('🧬 [IDENTIDADES] Panel listo');
}

// Auto-inicializar si hay panel
if (document.getElementById('identityPanel')) {
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initIdentidades);
  } else {
    initIdentidades();
  }
}
