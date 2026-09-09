// tools/nexus_multi_account_ext/src/popup.js
// 🔱 NEXUS OMEGA - Interfaz de Cambio de Contexto

document.addEventListener('DOMContentLoaded', () => {
  const buttons = {
    ARQUITECTO: document.getElementById('btn-arquitecto'),
    GABRIEL: document.getElementById('btn-gabriel'),
    LEGION: document.getElementById('btn-legion')
  };
  const statusMsg = document.getElementById('status-msg');

  Object.entries(buttons).forEach(([id, btn]) => {
    btn.addEventListener('click', () => {
      statusMsg.innerText = `Cambiando a ${id}...`;
      chrome.runtime.sendMessage({ action: "SWITCH_IDENTITY", identity: id }, (response) => {
        if (response && response.status === "SWITCHING") {
          statusMsg.innerText = `Estado: Identidad ${id} Activa.`;
          // Marcar botón activo
          Object.values(buttons).forEach(b => b.classList.remove('active'));
          btn.classList.add('active');
        }
      });
    });
  });
});
