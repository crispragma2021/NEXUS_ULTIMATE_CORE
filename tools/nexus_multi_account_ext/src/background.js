// tools/nexus_multi_account_ext/src/background.js
// 🔱 NEXUS OMEGA - Motor de Infiltración Multi-Cuenta (Cifrado AES-GCM)

const IDENTITIES = {
  ARQUITECTO: "arquitecto_context",
  GABRIEL: "gabriel_context",
  LEGION: "legion_context"
};

// Generar o recuperar clave maestra para cifrado (Soberanía de Datos)
async function getMasterKey() {
  const result = await chrome.storage.local.get(['nexus_master_key']);
  if (result.nexus_master_key) {
    const keyData = new Uint8Array(result.nexus_master_key);
    return await crypto.subtle.importKey("raw", keyData, { name: "AES-GCM" }, false, ["encrypt", "decrypt"]);
  } else {
    const key = await crypto.subtle.generateKey({ name: "AES-GCM", length: 256 }, true, ["encrypt", "decrypt"]);
    const rawKey = await crypto.subtle.exportKey("raw", key);
    await chrome.storage.local.set({ 'nexus_master_key': Array.from(new Uint8Array(rawKey)) });
    return key;
  }
}

// Cifrar datos antes de guardarlos en el storage local
async function encryptData(data) {
  const key = await getMasterKey();
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const encoded = new TextEncoder().encode(JSON.stringify(data));
  const ciphertext = await crypto.subtle.encrypt({ name: "AES-GCM", iv }, key, encoded);
  return { ciphertext: Array.from(new Uint8Array(ciphertext)), iv: Array.from(iv) };
}

// Descifrar datos al recuperarlos
async function decryptData(encryptedObj) {
  const key = await getMasterKey();
  const decrypted = await crypto.subtle.decrypt(
    { name: "AES-GCM", iv: new Uint8Array(encryptedObj.iv) },
    key,
    new Uint8Array(encryptedObj.ciphertext)
  );
  return JSON.parse(new TextDecoder().decode(decrypted));
}

// Escuchar mensajes (Popup e Inyección Nativa)
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === "SWITCH_IDENTITY") {
    switchIdentity(request.identity);
    sendResponse({ status: "SWITCHING", identity: request.identity });
  }
  return true;
});

// Listener para Inyección Directa desde NEXUS (Rust Bridge)
chrome.runtime.onConnectNative.addListener((port) => {
  port.onMessage.addListener(async (msg) => {
    console.log("🔱 NEXUS: Recibida orden nativa", msg);
    if (msg.action === "INJECT_COOKIES") {
      const encrypted = await encryptData(msg.cookies);
      await chrome.storage.local.set({ [`nexus_cookies_${msg.identityId}`]: encrypted });
    }
  });
});

async function switchIdentity(identityId) {
  console.log(`🔀 NEXUS: Cambiando a identidad ${identityId}...`);
  
  const domains = ["web.whatsapp.com", ".facebook.com", ".messenger.com"];
  for (const domain of domains) {
    const cookies = await chrome.cookies.getAll({ domain });
    for (const cookie of cookies) {
      const url = `http${cookie.secure ? 's' : ''}://${cookie.domain}${cookie.path}`;
      await chrome.cookies.remove({ url, name: cookie.name });
    }
  }

  const storageKey = `nexus_cookies_${identityId}`;
  chrome.storage.local.get([storageKey], async (result) => {
    if (result[storageKey]) {
      try {
        const savedCookies = await decryptData(result[storageKey]);
        for (const c of savedCookies) {
          const url = `http${c.secure ? 's' : ''}://${c.domain}${c.path}`;
          await chrome.cookies.set({
            url, name: c.name, value: c.value, domain: c.domain,
            path: c.path, secure: c.secure, httpOnly: c.httpOnly,
            expirationDate: c.expirationDate
          });
        }
        chrome.tabs.query({ url: ["*://web.whatsapp.com/*", "*://*.facebook.com/*"] }, (tabs) => {
          tabs.forEach(tab => chrome.tabs.reload(tab.id));
        });
      } catch (e) {
        console.error("🛑 NEXUS: Error descifrando búnker de cookies", e);
      }
    }
  });
}
