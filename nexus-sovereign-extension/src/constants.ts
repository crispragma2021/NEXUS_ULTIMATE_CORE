// ============================================================================
// 🔱 NEXUS CONSTANTS
// ============================================================================

// API de NEXUS Core
export const NEXUS_API_BASE = 'http://localhost:43210';

// Puertos del ecosistema
export const NEXUS_PORTS = {
  CORE: 43210,
  ENGINE_PURO: 43211,
  SANTUARIO: 5173,
  MCP_PROXY: 43212
} as const;

// Endpoints
export const NEXUS_ENDPOINTS = {
  HEALTH: '/api/health',
  CONSULTAR: '/api/consultar',
  MONOLOGUE: '/api/monologue',
  TTS_SPEAK: '/api/tts/speak',
  STT: '/api/stt',
  SCREENSHOTS: '/api/screenshots',
  DECISION: '/api/decision'
} as const;

// Colores del ecosistema
export const NEXUS_COLORS = {
  PRIMARY: '#ffd700',
  SECONDARY: '#00ff88',
  ACCENT: '#7aa2f7',
  WARNING: '#ffcc00',
  ERROR: '#ff5555',
  BG: '#0a0e1a',
  SURFACE: '#0d1230',
  TEXT: '#c0caf5'
} as const;

// Modelos disponibles para el Agentic Loop
export const AVAILABLE_MODELS = [
  { id: 'google/gemini-2.5-flash-preview-04-17', name: '⚡ Gemini Flash' },
  { id: 'google/gemini-2.5-pro-preview-04-17', name: '🧠 Gemini Pro' },
  { id: 'deepseek/deepseek-chat', name: '🔮 DeepSeek V3' },
  { id: 'deepseek/deepseek-r1', name: '🧬 DeepSeek R1' },
  { id: 'anthropic/claude-sonnet-20241022', name: '🎯 Claude Sonnet' },
  { id: 'openai/gpt-4o-2024-11-20', name: '🌟 GPT-4o' },
  { id: 'qwen/qwen-2.5-72b-instruct', name: '🐉 Qwen 2.5 72B' },
  { id: 'mistralai/mistral-large-2411', name: '🌪️ Mistral Large' }
] as const;
