const { exec } = require('child_process');
const fs = require('fs');
const path = require('path');


class ProxyMesh {
  constructor() {
    this.torActive = false;
    this.currentCircuit = 0;
    this.maxRequestsPerCircuit = 5;
    this.requestCount = 0;
    this.humanDelayMs = { min: 1500, max: 4000 }; // ms entre peticiones
    this.torControlPort = 9051;
    this.torControlPassword = process.env.TOR_CONTROL_PASSWORD; // Se debe configurar en .env
  }

  async init() {
    try {
      // Verificar si Tor está activo y el SOCKS5 está escuchando
      await this.verifyTorSOCKS5();
      this.torActive = true;
      console.error('✅ [ProxyMesh] Tor SOCKS5 activo y accesible.');
    } catch (error) {
      console.error(`⚠️ [ProxyMesh] Tor SOCKS5 no detectado o inactivo: ${error.message}. Funcionará en modo directo.`);
      this.torActive = false;
    }
  }

  async verifyTorSOCKS5() {
    return new Promise((resolve, reject) => {
      exec('ss -tlnp | grep -q 9050', (error) => {
        if (error) {
          return reject(new Error('Puerto SOCKS5 9050 no está escuchando.'));
        }
        resolve(true);
      });
    });
  }

  async getTorControlPassword() {
    if (!this.torControlPassword) {
      throw new Error('TOR_CONTROL_PASSWORD no está configurada en las variables de entorno.');
    }
    return this.torControlPassword;
  }



  async rotateCircuit() {
    if (!this.torActive) {
      console.error('ℹ️ [ProxyMesh] Tor no está activo, no se puede rotar circuito.');
      return;
    }

    console.error('🔄 [ProxyMesh] Solicitando nuevo circuito Tor...');
    try {
      const password = await this.getTorControlPassword();
      const command = `printf "AUTHENTICATE \\"${password}\\"\\r\\nSIGNAL NEWNYM\\r\\nQUIT\\r\\n" | nc 127.0.0.1 ${this.torControlPort}`;
      
      const { stdout, stderr } = await new Promise((resolve, reject) => {
        exec(command, (error, stdout, stderr) => {
          if (error) return reject(error);
          resolve({ stdout, stderr });
        });
      });

      console.error(`[ProxyMesh] NC stdout: ${stdout.trim()}`);
      console.error(`[ProxyMesh] NC stderr: ${stderr.trim()}`);

      if (stdout.includes('250 OK')) {
        this.currentCircuit++;
        this.requestCount = 0;
        console.error(`✅ [ProxyMesh] Nuevo circuito Tor establecido. Circuito #${this.currentCircuit}`);
        await this.humanDelay(2000, 5000); // Esperar un poco para que el circuito se estabilice
      } else {
        throw new Error(`Respuesta inesperada del control Tor: ${stdout} ${stderr}`);
      }
    } catch (error) {
      console.error(`❌ [ProxyMesh] Error al rotar circuito Tor: ${error.message}`);
      this.torActive = false; // Desactivar Tor si hay un error crítico
    }
  }

  async verifyIP() {
    if (!this.torActive) {
      return { isTor: false, ip: 'DIRECT_CONNECTION' };
    }
    try {
      const command = 'curl --socks5-hostname 127.0.0.1:9050 -s https://check.torproject.org/api/ip';
      const { stdout } = await new Promise((resolve, reject) => {
        exec(command, (error, stdout, stderr) => {
          if (error) return reject(error);
          resolve({ stdout, stderr });
        });
      });
      const ipInfo = JSON.parse(stdout);
      console.error(`🌐 [ProxyMesh] IP actual (Tor): ${ipInfo.IP} (IsTor: ${ipInfo.IsTor})`);
      return ipInfo;
    } catch (error) {
      console.error(`❌ [ProxyMesh] Error al verificar IP de Tor: ${error.message}`);
      return { isTor: false, ip: 'TOR_VERIFICATION_FAILED' };
    }
  }

  async getProxyConfig() {
    if (this.torActive && this.requestCount >= this.maxRequestsPerCircuit) {
      await this.rotateCircuit();
    }
    this.requestCount++;
    return this.torActive ? { server: 'socks5://127.0.0.1:9050' } : null; // null para conexión directa
  }

  async humanDelay(min = this.humanDelayMs.min, max = this.humanDelayMs.max) {
    const delay = Math.floor(Math.random() * (max - min + 1)) + min;
    console.error(`⏳ [ProxyMesh] Simulating human delay for ${delay}ms...`);
    return new Promise(resolve => setTimeout(resolve, delay));
  }
}

// Inicializar y exportar una instancia global (singleton)
const proxyMesh = new ProxyMesh();
proxyMesh.init(); // Iniciar verificación de Tor al cargar el módulo

module.exports = proxyMesh;
