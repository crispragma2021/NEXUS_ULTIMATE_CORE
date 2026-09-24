import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ARTIFACTS_DIR = path.resolve(__dirname, '../artifacts');

const OPENROUTER_API_KEY = process.env.OPENROUTER_API_KEY || "";

export async function runUIExtractor(userPrompt) {
  const prompt = userPrompt || "Dashboard de control general";
  console.log(`🎨 [AGENT UI LLM] Generando UI con IA (OpenRouter 70B) para: "${prompt}"`);

  let uiSpec = null;

  try {
    uiSpec = await generateUIWithLLM(prompt);
  } catch (err) {
    console.warn(`⚠️ [AGENT UI LLM] Inferencia remota falló, usando fallback adaptativo. Error:`, err.message);
    uiSpec = fallbackUIGenerator(prompt);
  }

  // Ajustes de iconos según prompt (ej: conejito/rabbit)
  if (prompt.toLowerCase().includes('conejito') || prompt.toLowerCase().includes('conejo') || prompt.toLowerCase().includes('rabbit')) {
    const kpi = uiSpec.components?.find(c => c.id === 'kpi_total_revenue' || c.title?.toLowerCase().includes('ingresos'));
    if (kpi) {
      kpi.icon = "rabbit";
    } else if (uiSpec.components && uiSpec.components.length > 0) {
      uiSpec.components[0].icon = "rabbit";
    }
  }

  const outputPath = path.join(ARTIFACTS_DIR, 'ui_spec.json');
  fs.writeFileSync(outputPath, JSON.stringify(uiSpec, null, 2), 'utf-8');
  
  compileLiveAppHTML(userPrompt, uiSpec);

  console.log(`✅ [AGENT UI LLM] ui_spec.json y generated_app.html actualizados para "${uiSpec.app_title}"`);
  return uiSpec;
}

export function compileLiveAppHTML(userPrompt, uiSpec) {
  const p = (userPrompt || '').toLowerCase();
  const liveAppFile = path.join(ARTIFACTS_DIR, 'generated_app.html');

  let html = '';

  if (p.includes('gdevelop') || p.includes('juego') || p.includes('game') || p.includes('clonar')) {
    html = `<!DOCTYPE html>
<html lang="es" class="dark">
<head>
  <meta charset="UTF-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
  <title>GDevelop 5 · Editor de Juegos 2D & Motor No-Code</title>
  <script src="https://cdn.tailwindcss.com"></script>
  <script src="https://unpkg.com/lucide@latest"></script>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800;900&display=swap" rel="stylesheet"/>
  <style>
    body { font-family: 'Inter', sans-serif; background: #0c0f1d; color: #e2e8f0; }
    .gdevelop-card { background: rgba(23, 28, 48, 0.95); border: 1px solid rgba(75, 111, 255, 0.2); }
    .gdevelop-btn { background: #4b6fff; color: white; font-weight: 700; }
    .gdevelop-btn:hover { background: #3557e6; }
    .canvas-grid { background-size: 24px 24px; background-image: linear-gradient(to right, rgba(255, 255, 255, 0.04) 1px, transparent 1px), linear-gradient(to bottom, rgba(255, 255, 255, 0.04) 1px, transparent 1px); }
  </style>
</head>
<body class="h-screen flex flex-col justify-between overflow-hidden select-none">

  <!-- HEADER GDEVELOP 5 NAVBAR -->
  <header class="h-12 bg-[#141829] border-b border-[#2a3356] px-4 flex items-center justify-between flex-shrink-0">
    <div class="flex items-center gap-3">
      <div class="w-7 h-7 rounded-lg bg-[#4b6fff] flex items-center justify-center font-black text-white text-xs shadow-md">
        GD
      </div>
      <div class="flex items-center gap-2">
        <span class="font-extrabold text-white text-sm">GDevelop 5</span>
        <span class="text-xs text-slate-400 font-mono">/ Space_Runner_2D.json</span>
      </div>
    </div>

    <!-- PESTAÑAS DE ESCENAS -->
    <div class="flex items-center bg-[#0c0f1d] p-1 rounded-lg border border-[#2a3356] text-xs">
      <button class="px-3 py-1 bg-[#4b6fff] text-white font-bold rounded-md flex items-center gap-1.5 shadow">
        <i data-lucide="layout-grid" class="w-3.5 h-3.5"></i> Escena 1 (Main Level)
      </button>
      <button class="px-3 py-1 text-slate-400 hover:text-white transition-all flex items-center gap-1.5">
        <i data-lucide="code" class="w-3.5 h-3.5"></i> Hoja de Eventos
      </button>
      <button class="px-3 py-1 text-slate-400 hover:text-white transition-all flex items-center gap-1.5">
        <i data-lucide="settings" class="w-3.5 h-3.5"></i> Ajustes
      </button>
    </div>

    <!-- BOTONES PROBAR Y EXPORTAR -->
    <div class="flex items-center gap-2">
      <button onclick="launchGamePreview()" class="px-4 py-1.5 rounded-lg bg-emerald-500 hover:bg-emerald-400 text-black font-extrabold text-xs shadow-lg transition-all flex items-center gap-1.5 active:scale-95">
        <i data-lucide="play" class="w-4 h-4 fill-black"></i> Probador de Juego ▶
      </button>
      <button onclick="alert('🚀 Exportando juego a HTML5/Android/Desktop...')" class="px-3 py-1.5 rounded-lg bg-[#2a3356] hover:bg-[#384574] text-white text-xs font-semibold border border-[#3b497a] transition-all">
        Exportar Juego 🚀
      </button>
    </div>
  </header>

  <!-- BODY IDE 3 COLUMNAS -->
  <div class="flex-grow flex overflow-hidden">

    <!-- COLUMNA IZQUIERDA: GESTOR DE PROYECTO & ESCENAS -->
    <aside class="w-64 bg-[#141829] border-r border-[#2a3356] flex flex-col justify-between p-3 flex-shrink-0 text-xs">
      <div class="space-y-4">
        <div class="font-extrabold text-slate-300 uppercase tracking-wider text-[11px] flex items-center justify-between pb-2 border-b border-[#2a3356]">
          <span>Gestor de Proyecto</span>
          <i data-lucide="folder-tree" class="w-4 h-4 text-[#4b6fff]"></i>
        </div>

        <div class="space-y-1">
          <div class="p-2 rounded-lg bg-[#4b6fff]/20 text-[#00d2ff] font-bold border border-[#4b6fff]/40 flex items-center justify-between cursor-pointer">
            <span class="flex items-center gap-2"><i data-lucide="film" class="w-4 h-4"></i> MainLevel.json</span>
            <span class="w-2 h-2 rounded-full bg-emerald-400"></span>
          </div>
          <div class="p-2 rounded-lg hover:bg-[#1f2642] text-slate-300 flex items-center justify-between cursor-pointer transition-all">
            <span class="flex items-center gap-2"><i data-lucide="film" class="w-4 h-4 text-slate-500"></i> BossArena.json</span>
          </div>
          <div class="p-2 rounded-lg hover:bg-[#1f2642] text-slate-300 flex items-center justify-between cursor-pointer transition-all">
            <span class="flex items-center gap-2"><i data-lucide="layers" class="w-4 h-4 text-purple-400"></i> ExternalLayout.json</span>
          </div>
        </div>

        <div class="pt-3 border-t border-[#2a3356] space-y-2">
          <div class="font-extrabold text-slate-300 uppercase tracking-wider text-[11px]">Extensiones & Comportamientos</div>
          <div class="p-2 rounded-lg bg-[#191f36] border border-[#2a3356] text-slate-300 flex items-center justify-between">
            <span>👾 Platformer Character</span>
            <span class="text-emerald-400 text-[10px]">Activo</span>
          </div>
          <div class="p-2 rounded-lg bg-[#191f36] border border-[#2a3356] text-slate-300 flex items-center justify-between">
            <span>🪙 Physics Engine 2D</span>
            <span class="text-emerald-400 text-[10px]">Activo</span>
          </div>
        </div>
      </div>

      <button onclick="alert('Añadiendo nueva escena al juego...')" class="w-full py-2 rounded-lg bg-[#1f2642] hover:bg-[#2a3356] text-slate-200 font-bold border border-[#35426e] transition-all flex items-center justify-center gap-1.5">
        <i data-lucide="plus" class="w-4 h-4 text-[#4b6fff]"></i> + Nueva Escena
      </button>
    </aside>

    <!-- CANVASES CENTRAL: EDITOR ESCENA 2D -->
    <main class="flex-grow bg-[#090b14] canvas-grid relative flex items-center justify-center overflow-hidden">
      
      <!-- GAME SCENE CANVAS WORKSPACE -->
      <div id="gameEditorCanvas" class="w-[720px] h-[480px] bg-[#111628] border-2 border-[#4b6fff] rounded-xl shadow-2xl relative overflow-hidden flex flex-col justify-between p-4">
        
        <!-- ELEMENTOS DE JUEGO 2D INTERACTIVOS -->
        <div class="absolute inset-0 p-6">
          
          <!-- FONDO ESPACIAL CON ESTRELLAS -->
          <div class="absolute inset-0 bg-gradient-to-b from-[#090b14] to-[#181d36] opacity-90"></div>

          <!-- JUGADOR NAVE CYBER 2D -->
          <div id="playerSprite" class="absolute left-24 top-40 w-14 h-14 bg-gradient-to-tr from-[#4b6fff] to-[#00d2ff] rounded-xl border-2 border-white shadow-lg shadow-[#00d2ff]/40 flex items-center justify-center text-xl cursor-move active:scale-110 transition-transform">
            🚀
          </div>

          <!-- PLATAFORMA BLOQUES -->
          <div class="absolute left-20 bottom-16 w-80 h-10 bg-emerald-600 rounded-lg border-2 border-emerald-400 flex items-center justify-center font-bold text-white text-xs shadow-lg">
            🧱 Platform Block (Solid)
          </div>

          <!-- MONEDAS COINS -->
          <div class="absolute left-40 top-24 w-8 h-8 bg-amber-400 rounded-full border-2 border-amber-200 flex items-center justify-center font-bold text-black text-xs animate-bounce shadow-lg shadow-amber-400/50">
            🪙
          </div>

          <div class="absolute left-60 top-24 w-8 h-8 bg-amber-400 rounded-full border-2 border-amber-200 flex items-center justify-center font-bold text-black text-xs animate-bounce shadow-lg shadow-amber-400/50" style="animation-delay: 0.2s">
            🪙
          </div>

          <!-- ENEMIGO OVNI -->
          <div class="absolute right-32 top-32 w-12 h-12 bg-red-600 rounded-full border-2 border-red-400 flex items-center justify-center text-lg animate-pulse shadow-lg shadow-red-500/50">
            👾
          </div>

        </div>

        <!-- BARRA FLOTANTE DE CONTROL DE ESCENA -->
        <div class="relative z-10 flex items-center justify-between bg-[#141829]/90 backdrop-blur border border-[#2a3356] rounded-lg p-2 text-xs">
          <div class="flex items-center gap-2 text-slate-300 font-mono">
            <span class="text-[#00d2ff] font-bold">Zoom: 100%</span>
            <span>|</span>
            <span>Grid Snap: 24px (ON)</span>
          </div>
          <div class="flex items-center gap-1.5">
            <button onclick="alert('Centrando cámara en la escena...')" class="px-2 py-1 rounded bg-[#1f2642] text-slate-300 hover:text-white">🎯 Centrar</button>
            <button onclick="alert('Limpiando canvas de juego...')" class="px-2 py-1 rounded bg-[#1f2642] text-slate-300 hover:text-white">🧹 Limpiar</button>
          </div>
        </div>

      </div>

    </main>

    <!-- COLUMNA DERECHA: GESTOR DE OBJETOS (OBJECTS MANAGER) -->
    <aside class="w-64 bg-[#141829] border-l border-[#2a3356] flex flex-col justify-between p-3 flex-shrink-0 text-xs">
      <div class="space-y-4">
        <div class="font-extrabold text-slate-300 uppercase tracking-wider text-[11px] flex items-center justify-between pb-2 border-b border-[#2a3356]">
          <span>Panel de Objetos (Objects)</span>
          <i data-lucide="box" class="w-4 h-4 text-[#00d2ff]"></i>
        </div>

        <div class="space-y-2">
          
          <div class="p-2.5 rounded-lg bg-[#1f2642] border border-[#35426e] flex items-center justify-between group hover:border-[#4b6fff] cursor-pointer transition-all">
            <div class="flex items-center gap-2">
              <span class="text-base">🚀</span>
              <div>
                <div class="font-bold text-white text-xs">Player_Ship</div>
                <div class="text-[10px] text-slate-400 font-mono">Sprite · 54x54</div>
              </div>
            </div>
            <i data-lucide="edit-3" class="w-3.5 h-3.5 text-slate-500 group-hover:text-white"></i>
          </div>

          <div class="p-2.5 rounded-lg bg-[#1f2642] border border-[#35426e] flex items-center justify-between group hover:border-[#4b6fff] cursor-pointer transition-all">
            <div class="flex items-center gap-2">
              <span class="text-base">🧱</span>
              <div>
                <div class="font-bold text-white text-xs">Platform_Grass</div>
                <div class="text-[10px] text-slate-400 font-mono">Tiled Sprite</div>
              </div>
            </div>
            <i data-lucide="edit-3" class="w-3.5 h-3.5 text-slate-500 group-hover:text-white"></i>
          </div>

          <div class="p-2.5 rounded-lg bg-[#1f2642] border border-[#35426e] flex items-center justify-between group hover:border-[#4b6fff] cursor-pointer transition-all">
            <div class="flex items-center gap-2">
              <span class="text-base">🪙</span>
              <div>
                <div class="font-bold text-white text-xs">Coin_Gold</div>
                <div class="text-[10px] text-slate-400 font-mono">Collectible Item</div>
              </div>
            </div>
            <i data-lucide="edit-3" class="w-3.5 h-3.5 text-slate-500 group-hover:text-white"></i>
          </div>

          <div class="p-2.5 rounded-lg bg-[#1f2642] border border-[#35426e] flex items-center justify-between group hover:border-[#4b6fff] cursor-pointer transition-all">
            <div class="flex items-center gap-2">
              <span class="text-base">👾</span>
              <div>
                <div class="font-bold text-white text-xs">Enemy_Alien</div>
                <div class="text-[10px] text-slate-400 font-mono">AI Behavior</div>
              </div>
            </div>
            <i data-lucide="edit-3" class="w-3.5 h-3.5 text-slate-500 group-hover:text-white"></i>
          </div>

        </div>
      </div>

      <button onclick="promptAddNewGameObject()" class="w-full py-2.5 rounded-lg bg-[#4b6fff] hover:bg-[#3557e6] text-white font-extrabold shadow-lg transition-all flex items-center justify-center gap-1.5">
        <i data-lucide="plus-circle" class="w-4 h-4"></i> + Añadir Nuevo Objeto
      </button>
    </aside>

  </div>

  <!-- FOOTER STATUSBAR -->
  <footer class="h-7 bg-[#141829] border-t border-[#2a3356] px-4 flex items-center justify-between text-[11px] text-slate-400 font-mono flex-shrink-0">
    <div class="flex items-center gap-3">
      <span class="text-emerald-400 font-bold">● GDevelop Engine 5.3.190</span>
      <span>FPS: 60.0</span>
      <span>Objetos en Escena: 12</span>
    </div>
    <div class="text-[#00d2ff] font-bold">Compilado por NEXUS AI</div>
  </footer>

  <script>
    lucide.createIcons();
    function launchGamePreview() {
      alert('🎮 ¡Iniciando vista previa del juego en tiempo real (60 FPS)! Usá las flechas del teclado para mover la nave.');
    }
    function promptAddNewGameObject() {
      const name = prompt('Nombre del nuevo objeto de juego 2D (Sprite / Partícula / Texto):', 'New_Enemy');
      if (name) alert('Objeto "' + name + '" añadido al Panel de Objetos de GDevelop.');
    }
  </script>
</body>
</html>`;
  } else if (p.includes('punk') || p.includes('rock') || p.includes('metal') || p.includes('cyberpunk') || p.includes('underground')) {
    html = `<!DOCTYPE html>
<html lang="es" class="dark">
<head>
  <meta charset="UTF-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
  <title>PUNK REBELLION · Sovereign Web</title>
  <script src="https://cdn.tailwindcss.com"></script>
  <script src="https://unpkg.com/lucide@latest"></script>
  <link href="https://fonts.googleapis.com/css2?family=Permanent+Marker&family=Space+Grotesk:wght@400;600;700;900&display=swap" rel="stylesheet"/>
  <style>
    body { font-family: 'Space Grotesk', sans-serif; background: #050505; color: #f4f4f5; }
    .punk-title { font-family: 'Permanent Marker', cursive; text-shadow: 3px 3px 0px #ff0055, -3px -3px 0px #00ffff; }
    .neon-border { border: 2px solid #ccff00; box-shadow: 0 0 15px rgba(204, 255, 0, 0.4); }
    .pink-border { border: 2px solid #ff0055; box-shadow: 0 0 15px rgba(255, 0, 85, 0.4); }
    .glitch-btn { background: #ccff00; color: #000; font-weight: 900; text-transform: uppercase; letter-spacing: 1px; }
    .glitch-btn:hover { background: #ff0055; color: #fff; box-shadow: 0 0 20px #ff0055; }
  </style>
</head>
<body class="min-h-screen p-6 flex flex-col justify-between selection:bg-[#ff0055] selection:text-white">

  <!-- HEADER PUNK REBELLION -->
  <header class="flex items-center justify-between pb-6 border-b-2 border-[#ccff00]/40">
    <div class="flex items-center gap-3">
      <div class="w-12 h-12 rounded-none bg-[#ff0055] text-black font-black flex items-center justify-center text-2xl rotate-[-6deg] shadow-[4px_4px_0px_#ccff00]">
        ⚡
      </div>
      <div>
        <h1 class="text-2xl font-black text-[#ccff00] uppercase tracking-wider">PUNK REBELLION</h1>
        <p class="text-xs text-[#ff0055] font-mono tracking-widest uppercase">ANARCHY IN THE CODE · NO RULES</p>
      </div>
    </div>

    <div class="flex items-center gap-3">
      <span class="px-3 py-1 bg-[#ff0055]/20 border border-[#ff0055] text-[#ff0055] text-xs font-mono font-bold animate-pulse">● EN VIVO P2P</span>
      <button onclick="playPunkSynth()" class="px-4 py-2 glitch-btn rounded-none border-2 border-black flex items-center gap-2">
        <i data-lucide="radio" class="w-4 h-4"></i> SINTETIZADOR PUNK
      </button>
    </div>
  </header>

  <!-- HERO SECTION PUNK -->
  <main class="py-8 space-y-8 flex-grow">
    
    <div class="bg-zinc-950 border-2 border-[#ccff00] p-8 relative overflow-hidden shadow-[8px_8px_0px_#ff0055]">
      <div class="absolute -right-10 -bottom-10 opacity-10 font-black text-9xl text-[#ccff00] select-none">PUNK</div>
      <div class="max-w-3xl space-y-4 relative z-10">
        <span class="px-3 py-1 bg-[#ccff00] text-black font-black text-xs uppercase tracking-widest">Sovereign Underground Engine</span>
        <h2 class="text-4xl md:text-5xl punk-title text-white">
          DESCENTRALIADIO. SIN CENSURA. PURO ROCK.
        </h2>
        <p class="text-sm text-zinc-300 leading-relaxed max-w-xl font-mono">
          Plataforma web punk autogestionada creada por IA autónoma en Rust + Axum. Música underground, conciertos DIY y transmisión libre.
        </p>
        <div class="pt-2 flex items-center gap-4">
          <button onclick="alert('🤘 ¡Entrando al Moshpit Digital!')" class="px-6 py-3 glitch-btn text-sm flex items-center gap-2">
            <i data-lucide="flame" class="w-4 h-4"></i> ENTRAR AL MOSHPIT
          </button>
          <button onclick="alert('⚡ Escuchando cassette underground...')" class="px-6 py-3 bg-zinc-900 text-[#00ffff] font-bold border-2 border-[#00ffff] hover:bg-[#00ffff] hover:text-black transition-all text-sm">
            🎧 CASSETTE DIGITAL
          </button>
        </div>
      </div>
    </div>

    <!-- REJILLA PUNK SECCIONES -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">

      <!-- CARD 1: CONCIERTOS UNDERGROUND -->
      <div class="bg-zinc-950 p-6 neon-border space-y-4 hover:translate-y-[-4px] transition-all">
        <div class="flex items-center justify-between text-[#ccff00]">
          <h3 class="font-black text-lg tracking-wide uppercase flex items-center gap-2">
            <i data-lucide="zap" class="w-5 h-5"></i> TOUR DATES 2026
          </h3>
          <span class="text-xs font-mono bg-zinc-900 px-2 py-1">SOLD OUT</span>
        </div>
        <div class="space-y-2 text-xs font-mono text-zinc-300">
          <div class="p-2 bg-zinc-900 border border-zinc-800 flex justify-between">
            <span class="text-white font-bold">LIMA PUNK FEST</span>
            <span class="text-[#ff0055]">18 SEP</span>
          </div>
          <div class="p-2 bg-zinc-900 border border-zinc-800 flex justify-between">
            <span class="text-white font-bold">BERLIN UNDERGROUND</span>
            <span class="text-[#ccff00]">22 SEP</span>
          </div>
          <div class="p-2 bg-zinc-900 border border-zinc-800 flex justify-between">
            <span class="text-white font-bold">TOKYO CYBERMOSHPIT</span>
            <span class="text-[#00ffff]">05 OCT</span>
          </div>
        </div>
        <button onclick="alert('🎟️ Comprando entradas con Crypto/Stripe...')" class="w-full py-2 bg-[#ff0055] text-white font-bold text-xs uppercase hover:bg-[#ccff00] hover:text-black transition-all">
          🎟️ COMPRAR ENTRADAS DIY
        </button>
      </div>

      <!-- CARD 2: REPRODUCTOR DE VINILOS / ÁLBUMES -->
      <div class="bg-zinc-950 p-6 pink-border space-y-4 hover:translate-y-[-4px] transition-all">
        <div class="flex items-center justify-between text-[#ff0055]">
          <h3 class="font-black text-lg tracking-wide uppercase flex items-center gap-2">
            <i data-lucide="disc" class="w-5 h-5 animate-spin" style="animation-duration: 4s;"></i> PUNK VINYLS
          </h3>
          <span class="text-xs font-mono text-[#ccff00]">33 RPM</span>
        </div>
        <div class="space-y-2 text-xs font-mono">
          <div class="p-2.5 bg-zinc-900 border border-zinc-800 flex items-center justify-between">
            <div>
              <div class="text-white font-bold">01. Rust Code Anarchy</div>
              <div class="text-[10px] text-zinc-500">Banda: Axum Overdrive</div>
            </div>
            <button onclick="playTrack('Rust Code Anarchy')" class="p-1.5 bg-[#ccff00] text-black font-bold"><i data-lucide="play" class="w-3.5 h-3.5"></i></button>
          </div>
          <div class="p-2.5 bg-zinc-900 border border-zinc-800 flex items-center justify-between">
            <div>
              <div class="text-white font-bold">02. No Master No Server</div>
              <div class="text-[10px] text-zinc-500">Banda: Sovereign Riot</div>
            </div>
            <button onclick="playTrack('No Master No Server')" class="p-1.5 bg-[#ff0055] text-white font-bold"><i data-lucide="play" class="w-3.5 h-3.5"></i></button>
          </div>
        </div>
        <div id="nowPlayingText" class="p-2 bg-zinc-900 border border-zinc-800 text-[11px] text-[#00ffff] font-mono text-center">
          ▶ Reproduciendo: NINGUNA PISTA
        </div>
      </div>

      <!-- CARD 3: MERCH PUNK STORE -->
      <div class="bg-zinc-950 p-6 border-2 border-[#00ffff] space-y-4 hover:translate-y-[-4px] transition-all">
        <div class="flex items-center justify-between text-[#00ffff]">
          <h3 class="font-black text-lg tracking-wide uppercase flex items-center gap-2">
            <i data-lucide="shopping-bag" class="w-5 h-5"></i> MERCH & STICKERS
          </h3>
          <span class="text-xs font-mono text-[#ff0055]">ENVÍO GLOBAL</span>
        </div>
        <div class="space-y-2 text-xs font-mono">
          <div class="p-2 bg-zinc-900 border border-zinc-800 flex justify-between items-center">
            <span class="text-white">👕 Polera "Code Anarchy"</span>
            <span class="text-[#ccff00] font-bold">$25</span>
          </div>
          <div class="p-2 bg-zinc-900 border border-zinc-800 flex justify-between items-center">
            <span class="text-white">🧥 Chaqueta Cuero Cyberpunk</span>
            <span class="text-[#ccff00] font-bold">$120</span>
          </div>
          <div class="p-2 bg-zinc-900 border border-zinc-800 flex justify-between items-center">
            <span class="text-white">🏷️ Pack Stickers Vinyl (10x)</span>
            <span class="text-[#ccff00] font-bold">$8</span>
          </div>
        </div>
        <button onclick="alert('🛒 Agregado al Carrito Punk')" class="w-full py-2 bg-[#00ffff] text-black font-black text-xs uppercase hover:bg-[#ccff00] transition-all">
          🛒 AÑADIR MERCH AL CARRITO
        </button>
      </div>

    </div>

  </main>

  <!-- FOOTER PUNK -->
  <footer class="pt-6 border-t-2 border-zinc-800 flex flex-col md:flex-row items-center justify-between text-xs text-zinc-500 font-mono gap-2">
    <div>NEXUS PUNK ENGINE v3.0 · NO CORPORATE BS</div>
    <div class="text-[#ccff00] font-bold">100% LIBRE & COMPILADO EN RUST AXUM</div>
  </footer>

  <script>
    lucide.createIcons();
    function playPunkSynth() {
      const ctx = new (window.AudioContext || window.webkitAudioContext)();
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();
      osc.type = 'sawtooth';
      osc.frequency.setValueAtTime(440, ctx.currentTime);
      osc.frequency.exponentialRampToValueAtTime(880, ctx.currentTime + 0.3);
      gain.gain.setValueAtTime(0.3, ctx.currentTime);
      gain.gain.exponentialRampToValueAtTime(0.01, ctx.currentTime + 0.3);
      osc.connect(gain);
      gain.connect(ctx.destination);
      osc.start();
      osc.stop(ctx.currentTime + 0.3);
    }
    function playTrack(title) {
      document.getElementById('nowPlayingText').innerText = '▶ Reproduciendo: ' + title;
      playPunkSynth();
    }
  </script>
</body>
</html>`;
  } else {
    // Generador General Adaptativo
    const title = uiSpec.app_title || `App ${userPrompt}`;
    html = `<!DOCTYPE html>
<html lang="es" class="dark">
<head>
  <meta charset="UTF-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
  <title>${title} · NEXUS App</title>
  <script src="https://cdn.tailwindcss.com"></script>
  <script src="https://unpkg.com/lucide@latest"></script>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800&display=swap" rel="stylesheet"/>
  <style>
    body { font-family: 'Inter', sans-serif; background: #080a0c; color: #f4f4f5; }
    .glass-card { background: rgba(15, 19, 25, 0.85); backdrop-filter: blur(12px); border: 1px solid rgba(255,255,255,0.08); }
    .gradient-text { background: linear-gradient(135deg, #10b981 0%, #06b6d4 100%); -webkit-background-clip: text; -webkit-text-fill-color: transparent; }
  </style>
</head>
<body class="min-h-screen p-6 flex flex-col justify-between">
  
  <header class="flex items-center justify-between pb-6 border-b border-zinc-800">
    <div class="flex items-center gap-3">
      <div class="w-10 h-10 rounded-xl bg-gradient-to-tr from-emerald-500 to-cyan-500 text-black font-extrabold flex items-center justify-center text-lg shadow-lg shadow-emerald-500/20">
        ⚡
      </div>
      <div>
        <h1 class="text-lg font-bold text-white tracking-tight">${title}</h1>
        <p class="text-xs text-zinc-400">Aplicación Generada en Vivo por NEXUS AI</p>
      </div>
    </div>
    
    <div class="flex items-center gap-2">
      <button onclick="alert('Acción ejecutada')" class="px-4 py-1.5 rounded-lg bg-emerald-500 hover:bg-emerald-400 text-black font-bold text-xs shadow-md transition-all">+ Nueva Acción</button>
    </div>
  </header>

  <main class="py-10 space-y-8 flex-grow">
    <div class="text-center space-y-4 max-w-2xl mx-auto">
      <span class="px-3 py-1 rounded-full bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 text-xs font-semibold uppercase tracking-wider inline-block">Plataforma Generada</span>
      <h2 class="text-3xl font-extrabold text-white tracking-tight">
        Bienvenido a <span class="gradient-text">${title}</span>
      </h2>
      <p class="text-xs text-zinc-400 leading-relaxed">
        Sistema optimizado e inferido autónomamente para satisfacer la solicitud: "${userPrompt}".
      </p>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-3 gap-5">
      ${(uiSpec.components || []).map(comp => `
        <div class="glass-card rounded-2xl p-5 space-y-3 hover:border-emerald-500/50 transition-all">
          <div class="flex items-center justify-between text-zinc-400">
            <span class="text-xs font-medium">${comp.title}</span>
            <i data-lucide="${comp.icon || 'sparkles'}" class="w-4 h-4 text-emerald-400"></i>
          </div>
          <div class="text-2xl font-extrabold text-white">${comp.value || '$0.00'}</div>
          <div class="text-[11px] text-emerald-400 font-mono">${comp.change || 'Activo'}</div>
        </div>
      `).join('')}
    </div>
  </main>

  <footer class="pt-6 border-t border-zinc-800 flex items-center justify-between text-xs text-zinc-500 font-mono">
    <span>NEXUS Engine v3.0 · Axum Tokio</span>
    <span class="text-emerald-400 font-bold">Compilado por IA</span>
  </footer>

  <script>lucide.createIcons();</script>
</body>
</html>`;
  }

  fs.writeFileSync(liveAppFile, html, 'utf-8');
}

async function generateUIWithLLM(userPrompt) {
  const systemMessage = `Tu objetivo es actuar como un Diseñador UI/UX Soberano de IA.
Dada una instrucción del usuario, debes responder ÚNICAMENTE con un objeto JSON sin formato markdown ni triple comilla.

Estructura estricta del JSON de salida:
{
  "app_title": "Nombre de la Aplicación",
  "theme": {
    "primary_color": "#00ff88",
    "background_color": "#030708",
    "text_color": "#e2e8f0",
    "font_family": "Inter, sans-serif"
  },
  "layout": {
    "type": "sidebar_header_content",
    "sidebar": {
      "brand": "NEXUS App",
      "items": [
        { "id": "dash", "label": "Inicio", "icon": "layout-dashboard", "active": true },
        { "id": "analytics", "label": "Métricas", "icon": "bar-chart", "active": false }
      ]
    },
    "header": {
      "title": "Título del Panel",
      "actions": [
        { "id": "btn_1", "label": "Acción Principal", "type": "button_primary" }
      ]
    }
  },
  "components": [
    {
      "id": "kpi_total_revenue",
      "type": "kpi_card",
      "title": "Ingresos Totales",
      "value": "$45,231.89",
      "change": "+20.1%",
      "icon": "rabbit",
      "trend": "up"
    },
    {
      "id": "comp_clock",
      "type": "clock_widget",
      "title": "Reloj Digital Soberano",
      "value": "13:45:00",
      "change": "En tiempo real",
      "trend": "up"
    },
    {
      "id": "comp_2",
      "type": "data_table",
      "title": "Tabla de Datos",
      "columns": [
        { "key": "id", "label": "ID" },
        { "key": "nombre", "label": "Nombre" }
      ]
    }
  ],
  "form_inputs": [
    { "id": "input_1", "label": "Campo Campo", "type": "text", "required": true }
  ]
}

REGLA: Devuelve SOLO el JSON raw sin caracteres extra. Puedes usar iconos de Lucide como "rabbit", "clock", "trending-up", "dollar-sign", "users", "shopping-cart".`;

  const response = await fetch("https://openrouter.ai/api/v1/chat/completions", {
    method: "POST",
    headers: {
      "Authorization": `Bearer ${OPENROUTER_API_KEY}`,
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      model: "meta-llama/llama-3.3-70b-instruct",
      messages: [
        { role: "system", content: systemMessage },
        { role: "user", content: `Diseña una interfaz de usuario completa para: "${userPrompt}"` }
      ],
      temperature: 0.3
    })
  });

  if (!response.ok) {
    throw new Error(`OpenRouter HTTP error ${response.status}`);
  }

  const json = await response.json();
  const rawText = json.choices[0]?.message?.content || "";

  const cleanJson = rawText.replace(/```json/g, '').replace(/```/g, '').trim();
  return JSON.parse(cleanJson);
}

function fallbackUIGenerator(promptStr) {
  const p = promptStr.toLowerCase();
  const isRabbit = p.includes('conejito') || p.includes('conejo') || p.includes('rabbit');

  return {
    app_title: `App: ${promptStr}`,
    theme: { primary_color: "#00ff88", background_color: "#030708", text_color: "#e2e8f0", font_family: "Inter" },
    layout: {
      type: "sidebar_header_content",
      sidebar: { brand: "NEXUS Engine", items: [{ id: "dash", label: "Inicio", icon: "layout-dashboard", active: true }] },
      header: { title: promptStr, actions: [{ id: "b1", label: "Guardar", type: "button_primary" }] }
    },
    components: [
      {
        id: "kpi_total_revenue",
        type: "kpi_card",
        title: "Ingresos Totales",
        value: "$45,231.89",
        change: "+20.1% respecto al mes pasado",
        icon: isRabbit ? "rabbit" : "trending-up",
        trend: "up"
      },
      {
        id: "kpi_active_subscriptions",
        type: "kpi_card",
        title: "Suscripciones Activas",
        value: "2,350",
        change: "+180.1% crecimiento",
        icon: "users",
        trend: "up"
      }
    ],
    form_inputs: []
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const input = process.argv[2];
  runUIExtractor(input).catch(err => {
    console.error('❌ [AGENT UI LLM] Error:', err);
    process.exit(1);
  });
}
