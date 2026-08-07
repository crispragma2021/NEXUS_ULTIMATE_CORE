#!/usr/bin/env node
/**
 * 🧬 TP-Link TL-WR840N Login v4 — MD5 MODE
 * getParm devuelve: ss (salt MD5), no RSA
 * Login: md5(base64(pass) + ss)
 */

import http from 'http';
import crypto from 'crypto';

const ROUTER = '192.168.0.1';
const COOKIE_JAR = new Map();

function request(method, path, body, contentType) {
  return new Promise((resolve, reject) => {
    const cookies = Array.from(COOKIE_JAR.entries())
      .map(([k, v]) => `${k}=${v}`).join('; ');

    const options = {
      hostname: ROUTER, port: 80, path, method,
      headers: {
        'User-Agent': 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36',
        'Referer': 'http://192.168.0.1/',
        'Accept': '*/*',
        'Connection': 'keep-alive',
      },
    };

    if (cookies) options.headers['Cookie'] = cookies;
    if (body && contentType) {
      options.headers['Content-Type'] = contentType;
      options.headers['Content-Length'] = Buffer.byteLength(body);
    }

    const req = http.request(options, (res) => {
      if (res.headers['set-cookie']) {
        const cArr = Array.isArray(res.headers['set-cookie'])
          ? res.headers['set-cookie'] : [res.headers['set-cookie']];
        for (const c of cArr) {
          const m = c.match(/^([^=]+)=([^;]+)/);
          if (m) {
            if (m[2] !== 'deleted') COOKIE_JAR.set(m[1], m[2]);
            else COOKIE_JAR.delete(m[1]);
          }
        }
      }
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => resolve({ status: res.statusCode, headers: res.headers, body: data }));
    });
    req.on('error', reject);
    req.setTimeout(5000, () => req.destroy(new Error('Timeout')));
    if (body) req.write(body);
    req.end();
  });
}

function get(path) { return request('GET', path); }
function post(path, body, ct) { return request('POST', path, body, ct || 'application/x-www-form-urlencoded'); }

function md5Encrypt(value, salt) {
  return crypto.createHash('md5').update(value + salt).digest('hex');
}

async function tryPasswords(passwords) {
  for (const pw of passwords) {
    console.log(`\n🔐 Trying password: "${pw}"`);

    // Step 1: GET homepage + cookie
    await get('/');

    // Step 2: GET auth params
    const parmRes = await post('/cgi/getParm', '', 'text/plain');
    const saltMatch = parmRes.body.match(/ss="([^"]+)"/);
    if (!saltMatch) { console.log('   ❌ No salt received'); continue; }
    const salt = saltMatch[1];
    console.log(`   Salt: ${salt}`);

    // Step 3: Compute MD5 auth
    const b64Pass = Buffer.from(pw).toString('base64');
    const b64User = Buffer.from('admin').toString('base64');
    const authHash = md5Encrypt(b64Pass, salt);
    const nameHash = md5Encrypt(b64User, salt);
    console.log(`   b64(pass): "${b64Pass}"`);
    console.log(`   MD5("${b64Pass}" + "${salt}") = ${authHash}`);

    // Step 4: POST login
    const body = `name=${encodeURIComponent(nameHash)}&auth=${encodeURIComponent(authHash)}&action=1`;
    const loginRes = await post('/cgi/login', body);
    console.log(`   Login status: ${loginRes.status}`);

    if (loginRes.status === 200) {
      console.log(`   ✅ LOGIN EXITOSO con password: "${pw}"`);

      // Step 5: Verify with getBusy
      const busyRes = await post('/cgi/getBusy', '', 'text/plain');
      console.log(`   getBusy: ${busyRes.body.substring(0, 200)}`);

      // Check if actually logged in
      if (busyRes.body.includes('isLogined=1')) {
        console.log('   ✅ Confirmado: sesión activa!');
      }

      // Step 6: Get status
      console.log('\n📡 STATUS PAGE:');
      const statusRes = await get('/cgi/status');
      console.log(`   Status: ${statusRes.status}`);
      console.log(`   Body:\n${statusRes.body.substring(0, 500)}`);

      // Step 7: Try WAN status
      console.log('\n📡 WAN STATUS:');
      const wanRes = await get('/userRpm/StatusRpm.htm');
      console.log(`   Status: ${wanRes.status}`);
      if (wanRes.status === 200) {
        // Extract WAN IP
        const body = wanRes.body;
        const ipMatch = body.match(/WAN[^<]*?(?:IP|ip|Ip)[^:]*:\s*([^<]+)/);
        if (ipMatch) console.log(`   WAN IP: ${ipMatch[1].trim()}`);
        const connMatch = body.match(/Connection[^:]*:\s*([^<]+)/);
        if (connMatch) console.log(`   Connection: ${connMatch[1].trim()}`);
        console.log(body.substring(0, 800));
      } else {
        // Try alternative status endpoints
        const altRes = await get('/');
        // Check if we see status info in the main page post-login
        console.log('   Status page blocked, checking main page...');
      }

      return true;
    } else {
      console.log(`   ❌ Login falló para "${pw}"`);
    }
  }
  return false;
}

async function main() {
  console.log('🧬 TP-Link TL-WR840N - Login v4 (MD5 Mode)\n');

  // Common TP-Link passwords
  const passwords = [
    'admin', '', '1234', '12345', '12345678', 'password',
    'root', 'user', 'Admin', 'ADMIN', 'guest', '123456789',
    'telecom', 'Telecom', 'tplink', 'Tplink', 'TP-LINK',
  ];

  const success = await tryPasswords(passwords);
  if (!success) {
    console.log('\n❌ No se pudo loguear con las contraseñas comunes.');
    console.log('Arquitecto, por favor revisa el sticker del router o dime la contraseña.');
  }
}

main().catch(console.error);
