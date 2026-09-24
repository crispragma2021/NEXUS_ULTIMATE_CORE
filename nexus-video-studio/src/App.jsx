import React, { useState, useRef, useEffect } from 'react';
import { 
  Play, Pause, Video, Music, Image as ImageIcon, Send, Loader2, 
  Layers, Settings, Sliders, Upload, MonitorPlay, Link as LinkIcon, 
  Download, Search, Sparkles, Wand2, RefreshCw, CheckCircle2, 
  Volume2, VolumeX, ZoomIn, ZoomOut, SkipBack, SkipForward, Repeat,
  Eye, EyeOff, Lock, Unlock, Magnet, Plus, Activity, Grid, Type,
  Trash2, ChevronUp, ChevronDown, X, ShieldAlert, Film
} from 'lucide-react';

function App() {
  const [status, setStatus] = useState("idle");
  const [downloading, setDownloading] = useState(false);
  const [downloadingUrl, setDownloadingUrl] = useState("");
  const [renderedVideoUrl, setRenderedVideoUrl] = useState(null);
  const [isVideoModalOpen, setIsVideoModalOpen] = useState(false);
  const [ytUrl, setYtUrl] = useState("");
  const [searchQuery, setSearchQuery] = useState("");
  const [isSearching, setIsSearching] = useState(false);
  const [searchResults, setSearchResults] = useState([]);
  const [logs, setLogs] = useState([]);
  const canvasRef = useRef(null);
  
  // Real Playback Time & Seeking State
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [volume, setVolume] = useState(1);
  const [isMuted, setIsMuted] = useState(false);
  const [zoomLevel, setZoomLevel] = useState(1);
  const [activeTrackName, setActiveTrackName] = useState("Sin audio seleccionado");

  // Dynamic Layer Management System (Vizzy Parity)
  const [layers, setLayers] = useState([
    { id: 'img-1', type: 'image', name: 'Background Image', visible: true, locked: false, color: '#00ffff' },
    { id: 'part-1', type: 'particles', name: 'Neon Particle Dust', visible: true, locked: false, color: '#38bdf8' },
    { id: 'fx-1', type: 'spectrum', name: 'Spectrum Wave FX', visible: true, locked: false, color: '#a855f7' },
    { id: 'txt-1', type: 'text', name: 'Title Overlay', visible: true, locked: false, color: '#f43f5e', text: 'NEXUS MUSIC', fontSize: 44, textColor: '#ffffff', y: 640, x: 640 },
    { id: 'aud-1', type: 'audio', name: 'Master Audio Track', visible: true, locked: false, color: '#10b981' }
  ]);
  const [selectedLayerId, setSelectedLayerId] = useState('txt-1');
  const [isAddLayerMenuOpen, setIsAddLayerMenuOpen] = useState(false);

  // Particles System Ref
  const particlesRef = useRef(
    Array.from({ length: 70 }).map(() => ({
      x: Math.random() * 1280,
      y: Math.random() * 720,
      radius: Math.random() * 3 + 1,
      alpha: Math.random() * 0.7 + 0.3,
      speedX: (Math.random() - 0.5) * 1.5,
      speedY: -Math.random() * 1.5 - 0.5,
      color: '#38bdf8'
    }))
  );

  const [isLooping, setIsLooping] = useState(true);
  const [isSnapped, setIsSnapped] = useState(true);

  const isLoopingRef = useRef(isLooping);
  useEffect(() => {
    isLoopingRef.current = isLooping;
  }, [isLooping]);

  // AI Image Gen State
  const [aiImagePrompt, setAiImagePrompt] = useState("");
  const [isGeneratingAiImage, setIsGeneratingAiImage] = useState(false);

  // Stock Image Search State
  const [imageSearchQuery, setImageSearchQuery] = useState("");
  const [imageSearchResults, setImageSearchResults] = useState([]);
  const [isSearchingImages, setIsSearchingImages] = useState(false);
  const [isImportingImage, setIsImportingImage] = useState(null);

  const handleSearchImages = async (term) => {
    const queryToUse = term || imageSearchQuery;
    if (!queryToUse) return;
    
    setIsSearchingImages(true);
    setLogs(prev => [...prev, `[SEARCH] Buscando imágenes HD libres de derechos: '${queryToUse}'...`]);
    
    try {
      const response = await fetch("http://localhost:43211/api/search-images", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ query: queryToUse })
      });
      const data = await response.json();
      
      if (data.status === "success") {
        setImageSearchResults(data.results || []);
        setLogs(prev => [...prev, `[SUCCESS] ${data.results.length} imágenes HD encontradas.`]);
      } else {
        setLogs(prev => [...prev, `[ERROR] No se pudieron obtener resultados de imágenes.`]);
      }
    } catch (err) {
      setLogs(prev => [...prev, `[ERROR] Error al conectar con el buscador de imágenes.`]);
    } finally {
      setIsSearchingImages(false);
    }
  };

  const handleSelectStockImage = async (item) => {
    setIsImportingImage(item.id);
    setLogs(prev => [...prev, `[IMPORT] Importando imagen Stock HD (${item.author}): ${item.title}...`]);
    
    try {
      const response = await fetch("http://localhost:43211/api/import-image-url", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ url: item.full_url })
      });
      const data = await response.json();
      
      if (data.status === "success" && data.media_url) {
        setImageUrl(data.media_url);
        setLogs(prev => [...prev, `[SUCCESS] ¡Imagen Stock HD establecida como fondo del lienzo!`]);
      } else {
        setImageUrl(item.full_url);
        setLogs(prev => [...prev, `[SUCCESS] Imagen establecida directamente desde CDN!`]);
      }
    } catch (err) {
      setImageUrl(item.full_url);
      setLogs(prev => [...prev, `[SUCCESS] Imagen establecida directamente desde CDN!`]);
    } finally {
      setIsImportingImage(null);
    }
  };

  // Preview State
  const [isPreviewLoading, setIsPreviewLoading] = useState(null);
  const [playingPreviewUrl, setPlayingPreviewUrl] = useState(null);
  const previewAudioRef = useRef(new Audio());

  // App State
  const [platform, setPlatform] = useState("youtube");
  const [waveColor, setWaveColor] = useState("#00ffff");
  const [sensitivity, setSensitivity] = useState(1.5);
  
  // Media State with LocalStorage Persistence
  const [audioUrl, setAudioUrlState] = useState(() => localStorage.getItem("nexus_active_audio") || null);
  const [imageUrl, setImageUrlState] = useState(() => localStorage.getItem("nexus_active_image") || null);
  const [isPlaying, setIsPlaying] = useState(false);

  const setAudioUrl = (url, name) => {
    setAudioUrlState(url);
    if (name) setActiveTrackName(name);
    if (url) localStorage.setItem("nexus_active_audio", url);
    else localStorage.removeItem("nexus_active_audio");
  };

  const setImageUrl = (url) => {
    setImageUrlState(url);
    if (url) localStorage.setItem("nexus_active_image", url);
    else localStorage.removeItem("nexus_active_image");
  };
  
  // Web Audio API refs
  const audioContextRef = useRef(null);
  const analyserRef = useRef(null);
  const sourceRef = useRef(null);
  const dataArrayRef = useRef(null);
  const animationRef = useRef(null);
  const audioElRef = useRef(null);

  // Preview Audio setup
  useEffect(() => {
    const audio = previewAudioRef.current;
    audio.onended = () => setPlayingPreviewUrl(null);
    return () => {
      audio.pause();
    };
  }, []);

  // Ensure Web Audio Graph is permanently connected
  const ensureAudioInitialized = () => {
    if (!audioContextRef.current) {
      audioContextRef.current = new (window.AudioContext || window.webkitAudioContext)();
      analyserRef.current = audioContextRef.current.createAnalyser();
      analyserRef.current.fftSize = 256;
      const bufferLength = analyserRef.current.frequencyBinCount;
      dataArrayRef.current = new Uint8Array(bufferLength);
    }
    
    if (!audioElRef.current) {
      audioElRef.current = new Audio();
      audioElRef.current.crossOrigin = "anonymous";
      sourceRef.current = audioContextRef.current.createMediaElementSource(audioElRef.current);
      sourceRef.current.connect(analyserRef.current);
      analyserRef.current.connect(audioContextRef.current.destination);
    }
    return audioElRef.current;
  };

  // Layer Actions with Smart Position Offset
  const handleAddLayer = (type) => {
    let newLayer = null;
    const timestamp = Date.now();

    if (type === 'text') {
      const existingTexts = layers.filter(l => l.type === 'text');
      const defaultY = existingTexts.length === 0 ? 640 : (existingTexts.length === 1 ? 160 : 360);
      newLayer = { 
        id: `txt-${timestamp}`, 
        type: 'text', 
        name: `Título ${existingTexts.length + 1}`, 
        visible: true, 
        locked: false, 
        color: '#f43f5e', 
        text: 'NEXUS STUDIO', 
        fontSize: 42, 
        textColor: '#ffffff',
        y: defaultY,
        x: 640
      };
    } else if (type === 'particles') {
      newLayer = { 
        id: `part-${timestamp}`, 
        type: 'particles', 
        name: 'Efecto Partículas', 
        visible: true, 
        locked: false, 
        color: '#38bdf8' 
      };
    } else if (type === 'spectrum') {
      newLayer = { 
        id: `fx-${timestamp}`, 
        type: 'spectrum', 
        name: 'Espectro de Onda', 
        visible: true, 
        locked: false, 
        color: '#a855f7' 
      };
    } else if (type === 'image') {
      newLayer = { 
        id: `img-${timestamp}`, 
        type: 'image', 
        name: 'Fondo Adicional', 
        visible: true, 
        locked: false, 
        color: '#00ffff' 
      };
    } else if (type === 'watermark') {
      newLayer = { 
        id: `wm-${timestamp}`, 
        type: 'watermark', 
        name: 'Marca de Agua', 
        visible: true, 
        locked: false, 
        color: '#10b981', 
        text: 'NEXUS PRODUCTIONS' 
      };
    }

    if (newLayer) {
      setLayers(prev => [newLayer, ...prev]);
      setSelectedLayerId(newLayer.id);
      setIsAddLayerMenuOpen(false);
      setLogs(prev => [...prev, `[LAYER] Nueva capa agregada: '${newLayer.name}'`]);
    }
  };

  const handleToggleLayerVisibility = (id) => {
    setLayers(prev => prev.map(l => l.id === id ? { ...l, visible: !l.visible } : l));
  };

  const handleToggleLayerLock = (id) => {
    setLayers(prev => prev.map(l => l.id === id ? { ...l, locked: !l.locked } : l));
  };

  const handleDeleteLayer = (id) => {
    if (id === 'aud-1') return; // Cannot delete master audio
    setLayers(prev => prev.filter(l => l.id !== id));
    if (selectedLayerId === id) {
      setSelectedLayerId('txt-1');
    }
    setLogs(prev => [...prev, `[LAYER] Capa eliminada del proyecto.`]);
  };

  const handleUpdateSelectedLayer = (updates) => {
    setLayers(prev => prev.map(l => l.id === selectedLayerId ? { ...l, ...updates } : l));
  };

  const handleTogglePreview = async (webpageUrl) => {
    const audio = previewAudioRef.current;

    if (playingPreviewUrl === webpageUrl) {
      audio.pause();
      setPlayingPreviewUrl(null);
      return;
    }

    setIsPreviewLoading(webpageUrl);
    setLogs(prev => [...prev, `[PREVIEW] Obteniendo stream de prueba para: ${webpageUrl}`]);

    try {
      const response = await fetch("http://localhost:43211/api/preview", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ url: webpageUrl })
      });
      const data = await response.json();

      if (data.status === "success" && data.preview_url) {
        audio.src = data.preview_url;
        audio.play();
        setPlayingPreviewUrl(webpageUrl);
        setLogs(prev => [...prev, `[PREVIEW] Reproduciendo vista previa...`]);
      } else {
        setLogs(prev => [...prev, `[ERROR] No se pudo obtener la vista previa.`]);
      }
    } catch (err) {
      setLogs(prev => [...prev, `[ERROR] Error al conectar con el servidor de previsualización.`]);
    } finally {
      setIsPreviewLoading(null);
    }
  };

  // Generate AI Image via Rust Backend
  const handleGenerateAiImage = async () => {
    if (!aiImagePrompt) return;

    setIsGeneratingAiImage(true);
    setLogs(prev => [...prev, `[AI IMAGE] Generando arte personalizado: '${aiImagePrompt}'...`]);

    const width = platform === 'tiktok' ? 720 : 1280;
    const height = platform === 'tiktok' ? 1280 : 720;

    try {
      const response = await fetch("http://localhost:43211/api/generate-image", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ prompt: aiImagePrompt, width, height })
      });
      const data = await response.json();

      if (data.status === "success" && data.image_url) {
        setImageUrl(data.image_url);
        setLogs(prev => [...prev, `[SUCCESS] Imagen personalizada generada e importada!`]);
      } else {
        setLogs(prev => [...prev, `[ERROR] ${data.message}`]);
      }
    } catch (err) {
      setLogs(prev => [...prev, `[ERROR] Error de conexión con el motor de IA.`]);
    } finally {
      setIsGeneratingAiImage(false);
    }
  };

  // Initialize Web Audio API and real audio player events
  useEffect(() => {
    if (!audioUrl) return;

    const audio = ensureAudioInitialized();
    if (audio.src !== audioUrl) {
      audio.src = audioUrl;
      audio.load();
    }

    audio.onloadedmetadata = () => {
      setDuration(audio.duration || 0);
    };

    audio.ontimeupdate = () => {
      setCurrentTime(audio.currentTime || 0);
    };

    audio.onended = () => {
      if (isLoopingRef.current) {
        audio.currentTime = 0;
        audio.play().catch(e => console.error("Loop error:", e));
      } else {
        setIsPlaying(false);
        setCurrentTime(0);
      }
    };

    audio.onerror = (e) => {
      console.error("Audio element error:", e);
      setLogs(prev => [...prev, `[ERROR] Error al cargar la pista de audio.`]);
      setIsPlaying(false);
    };
  }, [audioUrl]);

  const imageCacheRef = useRef({});

  // Canvas Multi-Layer Render Loop
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');

    const draw = () => {
      animationRef.current = requestAnimationFrame(draw);
      const width = canvas.width;
      const height = canvas.height;

      // Base Canvas Fill
      ctx.fillStyle = '#09090b';
      ctx.fillRect(0, 0, width, height);

      // Bass Frequency Beat Detection for Bass Bounce Effect
      let bassBounce = 0;
      if (analyserRef.current && dataArrayRef.current && isPlaying) {
        analyserRef.current.getByteFrequencyData(dataArrayRef.current);
        let bassSum = 0;
        for (let i = 0; i < 8; i++) {
          bassSum += dataArrayRef.current[i];
        }
        const avgBass = bassSum / 8;
        bassBounce = (avgBass / 255) * 0.15; // Pulse scale up to +15% on heavy bass kick
      }

      // Render layers from bottom to top according to layer order
      // (reverse array so top layer renders on top)
      const renderOrderLayers = [...layers].reverse();

      renderOrderLayers.forEach(layer => {
        if (!layer.visible) return;

        // 1. Image Layer
        if (layer.type === 'image') {
          const targetUrl = layer.url || imageUrl;
          if (targetUrl) {
            if (!imageCacheRef.current[targetUrl]) {
              const img = new Image();
              img.crossOrigin = "anonymous";
              img.src = targetUrl;
              img.onload = () => {
                imageCacheRef.current[targetUrl + '_loaded'] = true;
              };
              imageCacheRef.current[targetUrl] = img;
            }
            
            const bgImg = imageCacheRef.current[targetUrl];
            const isLoaded = imageCacheRef.current[targetUrl + '_loaded'];
            
            if (bgImg && isLoaded) {
              ctx.save();
              ctx.globalAlpha = layer.opacity !== undefined ? layer.opacity : 1.0;
              const bassPulse = (layer.bassBounce !== false) ? bassBounce : 0;
              const kenBurns = (layer.kenBurns !== false) ? (Math.sin(Date.now() / 2500) * 0.04) : 0;
              const panX = (layer.kenBurns !== false) ? (Math.cos(Date.now() / 3500) * 12) : 0;
              const panY = (layer.kenBurns !== false) ? (Math.sin(Date.now() / 3500) * 8) : 0;
              
              const scaleFactor = (layer.scale !== undefined ? layer.scale : 1.0) * (1.0 + bassPulse + kenBurns);
              const scale = Math.max(width / bgImg.width, height / bgImg.height) * scaleFactor;
              const x = (width / 2) - (bgImg.width / 2) * scale + panX;
              const y = (height / 2) - (bgImg.height / 2) * scale + panY;
              ctx.drawImage(bgImg, x, y, bgImg.width * scale, bgImg.height * scale);
              
              const grad = ctx.createLinearGradient(0, height * 0.6, 0, height);
              grad.addColorStop(0, 'rgba(0,0,0,0)');
              grad.addColorStop(1, 'rgba(0,0,0,0.5)');
              ctx.fillStyle = grad;
              ctx.fillRect(0, 0, width, height);
              ctx.restore();
            }
          }
        }

        // 2. Particle Dust Layer
        if (layer.type === 'particles') {
          particlesRef.current.forEach(p => {
            p.x += p.speedX;
            p.y += p.speedY;
            if (p.y < 0) p.y = height;
            if (p.x < 0) p.x = width;
            if (p.x > width) p.x = 0;

            ctx.beginPath();
            ctx.arc(p.x, p.y, p.radius, 0, Math.PI * 2);
            ctx.fillStyle = p.color || '#38bdf8';
            ctx.globalAlpha = p.alpha;
            ctx.shadowBlur = 10;
            ctx.shadowColor = p.color || '#38bdf8';
            ctx.fill();
            ctx.shadowBlur = 0;
            ctx.globalAlpha = 1.0;
          });
        }

        // 3. Spectrum Visualizer Layer (Horizontal Wave or Circular Radial Ring)
        if (layer.type === 'spectrum' && analyserRef.current && dataArrayRef.current && isPlaying) {
          analyserRef.current.getByteFrequencyData(dataArrayRef.current);
          
          const bufferLength = analyserRef.current.frequencyBinCount;
          const hasAudioActivity = dataArrayRef.current.some(val => val > 0);

          if (hasAudioActivity) {
            ctx.save();
            if (layer.style === 'circle') {
              const centerX = width / 2;
              const centerY = height / 2;
              const radius = 140;
              ctx.beginPath();
              ctx.strokeStyle = waveColor;
              ctx.lineWidth = 4;
              ctx.shadowBlur = 25;
              ctx.shadowColor = waveColor;

              for (let i = 0; i < bufferLength; i += 2) {
                const angle = (i / bufferLength) * Math.PI * 2;
                const amplitude = (dataArrayRef.current[i] / 255) * 60 * sensitivity;
                const r = radius + amplitude;
                const x = centerX + Math.cos(angle) * r;
                const y = centerY + Math.sin(angle) * r;
                if (i === 0) ctx.moveTo(x, y);
                else ctx.lineTo(x, y);
              }
              ctx.closePath();
              ctx.stroke();
            } else {
              const barWidth = (width / bufferLength) * 2.5;
              let x = 0;
              const centerY = height / 2;

              ctx.beginPath();
              ctx.moveTo(0, centerY);

              for (let i = 0; i < bufferLength; i++) {
                const rawBarHeight = dataArrayRef.current[i];
                const barHeight = rawBarHeight * sensitivity;
                
                ctx.lineTo(x, centerY - barHeight / 2);
                x += barWidth;
              }
              
              ctx.strokeStyle = waveColor;
              ctx.lineWidth = 4;
              ctx.lineCap = 'round';
              ctx.lineJoin = 'round';
              
              ctx.shadowBlur = 20;
              ctx.shadowColor = waveColor;
              ctx.stroke();
            }
            ctx.restore();
          }
        }

        // 4. Text Title Overlay Layer (Position-Aware)
        if (layer.type === 'text' && layer.text) {
          ctx.save();
          ctx.font = `bold ${layer.fontSize || 44}px Inter, sans-serif`;
          ctx.fillStyle = layer.textColor || '#ffffff';
          ctx.textAlign = 'center';
          ctx.shadowColor = layer.color || '#f43f5e';
          ctx.shadowBlur = 25;
          const targetX = layer.x !== undefined ? layer.x : (width / 2);
          const targetY = layer.y !== undefined ? layer.y : (height - 80);
          ctx.fillText(layer.text, targetX, targetY);
          ctx.restore();
        }

        // 5. Watermark Layer
        if (layer.type === 'watermark' && layer.text) {
          ctx.save();
          ctx.font = `600 14px Inter, sans-serif`;
          ctx.fillStyle = 'rgba(255, 255, 255, 0.7)';
          ctx.textAlign = 'right';
          ctx.shadowColor = 'rgba(0,0,0,0.8)';
          ctx.shadowBlur = 5;
          ctx.fillText(layer.text, width - 30, 40);
          ctx.restore();
        }

      });
    };

    draw();
    
    return () => {
      cancelAnimationFrame(animationRef.current);
    };
  }, [imageUrl, waveColor, sensitivity, isPlaying, layers]);

  const togglePlay = () => {
    if (!audioUrl) {
      alert("Por favor selecciona o descarga una pista de audio primero.");
      return;
    }
    
    const audio = ensureAudioInitialized();

    if (previewAudioRef.current) {
      previewAudioRef.current.pause();
      setPlayingPreviewUrl(null);
    }

    if (audioContextRef.current && audioContextRef.current.state === 'suspended') {
      audioContextRef.current.resume();
    }
    
    if (isPlaying) {
      audio.pause();
      setIsPlaying(false);
    } else {
      if (audio.src !== audioUrl) {
        audio.src = audioUrl;
        audio.load();
      }

      const playPromise = audio.play();
      if (playPromise !== undefined) {
        playPromise
          .then(() => {
            setIsPlaying(true);
            setLogs(prev => [...prev, `[PLAYBACK] Reproduciendo audio master en el Timeline.`]);
          })
          .catch(err => {
            console.error("Audio playback error:", err);
            setLogs(prev => [...prev, `[ERROR] No se pudo reproducir el audio: ${err.message}`]);
            setIsPlaying(false);
          });
      }
    }
  };

  const handleSeek = (e) => {
    if (!audioUrl) return;
    const audio = ensureAudioInitialized();
    if (!duration) return;
    
    const rect = e.currentTarget.getBoundingClientRect();
    const clickX = Math.max(0, Math.min(e.clientX - rect.left, rect.width));
    const percentage = clickX / rect.width;
    const newTime = percentage * duration;
    audio.currentTime = newTime;
    setCurrentTime(newTime);
  };

  const stepTime = (delta) => {
    if (!audioUrl) return;
    const audio = ensureAudioInitialized();
    if (!duration) return;

    const newTime = Math.max(0, Math.min(duration, audio.currentTime + delta));
    audio.currentTime = newTime;
    setCurrentTime(newTime);
  };

  const handleVolumeChange = (e) => {
    const val = parseFloat(e.target.value);
    setVolume(val);
    const audio = ensureAudioInitialized();
    audio.volume = val;
    if (val === 0) setIsMuted(true);
    else setIsMuted(false);
  };

  const toggleMute = () => {
    const audio = ensureAudioInitialized();
    audio.muted = !isMuted;
    setIsMuted(!isMuted);
  };

  const formatTimeDetailed = (seconds) => {
    if (!seconds || isNaN(seconds)) return "00:00.00";
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    const ms = Math.floor((seconds % 1) * 100);
    return `${mins < 10 ? '0' : ''}${mins}:${secs < 10 ? '0' : ''}${secs}.${ms < 10 ? '0' : ''}${ms}`;
  };

  const handleImageUpload = async (e) => {
    const file = e.target.files[0];
    if (!file) return;

    const localUrl = URL.createObjectURL(file);
    setImageUrl(localUrl);

    // Upload to backend workspace so FFmpeg can render it!
    const reader = new FileReader();
    reader.onload = async (evt) => {
      const base64Data = evt.target.result;
      try {
        const response = await fetch("http://localhost:43211/api/upload-media", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ file_name: file.name, base64_data: base64Data })
        });
        const data = await response.json();
        if (data.status === "success" && data.media_url) {
          setImageUrl(data.media_url);
          setLogs(prev => [...prev, `[UPLOAD] Imagen local subida y lista para renderizado FFmpeg!`]);
        }
      } catch (err) {
        console.error("Upload error:", err);
      }
    };
    reader.readAsDataURL(file);
  };

  const handleAudioUpload = async (e) => {
    const file = e.target.files[0];
    if (!file) return;

    const localUrl = URL.createObjectURL(file);
    setAudioUrl(localUrl, file.name);
    setCurrentTime(0);

    // Upload to backend workspace so FFmpeg can render it!
    const reader = new FileReader();
    reader.onload = async (evt) => {
      const base64Data = evt.target.result;
      try {
        const response = await fetch("http://localhost:43211/api/upload-media", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ file_name: file.name, base64_data: base64Data })
        });
        const data = await response.json();
        if (data.status === "success" && data.media_url) {
          setAudioUrl(data.media_url, file.name);
          setLogs(prev => [...prev, `[UPLOAD] Audio local subido y listo para renderizado FFmpeg!`]);
        }
      } catch (err) {
        console.error("Upload error:", err);
      }
    };
    reader.readAsDataURL(file);
  };

  const handleSearch = async (term) => {
    const queryToUse = term || searchQuery;
    if (!queryToUse) return;
    
    setIsSearching(true);
    setLogs(prev => [...prev, `[SEARCH] Buscando temas sin copyright: '${queryToUse}'...`]);
    
    try {
      const response = await fetch("http://localhost:43211/api/search", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ query: queryToUse })
      });
      const data = await response.json();
      
      if (data.status === "success") {
        setSearchResults(data.results || []);
        setLogs(prev => [...prev, `[SUCCESS] ${data.results.length} temas encontrados.`]);
      } else {
        setLogs(prev => [...prev, `[ERROR] No se pudieron obtener resultados de búsqueda.`]);
      }
    } catch (err) {
      setLogs(prev => [...prev, `[ERROR] Error al conectar con el servidor de búsqueda.`]);
    } finally {
      setIsSearching(false);
    }
  };

  const handleDownloadTrack = async (targetUrl, title) => {
    if (previewAudioRef.current) {
      previewAudioRef.current.pause();
      setPlayingPreviewUrl(null);
    }

    setDownloading(true);
    setDownloadingUrl(targetUrl);
    setLogs(prev => [...prev, `[DOWNLOAD] Descargando audio completo: ${targetUrl}`]);
    
    try {
      const response = await fetch("http://localhost:43211/api/download", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ url: targetUrl })
      });
      const data = await response.json();
      
      if (data.status === "success" && data.media_url) {
        setLogs(prev => [...prev, `[SUCCESS] Audio importado al Timeline!`]);
        setAudioUrl(data.media_url, title || "Audio importado de YouTube");
        setCurrentTime(0);
      } else {
        setLogs(prev => [...prev, `[ERROR] ${data.message}`]);
      }
    } catch (err) {
      setLogs(prev => [...prev, `[ERROR] Falló conexión con el servidor de descarga.`]);
    } finally {
      setDownloading(false);
      setDownloadingUrl("");
    }
  };

  const handleExport = async () => {
    if (!imageUrl || !audioUrl) {
      alert("Por favor selecciona o genera una imagen y un audio primero.");
      return;
    }
    
    setStatus("processing");
    setRenderedVideoUrl(null);
    setIsVideoModalOpen(true);
    setLogs(prev => [...prev, `[INIT] Iniciando compilación de video MP4 con FFmpeg...`]);
    setLogs(prev => [...prev, `[FFMPEG] Parámetros: Color=${waveColor}, Platform=${platform}`]);
    
    try {
      const response = await fetch("http://localhost:43211/api/generate", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ 
          prompt: aiImagePrompt || "Custom Visualizer Video", 
          tipo: platform,
          color: waveColor,
          image_path: imageUrl,
          audio_path: audioUrl,
          duration: duration ? Math.round(duration) : null
        })
      });
      const data = await response.json();
      setLogs(prev => [...prev, `[SERVER] ${data.message}`]);
      
      if (data.video_url) {
        setRenderedVideoUrl(data.video_url);
      }
      
      setTimeout(() => setLogs(prev => [...prev, `[NEXUS] FFMPEG procesando imágenes y espectro de audio...`]), 2000);
      setTimeout(() => {
        setLogs(prev => [...prev, `[DONE] ¡Video MP4 generado con éxito! Reproductor listo.`]);
        setStatus("done");
      }, 5000);
    } catch (error) {
      setLogs(prev => [...prev, `[ERROR] Falló conexión al motor FFMPEG.`]);
      setStatus("error");
    }
  };

  const handleSaveVideoToDisk = async (url) => {
    const targetUrl = url || renderedVideoUrl;
    if (!targetUrl) return;
    setLogs(prev => [...prev, `[DOWNLOAD] Guardando archivo .mp4 en tu computadora...`]);
    try {
      const response = await fetch(targetUrl);
      const blob = await response.blob();
      const blobUrl = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = blobUrl;
      const filename = `nexus_video_${Date.now()}.mp4`;
      link.setAttribute('download', filename);
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      window.URL.revokeObjectURL(blobUrl);
      setLogs(prev => [...prev, `[SUCCESS] Archivo ${filename} guardado directamente en tu PC!`]);
    } catch (err) {
      console.error("Direct download failed, falling back", err);
      const link = document.createElement('a');
      link.href = targetUrl;
      link.setAttribute('download', 'nexus_video.mp4');
      link.click();
    }
  };

  const presetCategories = ["NCS Slap House", "Lofi Girl Study", "NEFFEX Royalty Free", "StreamBeats Harris Heller", "Magic Music Slap House"];
  const presetImageCategories = ["Cyberpunk Neon", "Lofi Anime", "Space Galaxy", "Nature 4K", "Abstract Dark", "Futuristic City"];

  // Calculate ticks for time ruler based on duration
  const renderRulerTicks = () => {
    const totalSecs = duration || 30;
    const ticksCount = 10;
    const ticks = [];
    for (let i = 0; i <= ticksCount; i++) {
      const sec = (totalSecs / ticksCount) * i;
      const mins = Math.floor(sec / 60);
      const secs = Math.floor(sec % 60);
      const timeStr = `${mins}:${secs < 10 ? '0' : ''}${secs}`;
      ticks.push(
        <div key={i} className="flex flex-col items-center select-none" style={{ left: `${(i / ticksCount) * 100}%`, position: 'absolute' }}>
          <div className="h-2 w-px bg-zinc-600"></div>
          <span className="text-[9px] text-zinc-500 font-mono mt-0.5">{timeStr}</span>
        </div>
      );
    }
    return ticks;
  };

  const selectedLayer = layers.find(l => l.id === selectedLayerId);

  return (
    <div className="min-h-screen bg-[#050505] text-zinc-200 font-sans overflow-hidden flex flex-col selection:bg-cyan-500/30 relative">
      
      {/* Interactive Rendered Video Preview Modal */}
      {isVideoModalOpen && (
        <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-md flex items-center justify-center p-6 animate-in fade-in duration-300">
           <div className="bg-[#0b0b10] border border-cyan-500/30 rounded-2xl max-w-3xl w-full p-6 shadow-[0_0_50px_rgba(6,182,212,0.3)] space-y-5 relative">
              <div className="flex items-center justify-between border-b border-white/10 pb-3">
                 <div className="flex items-center gap-2">
                    <Film size={20} className="text-cyan-400" />
                    <h2 className="font-semibold text-white text-base">Video MP4 Renderizado (FFmpeg)</h2>
                 </div>
                 <button 
                   onClick={() => setIsVideoModalOpen(false)}
                   className="p-1 hover:bg-zinc-800 rounded-lg text-zinc-400 hover:text-white transition-colors"
                 >
                    <X size={18} />
                 </button>
              </div>

              {status === 'processing' ? (
                <div className="h-80 flex flex-col items-center justify-center space-y-4 text-center">
                   <Loader2 size={40} className="text-cyan-400 animate-spin" />
                   <div>
                      <h3 className="text-sm font-semibold text-white">Sintetizando Video y Espectro Audio...</h3>
                      <p className="text-xs text-zinc-400 max-w-sm mt-1">El motor FFMPEG de Rust está combinando tus imágenes de fondo, audio y efectos.</p>
                   </div>
                </div>
              ) : (
                <div className="space-y-5">
                   <div className="aspect-video bg-black rounded-xl overflow-hidden border border-white/10 shadow-2xl relative">
                      <video 
                        src={renderedVideoUrl} 
                        controls 
                        autoPlay 
                        loop 
                        className="w-full h-full object-contain"
                      />
                   </div>

                   <div className="flex items-center justify-between pt-2">
                      <div className="flex items-center gap-2 text-xs text-emerald-400 font-semibold">
                         <CheckCircle2 size={16} />
                         <span>¡Video listo para guardar en tu PC!</span>
                      </div>

                      <div className="flex items-center gap-3">
                         <button 
                           onClick={() => setIsVideoModalOpen(false)}
                           className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs rounded-lg font-medium transition-colors"
                         >
                            Cerrar
                         </button>
                         <button
                           onClick={() => handleSaveVideoToDisk(renderedVideoUrl)}
                           className="px-5 py-2 bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 text-white font-semibold text-xs rounded-lg flex items-center gap-2 transition-all shadow-[0_0_20px_rgba(16,185,129,0.4)]"
                         >
                           <Download size={15} />
                           Descargar MP4 a mi PC
                         </button>
                      </div>
                   </div>
                </div>
              )}
           </div>
        </div>
      )}

      {/* Top Navbar */}
      <header className="h-14 border-b border-white/10 bg-black/50 backdrop-blur-md flex items-center justify-between px-6 z-50">
        <div className="flex items-center gap-3">
           <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-cyan-500 to-purple-600 flex items-center justify-center shadow-[0_0_15px_rgba(6,182,212,0.5)]">
              <MonitorPlay size={18} className="text-white" />
           </div>
           <h1 className="font-semibold tracking-wide text-white">NEXUS <span className="text-zinc-500 font-light">Vizzy Studio</span></h1>
        </div>
        
        <div className="flex items-center gap-4">
           <div className="flex bg-zinc-900 rounded-lg p-1 border border-white/5">
             {['youtube', 'tiktok', 'twitch'].map(p => (
               <button 
                 key={p}
                 onClick={() => setPlatform(p)}
                 className={`px-4 py-1 rounded-md text-xs font-medium uppercase transition-all duration-300 ${
                   platform === p ? 'bg-cyan-500/20 text-cyan-300' : 'text-zinc-500 hover:text-zinc-300'
                 }`}
               >
                 {p}
               </button>
             ))}
           </div>
           
           {renderedVideoUrl && status === 'done' ? (
             <div className="flex items-center gap-2">
                <button 
                  onClick={() => setIsVideoModalOpen(true)}
                  className="bg-zinc-800 hover:bg-zinc-700 text-cyan-300 border border-cyan-500/30 px-3 py-1.5 rounded-lg text-xs font-semibold flex items-center gap-1.5 transition-all"
                >
                   <Film size={14} /> Ver Video
                </button>

                <button
                  onClick={() => handleSaveVideoToDisk(renderedVideoUrl)}
                  className="bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 text-white px-5 py-1.5 rounded-lg text-sm font-semibold flex items-center gap-2 transition-all duration-300 shadow-[0_0_20px_rgba(16,185,129,0.5)]"
                >
                  <Download size={16} />
                  Descargar MP4
                </button>
             </div>
           ) : (
             <button 
               onClick={handleExport}
               disabled={status === 'processing'}
               className="bg-gradient-to-r from-cyan-600 to-blue-600 hover:from-cyan-500 hover:to-blue-500 text-white px-5 py-1.5 rounded-lg text-sm font-medium flex items-center gap-2 transition-all duration-300 shadow-[0_0_15px_rgba(6,182,212,0.3)] disabled:opacity-50"
             >
               {status === 'processing' ? <Loader2 className="animate-spin" size={16} /> : <Film size={16} />}
               Crear Video (FFMPEG)
             </button>
           )}
        </div>
      </header>

      {/* Main Workspace */}
      <main className="flex-1 flex overflow-hidden">
        
        {/* Left Panel - Tools */}
        <aside className="w-96 border-r border-white/5 bg-[#0a0a0c] flex flex-col z-10">
          <div className="p-4 border-b border-white/5 flex items-center gap-2 text-sm font-semibold text-zinc-300">
            <Layers size={16} className="text-purple-400" />
            Herramientas & Creador Personalizado
          </div>
          
          <div className="flex-1 overflow-y-auto p-3 space-y-4">
            
            {/* Custom AI Image Prompt Input */}
            <div className="bg-gradient-to-br from-cyan-950/40 to-blue-950/30 border border-cyan-500/30 rounded-xl p-3.5 space-y-3">
               <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Wand2 size={16} className="text-cyan-400" />
                    <span className="text-xs font-semibold text-cyan-200">Generar Fondo Personalizado (IA)</span>
                  </div>
               </div>
               
               <p className="text-[11px] text-zinc-400">Escribe exactamente lo que quieres que aparezca en tu fondo:</p>

               <div className="space-y-2">
                  <textarea 
                    placeholder="Ej: Un guerrero robot en una ciudad cyberpunk bajo la lluvia nocturna con luces neón rosas..."
                    value={aiImagePrompt}
                    onChange={(e) => setAiImagePrompt(e.target.value)}
                    className="w-full bg-black/70 border border-cyan-500/30 rounded-lg p-2.5 text-xs focus:outline-none focus:border-cyan-400 text-zinc-100 h-24 resize-none placeholder-zinc-600"
                  />

                  <button 
                    onClick={() => handleGenerateAiImage()}
                    disabled={isGeneratingAiImage || !aiImagePrompt}
                    className="w-full bg-cyan-600 hover:bg-cyan-500 text-white py-2.5 rounded-lg text-xs font-semibold flex items-center justify-center gap-2 transition-colors disabled:opacity-50 shadow-[0_0_15px_rgba(6,182,212,0.4)]"
                  >
                    {isGeneratingAiImage ? <Loader2 className="animate-spin" size={15} /> : <Wand2 size={15} />}
                    {isGeneratingAiImage ? 'Creando Imagen con IA...' : 'Crear Fondo Personalizado'}
                  </button>
               </div>
            </div>

            {/* Copyright-Free Stock Image Search */}
            <div className="bg-emerald-950/20 border border-emerald-500/30 rounded-xl p-3.5 space-y-3">
               <div className="flex items-center gap-2">
                  <ImageIcon size={16} className="text-emerald-400" />
                  <span className="text-xs font-semibold text-emerald-200">Explorar Imágenes No-Copyright (Stock HD)</span>
               </div>
               
               <div className="flex gap-1.5 flex-wrap">
                  {presetImageCategories.map(cat => (
                    <button
                      key={cat}
                      onClick={() => { setImageSearchQuery(cat); handleSearchImages(cat); }}
                      className="px-2.5 py-1 bg-emerald-900/40 border border-emerald-500/20 hover:border-emerald-400 rounded-full text-[10px] text-emerald-300 transition-colors"
                    >
                      {cat}
                    </button>
                  ))}
               </div>

               <div className="flex gap-2">
                  <input 
                    type="text" 
                    placeholder="Buscar fotos 4K (Cyberpunk, Anime, Space)..."
                    value={imageSearchQuery}
                    onChange={(e) => setImageSearchQuery(e.target.value)}
                    onKeyDown={(e) => e.key === 'Enter' && handleSearchImages()}
                    className="flex-1 bg-black/60 border border-emerald-500/20 rounded-lg px-3 py-1.5 text-xs focus:outline-none focus:border-emerald-400 text-zinc-200"
                  />
                  <button 
                    onClick={() => handleSearchImages()}
                    disabled={isSearchingImages || !imageSearchQuery}
                    className="bg-emerald-600 hover:bg-emerald-500 text-white px-3 py-1.5 rounded-lg text-xs font-medium flex items-center justify-center transition-colors disabled:opacity-50"
                  >
                    {isSearchingImages ? <Loader2 className="animate-spin" size={14} /> : <Search size={14} />}
                  </button>
               </div>

               {/* Stock Image Results Grid */}
               {imageSearchResults.length > 0 && (
                 <div className="grid grid-cols-2 gap-2 pt-2 max-h-60 overflow-y-auto pr-1">
                    {imageSearchResults.map((item) => (
                      <div key={item.id} className="group relative aspect-video bg-black/60 rounded-lg overflow-hidden border border-white/10 hover:border-emerald-400 transition-all">
                         <img src={item.thumb_url} alt={item.title} className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300" />
                         <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent opacity-0 group-hover:opacity-100 transition-opacity p-2 flex flex-col justify-between">
                            <span className="text-[9px] text-zinc-300 truncate">{item.author}</span>
                            <button
                              onClick={() => handleSelectStockImage(item)}
                              disabled={isImportingImage === item.id}
                              className="w-full bg-emerald-600 hover:bg-emerald-500 text-white py-1 rounded text-[10px] font-semibold flex items-center justify-center gap-1 shadow-lg"
                            >
                              {isImportingImage === item.id ? <Loader2 className="animate-spin" size={10} /> : <CheckCircle2 size={10} />}
                              Usar Fondo
                            </button>
                         </div>
                      </div>
                    ))}
                 </div>
               )}
            </div>

            {/* Copyright-Free Music Search */}
            <div className="bg-purple-950/20 border border-purple-500/30 rounded-xl p-3.5 space-y-3">
               <div className="flex items-center gap-2">
                  <Sparkles size={16} className="text-purple-400" />
                  <span className="text-xs font-semibold text-purple-200">Explorar Música No-Copyright</span>
               </div>
               
               <div className="flex gap-1.5 flex-wrap">
                  {presetCategories.map(cat => (
                    <button
                      key={cat}
                      onClick={() => { setSearchQuery(cat); handleSearch(cat); }}
                      className="px-2.5 py-1 bg-purple-900/40 border border-purple-500/20 hover:border-purple-400 rounded-full text-[10px] text-purple-300 transition-colors"
                    >
                      {cat}
                    </button>
                  ))}
               </div>

               <div className="flex gap-2">
                  <input 
                    type="text" 
                    placeholder="Ej: Slap House NCS, Lofi Girl..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
                    className="flex-1 bg-black/60 border border-purple-500/20 rounded-lg px-3 py-1.5 text-xs focus:outline-none focus:border-purple-400 text-zinc-200"
                  />
                  <button 
                    onClick={() => handleSearch()}
                    disabled={isSearching || !searchQuery}
                    className="bg-purple-600 hover:bg-purple-500 text-white px-3 py-1.5 rounded-lg text-xs font-medium flex items-center justify-center transition-colors disabled:opacity-50"
                  >
                    {isSearching ? <Loader2 className="animate-spin" size={14} /> : <Search size={14} />}
                  </button>
               </div>

               {/* Search Results List */}
               {searchResults.length > 0 && (
                 <div className="space-y-2 pt-2 max-h-52 overflow-y-auto pr-1">
                    {searchResults.map((item, idx) => (
                      <div key={idx} className="bg-black/40 border border-white/5 rounded-lg p-2.5 flex items-center justify-between gap-2 hover:border-purple-500/40 transition-colors">
                         <div className="overflow-hidden flex-1">
                            <h4 className="text-xs font-medium text-zinc-200 truncate">{item.title}</h4>
                            <p className="text-[10px] text-zinc-500 truncate">{item.channel} • {item.duration_string || 'N/A'}</p>
                         </div>
                         
                         <div className="flex items-center gap-1.5 shrink-0">
                            <button
                              onClick={() => handleTogglePreview(item.webpage_url)}
                              className={`p-1.5 rounded-md transition-colors text-[10px] flex items-center ${
                                playingPreviewUrl === item.webpage_url 
                                  ? 'bg-emerald-500/30 text-emerald-400 border border-emerald-500/50' 
                                  : 'bg-zinc-800 hover:bg-zinc-700 text-zinc-300'
                              }`}
                              title="Escuchar vista previa"
                            >
                              {isPreviewLoading === item.webpage_url ? (
                                <Loader2 className="animate-spin" size={12} />
                              ) : playingPreviewUrl === item.webpage_url ? (
                                <Pause size={12} />
                              ) : (
                                <Play size={12} />
                              )}
                            </button>

                            <button 
                              onClick={() => handleDownloadTrack(item.webpage_url, item.title)}
                              disabled={downloading}
                              className="bg-purple-600/30 hover:bg-purple-600 text-purple-200 hover:text-white p-1.5 rounded-md transition-colors text-[10px] flex items-center gap-1"
                              title="Descargar y usar en video"
                            >
                              {downloading && downloadingUrl === item.webpage_url ? (
                                <Loader2 className="animate-spin" size={12} />
                              ) : (
                                <Download size={12} />
                              )}
                            </button>
                         </div>
                      </div>
                    ))}
                 </div>
               )}
            </div>

            {/* Direct Link Input */}
            <div className="bg-zinc-900/40 border border-white/5 rounded-xl p-3">
               <div className="flex items-center gap-2 mb-2">
                  <LinkIcon size={14} className="text-zinc-400" />
                  <span className="text-xs font-semibold text-zinc-300">Enlace Directo de YouTube</span>
               </div>
               <div className="flex gap-2">
                  <input 
                    type="url" 
                    placeholder="URL de YouTube..."
                    value={ytUrl}
                    onChange={(e) => setYtUrl(e.target.value)}
                    className="flex-1 bg-black/60 border border-zinc-800 rounded-lg px-2.5 py-1.5 text-xs focus:outline-none focus:border-purple-400 text-zinc-200"
                  />
                  <button 
                    onClick={() => handleDownloadTrack(ytUrl, "YouTube Track")}
                    disabled={downloading || !ytUrl}
                    className="bg-purple-600 hover:bg-purple-500 text-white px-3 rounded-lg text-xs font-medium flex items-center justify-center transition-colors disabled:opacity-50"
                  >
                    {downloading && downloadingUrl === ytUrl ? <Loader2 className="animate-spin" size={14} /> : <Download size={14} />}
                  </button>
               </div>
            </div>

            {/* Local Uploads */}
            <div className="grid grid-cols-2 gap-2">
               <label className="flex items-center justify-center h-10 bg-zinc-900 border border-white/10 rounded-lg text-xs cursor-pointer hover:border-cyan-500/40 transition-colors">
                  <Upload size={14} className="mr-1.5 text-cyan-400" /> Subir Imagen
                  <input type="file" accept="image/*" className="hidden" onChange={handleImageUpload} />
               </label>
               
               <label className="flex items-center justify-center h-10 bg-zinc-900 border border-white/10 rounded-lg text-xs cursor-pointer hover:border-purple-500/40 transition-colors">
                  <Upload size={14} className="mr-1.5 text-purple-400" /> Subir Audio
                  <input type="file" accept="audio/*" className="hidden" onChange={handleAudioUpload} />
               </label>
            </div>

          </div>
        </aside>

        {/* Center Panel - Canvas & Meticulous Vizzy.io Timeline Editor */}
        <section className="flex-1 flex flex-col relative bg-[#020202]">
          <div className="flex-1 p-6 flex items-center justify-center overflow-hidden relative">
             <div className="relative rounded-lg overflow-hidden border border-white/10 shadow-2xl ring-1 ring-white/5 max-w-full max-h-full aspect-video flex items-center justify-center bg-black">
                <canvas 
                  ref={canvasRef} 
                  width={1280} 
                  height={720}
                  className="w-full h-full object-contain"
                />
                
                {!imageUrl && !audioUrl && (
                  <div className="absolute inset-0 flex flex-col items-center justify-center bg-black/60 backdrop-blur-sm z-10 p-6 text-center">
                     <ImageIcon size={48} className="text-zinc-600 mb-4 animate-bounce" />
                     <p className="text-zinc-400 text-sm max-w-md">
                       Escribe lo que quieras crear en la casilla de IA arriba a la izquierda y presiona "Crear Fondo Personalizado".
                     </p>
                  </div>
                )}

                {imageUrl && !audioUrl && (
                  <div className="absolute bottom-4 left-1/2 -translate-x-1/2 bg-black/80 backdrop-blur-md px-4 py-2 rounded-full border border-purple-500/30 text-purple-300 text-xs flex items-center gap-2 shadow-lg">
                    <Music size={14} className="animate-pulse text-purple-400" />
                    Selecciona una canción en el panel izquierdo para encender el ecualizador
                  </div>
                )}
             </div>
          </div>
          
          {/* Meticulous Vizzy.io Timeline Editor with Fully Interactive Layer Engine */}
          <div className="h-64 border-t border-white/10 bg-[#08080a] flex flex-col justify-between z-10 font-mono select-none">
             
             {/* Vizzy Header Toolbar with Interactive "+ Layer" Popup Menu */}
             <div className="h-9 border-b border-white/10 bg-[#0d0d11] px-4 flex items-center justify-between text-xs text-zinc-400 relative">
                
                {/* Left: Layer Actions & Menu Toggle */}
                <div className="flex items-center gap-2 relative">
                   <button 
                     onClick={() => setIsAddLayerMenuOpen(!isAddLayerMenuOpen)}
                     className="flex items-center gap-1.5 px-3 py-1 bg-gradient-to-r from-cyan-600 to-blue-600 hover:from-cyan-500 hover:to-blue-500 text-white rounded text-[11px] font-semibold transition-all shadow-[0_0_10px_rgba(6,182,212,0.4)]"
                     title="Agregar Nueva Capa al Proyecto Vizzy"
                   >
                     <Plus size={13} />
                     <span>+ Layer</span>
                   </button>

                   {/* Add Layer Floating Popup Menu */}
                   {isAddLayerMenuOpen && (
                     <div className="absolute bottom-full mb-2 left-0 z-50 bg-[#0d0d14] border border-cyan-500/40 shadow-[0_0_25px_rgba(0,0,0,0.8)] rounded-xl p-2 w-64 text-xs font-sans space-y-1 animate-in fade-in duration-200">
                        <div className="px-2 py-1 text-[10px] uppercase font-mono text-cyan-400 font-bold tracking-wider flex items-center justify-between">
                           <span>Añadir Capa Vizzy</span>
                           <button onClick={() => setIsAddLayerMenuOpen(false)} className="hover:text-white"><X size={12} /></button>
                        </div>
                        <div className="h-px bg-white/10 my-1"></div>

                        <button 
                          onClick={() => handleAddLayer('text')}
                          className="w-full flex items-center gap-2.5 px-2.5 py-1.5 hover:bg-cyan-950/60 hover:text-cyan-300 rounded-lg transition-colors text-left text-zinc-300"
                        >
                           <Type size={14} className="text-rose-400 shrink-0" />
                           <div>
                              <div className="font-semibold text-[11px]">Texto Animado / Título</div>
                              <div className="text-[9px] text-zinc-500">Overlay de texto personalizado</div>
                           </div>
                        </button>

                        <button 
                          onClick={() => handleAddLayer('particles')}
                          className="w-full flex items-center gap-2.5 px-2.5 py-1.5 hover:bg-cyan-950/60 hover:text-cyan-300 rounded-lg transition-colors text-left text-zinc-300"
                        >
                           <Sparkles size={14} className="text-sky-400 shrink-0" />
                           <div>
                              <div className="font-semibold text-[11px]">Partículas Neón & Dust</div>
                              <div className="text-[9px] text-zinc-500">Efectos flotantes estelares</div>
                           </div>
                        </button>

                        <button 
                          onClick={() => handleAddLayer('spectrum')}
                          className="w-full flex items-center gap-2.5 px-2.5 py-1.5 hover:bg-cyan-950/60 hover:text-cyan-300 rounded-lg transition-colors text-left text-zinc-300"
                        >
                           <Activity size={14} className="text-purple-400 shrink-0" />
                           <div>
                              <div className="font-semibold text-[11px]">Espectro de Onda (FX)</div>
                              <div className="text-[9px] text-zinc-500">Ecualizador audio-reactivo</div>
                           </div>
                        </button>

                        <button 
                          onClick={() => handleAddLayer('image')}
                          className="w-full flex items-center gap-2.5 px-2.5 py-1.5 hover:bg-cyan-950/60 hover:text-cyan-300 rounded-lg transition-colors text-left text-zinc-300"
                        >
                           <ImageIcon size={14} className="text-cyan-400 shrink-0" />
                           <div>
                              <div className="font-semibold text-[11px]">Imagen de Fondo</div>
                              <div className="text-[9px] text-zinc-500">Capa de arte adicional</div>
                           </div>
                        </button>

                        <button 
                          onClick={() => handleAddLayer('watermark')}
                          className="w-full flex items-center gap-2.5 px-2.5 py-1.5 hover:bg-cyan-950/60 hover:text-cyan-300 rounded-lg transition-colors text-left text-zinc-300"
                        >
                           <ShieldAlert size={14} className="text-emerald-400 shrink-0" />
                           <div>
                              <div className="font-semibold text-[11px]">Marca de Agua / Logo</div>
                              <div className="text-[9px] text-zinc-500">Badge de verificación de canal</div>
                           </div>
                        </button>
                     </div>
                   )}

                   <button 
                     onClick={() => setIsSnapped(!isSnapped)}
                     className={`flex items-center gap-1 px-2 py-1 rounded text-[10px] font-semibold border transition-colors ${
                       isSnapped ? 'bg-cyan-950/60 border-cyan-500/40 text-cyan-300' : 'bg-zinc-900 border-white/5 text-zinc-500'
                     }`}
                     title="Ajuste Magnético al Timeline"
                   >
                     <Magnet size={12} />
                     <span>SNAP</span>
                   </button>
                </div>

                {/* Center: Layer Tree Badge */}
                <div className="flex items-center gap-2 bg-zinc-950/80 px-3 py-1 rounded border border-white/5 text-[11px] text-zinc-300">
                   <Layers size={13} className="text-purple-400" />
                   <span className="text-zinc-400 font-sans text-[11px]">Vizzy Layer Sequence Tree</span>
                   <span className="bg-purple-900/50 text-purple-300 px-1.5 py-0.2 rounded text-[10px]">{layers.length} Layers</span>
                </div>

                {/* Right: Resolution & Metrics */}
                <div className="flex items-center gap-3 text-[11px]">
                   <span className="text-emerald-400 font-mono flex items-center gap-1">
                      <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
                      60 FPS
                   </span>
                   <span className="text-zinc-500">|</span>
                   <span className="text-zinc-400 font-mono">1080p FHD</span>
                </div>
             </div>

             {/* Vizzy Multi-Track View & Time Scrubber Grid */}
             <div className="flex-1 flex flex-col bg-[#050507] relative overflow-hidden">
                
                {/* Time Scrubber Header Line (Ruler) */}
                <div 
                  onClick={handleSeek}
                  className="h-6 border-b border-white/10 bg-[#09090d] relative cursor-pointer overflow-hidden flex items-center"
                >
                   {/* Left Empty Header Spacer to match track headers */}
                   <div className="w-48 shrink-0 h-full border-r border-white/10 bg-[#0c0c10] flex items-center px-3 text-[10px] text-zinc-500 font-semibold tracking-wider uppercase">
                      Timeline Ruler
                   </div>

                   {/* Right Time Ruler Ticks */}
                   <div className="flex-1 h-full relative">
                      {renderRulerTicks()}
                   </div>
                </div>

                {/* Vertical Playhead Scrubber Line */}
                <div 
                  className="absolute top-0 bottom-0 pointer-events-none z-30 transition-all duration-75 flex flex-col items-center"
                  style={{ 
                    left: duration ? `calc(12rem + ${(currentTime / duration)} * (100% - 12rem))` : '12rem'
                  }}
                >
                   {/* Diamond Head Needle */}
                   <div className="w-3 h-3 bg-cyan-400 rotate-45 border border-white shadow-[0_0_8px_rgba(6,182,212,0.8)] -mt-1.5 shrink-0" />
                   <div className="w-0.5 flex-1 bg-cyan-400 shadow-[0_0_5px_rgba(6,182,212,0.6)]" />
                </div>

                {/* Dynamic Tracks Rows Container */}
                <div className="flex-1 flex flex-col justify-around py-1 space-y-1 overflow-y-auto">
                   
                   {layers.map((layer) => (
                     <div 
                       key={layer.id} 
                       onClick={() => setSelectedLayerId(layer.id)}
                       className={`h-9 flex items-center border-b border-white/5 transition-colors cursor-pointer ${
                         selectedLayerId === layer.id ? 'bg-zinc-900/90 ring-1 ring-cyan-500/40' : 'bg-zinc-950/40 hover:bg-zinc-900/40'
                       }`}
                     >
                        {/* Left Header */}
                        <div className="w-48 shrink-0 h-full border-r border-white/10 bg-[#0b0b0f] flex items-center px-2.5 justify-between text-[11px]">
                           <div className="flex items-center gap-1.5 truncate">
                              <span className="w-2 h-2 rounded-full shrink-0" style={{ backgroundColor: layer.color }}></span>
                              <span className="text-zinc-300 font-sans text-[11px] font-medium truncate">{layer.name}</span>
                           </div>
                           <div className="flex items-center gap-1">
                              <button 
                                onClick={(e) => { e.stopPropagation(); handleToggleLayerVisibility(layer.id); }}
                                className={`p-1 rounded hover:bg-zinc-800 ${layer.visible ? 'text-cyan-400' : 'text-zinc-600'}`}
                                title={layer.visible ? "Ocultar Capa" : "Mostrar Capa"}
                              >
                                 {layer.visible ? <Eye size={12} /> : <EyeOff size={12} />}
                              </button>
                              <button 
                                onClick={(e) => { e.stopPropagation(); handleToggleLayerLock(layer.id); }}
                                className={`p-1 rounded hover:bg-zinc-800 ${layer.locked ? 'text-amber-400' : 'text-zinc-600'}`}
                                title="Bloquear Capa"
                              >
                                 {layer.locked ? <Lock size={12} /> : <Unlock size={12} />}
                              </button>
                              {layer.type !== 'audio' && (
                                <button 
                                  onClick={(e) => { e.stopPropagation(); handleDeleteLayer(layer.id); }}
                                  className="p-1 rounded hover:bg-zinc-800 text-zinc-600 hover:text-red-400 transition-colors"
                                  title="Eliminar Capa"
                                >
                                   <Trash2 size={12} />
                                </button>
                              )}
                           </div>
                        </div>

                        {/* Track Clip Bar */}
                        <div className="flex-1 h-full px-2 flex items-center">
                           {layer.type === 'audio' ? (
                             <div 
                               onClick={handleSeek}
                               className="w-full bg-emerald-950/20 border border-emerald-500/30 rounded h-7 relative overflow-hidden cursor-pointer group"
                             >
                                <div 
                                  className="absolute left-0 top-0 bottom-0 bg-emerald-500/20 border-r-2 border-cyan-400 transition-all duration-75 z-10"
                                  style={{ width: duration ? `${(currentTime / duration) * 100}%` : '0%' }}
                                />
                                <div className="absolute inset-0 flex items-center justify-between px-1 opacity-50 pointer-events-none">
                                   {Array.from({ length: 140 }).map((_, i) => (
                                     <div key={i} className="w-0.5 bg-emerald-400 rounded-full" style={{ height: `${Math.abs(Math.sin(i * 0.15)) * 60 + 20}%` }} />
                                   ))}
                                </div>
                                <div className="absolute inset-0 flex items-center px-3 z-20 pointer-events-none">
                                   <Music size={12} className="text-emerald-400 mr-2 shrink-0" />
                                   <span className="text-[10px] text-emerald-200 font-sans truncate">{activeTrackName}</span>
                                </div>
                             </div>
                           ) : (
                             <div 
                               className="w-full border rounded h-7 flex items-center justify-between px-3 text-[10px] transition-all"
                               style={{ 
                                 borderColor: `${layer.color}40`,
                                 backgroundColor: `${layer.color}15`,
                                 color: layer.color
                               }}
                             >
                                <div className="flex items-center gap-2 truncate">
                                   {layer.type === 'text' && <Type size={12} className="shrink-0" />}
                                   {layer.type === 'particles' && <Sparkles size={12} className="shrink-0" />}
                                   {layer.type === 'spectrum' && <Activity size={12} className="shrink-0" />}
                                   {layer.type === 'image' && <ImageIcon size={12} className="shrink-0" />}
                                   {layer.type === 'watermark' && <ShieldAlert size={12} className="shrink-0" />}
                                   <span className="truncate font-sans font-medium">
                                      {layer.type === 'text' ? `Texto: "${layer.text || 'Sin texto'}"` : layer.name}
                                   </span>
                                </div>
                                <span className="font-mono text-[9px] opacity-70 shrink-0">00:00 - {formatTimeDetailed(duration)}</span>
                             </div>
                           )}
                        </div>
                     </div>
                   ))}

                </div>

             </div>

             {/* Vizzy Bottom Playback & Master Control Bar */}
             <div className="h-10 border-t border-white/10 bg-[#0c0c10] px-4 flex items-center justify-between text-xs text-zinc-400">
                
                {/* Left: Zoom Controls */}
                <div className="flex items-center gap-2">
                   <span className="text-[10px] text-zinc-500 uppercase tracking-wider font-sans">Zoom:</span>
                   <button 
                     onClick={() => setZoomLevel(prev => Math.max(0.5, prev - 0.25))}
                     className="p-1 hover:bg-zinc-800 rounded text-zinc-300 transition-colors"
                     title="Alejar Zoom"
                   >
                     <ZoomOut size={14} />
                   </button>
                   <span className="text-[11px] text-cyan-400 font-semibold w-9 text-center">{(zoomLevel * 100).toFixed(0)}%</span>
                   <button 
                     onClick={() => setZoomLevel(prev => Math.min(3, prev + 0.25))}
                     className="p-1 hover:bg-zinc-800 rounded text-zinc-300 transition-colors"
                     title="Acercar Zoom"
                   >
                     <ZoomIn size={14} />
                   </button>
                </div>

                {/* Center: Vizzy Playback Controls */}
                <div className="flex items-center gap-3">
                   <button 
                     onClick={() => { if(audioElRef.current) { audioElRef.current.currentTime = 0; setCurrentTime(0); } }}
                     className="p-1 hover:bg-zinc-800 rounded text-zinc-400 hover:text-white transition-colors"
                     title="Ir al inicio"
                   >
                     <SkipBack size={14} />
                   </button>

                   <button 
                     onClick={() => stepTime(-5)}
                     className="p-1 hover:bg-zinc-800 rounded text-zinc-400 hover:text-white transition-colors text-[10px]"
                     title="Retroceder 5s"
                   >
                     -5s
                   </button>

                   <button 
                      onClick={togglePlay}
                      disabled={!audioUrl}
                      className="w-8 h-8 rounded-lg bg-gradient-to-br from-cyan-500 to-cyan-600 hover:from-cyan-400 hover:to-cyan-500 flex items-center justify-center text-white disabled:opacity-30 transition-all shadow-[0_0_12px_rgba(6,182,212,0.5)]"
                   >
                      {isPlaying ? <Pause size={15} /> : <Play size={15} className="ml-0.5" />}
                   </button>

                   <button 
                     onClick={() => stepTime(5)}
                     className="p-1 hover:bg-zinc-800 rounded text-zinc-400 hover:text-white transition-colors text-[10px]"
                     title="Adelantar 5s"
                   >
                     +5s
                   </button>

                   <button 
                     onClick={() => { if(audioElRef.current && duration) { audioElRef.current.currentTime = duration; setCurrentTime(duration); } }}
                     className="p-1 hover:bg-zinc-800 rounded text-zinc-400 hover:text-white transition-colors"
                     title="Ir al final"
                   >
                     <SkipForward size={14} />
                   </button>

                   <button 
                     onClick={() => setIsLooping(!isLooping)}
                     className={`p-1 rounded transition-colors ${isLooping ? 'bg-cyan-950 text-cyan-400 border border-cyan-500/40' : 'text-zinc-500 hover:text-white'}`}
                     title={isLooping ? "Bucle Activado" : "Bucle Desactivado"}
                   >
                     <Repeat size={14} />
                   </button>

                   {/* Detailed Time Counter */}
                   <div className="text-[11px] text-zinc-200 bg-black/80 px-3 py-1 rounded border border-white/10 tracking-widest font-mono">
                      {formatTimeDetailed(currentTime)} / {formatTimeDetailed(duration)}
                   </div>
                </div>

                {/* Right: Audio Volume Slider & Mute Toggle */}
                <div className="flex items-center gap-3">
                   <div className="flex items-center gap-2 bg-zinc-900 px-2.5 py-1 rounded border border-white/5">
                      <button onClick={toggleMute} className="text-zinc-400 hover:text-white transition-colors">
                        {isMuted ? <VolumeX size={14} className="text-red-400" /> : <Volume2 size={14} className="text-cyan-400" />}
                      </button>
                      <input 
                        type="range" 
                        min="0" max="1" step="0.05"
                        value={isMuted ? 0 : volume}
                        onChange={handleVolumeChange}
                        className="w-16 h-1 bg-zinc-800 rounded appearance-none cursor-pointer accent-cyan-500"
                      />
                   </div>
                </div>
             </div>

          </div>
        </section>

        {/* Right Panel - Dynamic Layer Properties Panel */}
        <aside className="w-72 border-l border-white/5 bg-[#0a0a0c] flex flex-col z-10">
          <div className="p-4 border-b border-white/5 flex items-center justify-between text-sm font-semibold text-zinc-300">
            <div className="flex items-center gap-2">
               <Settings size={16} className="text-cyan-400" />
               <span>Propiedades Capa</span>
            </div>
            {selectedLayer && (
              <span className="text-[10px] px-2 py-0.5 rounded font-mono border" style={{ color: selectedLayer.color, borderColor: `${selectedLayer.color}40`, backgroundColor: `${selectedLayer.color}15` }}>
                 {selectedLayer.type.toUpperCase()}
              </span>
            )}
          </div>
          
          <div className="p-5 space-y-6 flex-1 flex flex-col overflow-y-auto">
            
            {/* Selected Layer Specific Customization */}
            {selectedLayer ? (
               <div className="space-y-5">
                  <div>
                     <label className="block text-xs font-medium text-zinc-400 mb-1.5">Nombre de la Capa</label>
                     <input 
                       type="text" 
                       value={selectedLayer.name}
                       onChange={(e) => handleUpdateSelectedLayer({ name: e.target.value })}
                       className="w-full bg-black/70 border border-zinc-800 rounded-lg p-2 text-xs focus:outline-none focus:border-cyan-400 text-zinc-100 font-sans"
                     />
                  </div>

                  {/* Text Layer Customization (Position Aware) */}
                  {selectedLayer.type === 'text' && (
                     <div className="space-y-4 pt-2 border-t border-white/5">
                        <div>
                           <label className="block text-xs font-medium text-zinc-400 mb-1.5">Texto del Título</label>
                           <input 
                             type="text" 
                             value={selectedLayer.text || ''}
                             onChange={(e) => handleUpdateSelectedLayer({ text: e.target.value })}
                             placeholder="Ej: NEXUS MUSIC..."
                             className="w-full bg-black/70 border border-rose-500/30 rounded-lg p-2 text-xs focus:outline-none focus:border-rose-400 text-zinc-100 font-sans"
                           />
                        </div>

                        <div>
                           <label className="block text-xs font-medium text-zinc-400 mb-2">Posición Vertical (Y)</label>
                           <div className="grid grid-cols-3 gap-1.5 mb-2">
                              <button 
                                onClick={() => handleUpdateSelectedLayer({ y: 140 })}
                                className={`py-1 rounded text-[10px] font-semibold border transition-colors ${
                                  (selectedLayer.y || 640) <= 200 ? 'bg-rose-950 text-rose-300 border-rose-500/50' : 'bg-zinc-900 border-white/5 text-zinc-400 hover:text-white'
                                }`}
                              >
                                 Arriba
                              </button>
                              <button 
                                onClick={() => handleUpdateSelectedLayer({ y: 360 })}
                                className={`py-1 rounded text-[10px] font-semibold border transition-colors ${
                                  (selectedLayer.y || 640) > 200 && (selectedLayer.y || 640) < 550 ? 'bg-rose-950 text-rose-300 border-rose-500/50' : 'bg-zinc-900 border-white/5 text-zinc-400 hover:text-white'
                                }`}
                              >
                                 Centro
                              </button>
                              <button 
                                onClick={() => handleUpdateSelectedLayer({ y: 640 })}
                                className={`py-1 rounded text-[10px] font-semibold border transition-colors ${
                                  (selectedLayer.y || 640) >= 550 ? 'bg-rose-950 text-rose-300 border-rose-500/50' : 'bg-zinc-900 border-white/5 text-zinc-400 hover:text-white'
                                }`}
                              >
                                 Abajo
                              </button>
                           </div>
                           <input 
                             type="range" 
                             min="60" max="660" step="10"
                             value={selectedLayer.y || 640}
                             onChange={(e) => handleUpdateSelectedLayer({ y: parseInt(e.target.value) })}
                             className="w-full h-1 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-rose-500"
                           />
                        </div>

                        <div>
                           <label className="flex items-center justify-between text-xs font-medium text-zinc-400 mb-2">
                             Tamaño Fuente <span className="text-rose-400 font-mono">{selectedLayer.fontSize || 42}px</span>
                           </label>
                           <input 
                             type="range" 
                             min="20" max="90" step="2"
                             value={selectedLayer.fontSize || 42}
                             onChange={(e) => handleUpdateSelectedLayer({ fontSize: parseInt(e.target.value) })}
                             className="w-full h-1 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-rose-500"
                           />
                        </div>

                        <div>
                          <label className="block text-xs font-medium text-zinc-400 mb-2">Color del Texto</label>
                          <div className="flex gap-2">
                            {['#ffffff', '#f43f5e', '#38bdf8', '#f59e0b', '#10b981', '#a855f7'].map(c => (
                              <button 
                                key={c} 
                                onClick={() => handleUpdateSelectedLayer({ textColor: c, color: c })}
                                className={`w-6 h-6 rounded-full border-2 transition-transform ${
                                  (selectedLayer.textColor || '#ffffff') === c ? 'border-white scale-110' : 'border-transparent hover:scale-105'
                                }`}
                                style={{ backgroundColor: c }}
                              />
                            ))}
                          </div>
                        </div>
                     </div>
                  )}

                  {/* Image Layer Customization */}
                  {selectedLayer.type === 'image' && (
                     <div className="space-y-4 pt-2 border-t border-white/5">
                        <div>
                           <label className="block text-xs font-medium text-zinc-400 mb-1.5">Imagen Específica de esta Capa</label>
                           <label className="flex items-center justify-center py-2 px-3 bg-zinc-900 border border-cyan-500/30 rounded-lg text-xs font-semibold text-cyan-300 cursor-pointer hover:bg-zinc-800 transition-colors">
                              <Upload size={14} className="mr-2 text-cyan-400" /> Subir Nueva Imagen
                              <input 
                                type="file" 
                                accept="image/*" 
                                className="hidden" 
                                onChange={async (e) => {
                                  const file = e.target.files[0];
                                  if (!file) return;
                                  const reader = new FileReader();
                                  reader.onload = async (evt) => {
                                    const base64 = evt.target.result;
                                    handleUpdateSelectedLayer({ url: base64 });
                                    setImageUrl(base64);
                                    try {
                                      const res = await fetch("http://localhost:43211/api/upload-media", {
                                        method: "POST",
                                        headers: { "Content-Type": "application/json" },
                                        body: JSON.stringify({ file_name: file.name, file_data: base64 })
                                      });
                                      const data = await res.json();
                                      if (data.media_url) {
                                        handleUpdateSelectedLayer({ url: data.media_url });
                                        setImageUrl(data.media_url);
                                      }
                                    } catch (err) {}
                                  };
                                  reader.readAsDataURL(file);
                                }}
                              />
                           </label>
                        </div>
                        
                        <div>
                           <label className="flex items-center justify-between text-xs font-medium text-zinc-400 mb-2">
                             Opacidad Capa <span className="text-cyan-400">{Math.round((selectedLayer.opacity !== undefined ? selectedLayer.opacity : 1) * 100)}%</span>
                           </label>
                           <input 
                             type="range" 
                             min="0" max="1" step="0.05"
                             value={selectedLayer.opacity !== undefined ? selectedLayer.opacity : 1}
                             onChange={(e) => handleUpdateSelectedLayer({ opacity: parseFloat(e.target.value) })}
                             className="w-full h-1 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-cyan-500"
                           />
                        </div>

                        <div>
                           <label className="flex items-center justify-between text-xs font-medium text-zinc-400 mb-2">
                             Escala / Zoom <span className="text-cyan-400">{Math.round((selectedLayer.scale !== undefined ? selectedLayer.scale : 1) * 100)}%</span>
                           </label>
                           <input 
                             type="range" 
                             min="0.5" max="2.5" step="0.05"
                             value={selectedLayer.scale !== undefined ? selectedLayer.scale : 1}
                             onChange={(e) => handleUpdateSelectedLayer({ scale: parseFloat(e.target.value) })}
                             className="w-full h-1 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-cyan-500"
                           />
                        </div>

                        <div className="pt-2 border-t border-white/5 flex items-center justify-between">
                           <div>
                              <label className="block text-xs font-medium text-zinc-200">Pulso de Bajo (Bass Bounce)</label>
                              <p className="text-[10px] text-zinc-500">La imagen vibra y late al ritmo del bajo</p>
                           </div>
                           <button 
                             onClick={() => handleUpdateSelectedLayer({ bassBounce: !(selectedLayer.bassBounce !== false) })}
                             className={`px-3 py-1.5 rounded-lg text-xs font-semibold border transition-all ${
                               selectedLayer.bassBounce !== false 
                                 ? 'bg-cyan-950 text-cyan-300 border-cyan-500/50 shadow-[0_0_10px_rgba(6,182,212,0.3)]' 
                                 : 'bg-zinc-900 text-zinc-500 border-white/5'
                             }`}
                           >
                              {selectedLayer.bassBounce !== false ? 'ON 🔥' : 'OFF'}
                           </button>
                        </div>
                     </div>
                  )}

                  {/* Spectrum Layer Customization */}
                  {selectedLayer.type === 'spectrum' && (
                     <div className="space-y-4 pt-2 border-t border-white/5">
                        <div>
                          <label className="block text-xs font-medium text-zinc-400 mb-3">Color de Onda</label>
                          <div className="flex gap-2">
                            {['#00ffff', '#a855f7', '#f43f5e', '#10b981', '#ffffff'].map(c => (
                              <button 
                                key={c} 
                                onClick={() => setWaveColor(c)}
                                className={`w-8 h-8 rounded-full border-2 transition-transform ${waveColor === c ? 'border-white scale-110' : 'border-transparent hover:scale-105'}`}
                                style={{ backgroundColor: c, boxShadow: waveColor === c ? `0 0 10px ${c}` : 'none' }}
                              />
                            ))}
                          </div>
                        </div>
                        
                        <div>
                          <label className="flex items-center justify-between text-xs font-medium text-zinc-400 mb-2">
                            Sensibilidad Audio <span className="text-cyan-400">{sensitivity}x</span>
                          </label>
                          <input 
                            type="range" 
                            min="0.5" max="3" step="0.1" 
                            value={sensitivity}
                            onChange={(e) => setSensitivity(parseFloat(e.target.value))}
                            className="w-full h-1 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-cyan-500"
                          />
                        </div>
                     </div>
                  )}

                  {/* Watermark Layer Customization */}
                  {selectedLayer.type === 'watermark' && (
                     <div className="space-y-4 pt-2 border-t border-white/5">
                        <div>
                           <label className="block text-xs font-medium text-zinc-400 mb-1.5">Texto Marca de Agua</label>
                           <input 
                             type="text" 
                             value={selectedLayer.text || ''}
                             onChange={(e) => handleUpdateSelectedLayer({ text: e.target.value })}
                             placeholder="Ej: NEXUS OFFICIAL..."
                             className="w-full bg-black/70 border border-emerald-500/30 rounded-lg p-2 text-xs focus:outline-none focus:border-emerald-400 text-zinc-100 font-sans"
                           />
                        </div>
                     </div>
                  )}

                  {/* Delete Layer Action Button */}
                  {selectedLayer.type !== 'audio' && (
                     <button 
                       onClick={() => handleDeleteLayer(selectedLayer.id)}
                       className="w-full mt-4 bg-red-950/40 hover:bg-red-900/60 border border-red-500/30 text-red-300 py-2 rounded-lg text-xs font-semibold flex items-center justify-center gap-2 transition-colors"
                     >
                        <Trash2 size={14} /> Eliminar Capa ({selectedLayer.name})
                     </button>
                  )}
               </div>
            ) : (
               <div className="text-xs text-zinc-500 text-center py-6">
                  Selecciona una capa en el timeline para editar sus propiedades.
               </div>
            )}

            {/* Render Video Card in Panel */}
            <div className="pt-4 border-t border-white/5 space-y-3">
               <button 
                 onClick={handleExport}
                 disabled={status === 'processing'}
                 className="w-full bg-gradient-to-r from-cyan-600 to-blue-600 hover:from-cyan-500 hover:to-blue-500 text-white font-semibold py-2.5 rounded-lg text-xs flex items-center justify-center gap-2 transition-all shadow-[0_0_15px_rgba(6,182,212,0.4)] disabled:opacity-50"
               >
                 {status === 'processing' ? <Loader2 className="animate-spin" size={15} /> : <Film size={15} />}
                 {status === 'processing' ? 'Procesando FFMPEG...' : 'Crear Video MP4 Completo'}
               </button>

               {renderedVideoUrl && status === 'done' && (
                  <div className="bg-emerald-950/40 border border-emerald-500/40 rounded-xl p-3 text-center space-y-2 animate-in fade-in duration-500">
                     <div className="flex items-center justify-center gap-2 text-emerald-400 font-semibold text-xs">
                        <CheckCircle2 size={15} /> ¡Video Listo!
                     </div>
                     <button 
                       onClick={() => setIsVideoModalOpen(true)}
                       className="w-full bg-zinc-800 hover:bg-zinc-700 text-cyan-300 font-semibold py-1.5 rounded-lg text-xs flex items-center justify-center gap-1.5 transition-colors border border-cyan-500/30"
                     >
                        <Film size={14} /> Ver Reproductor de Video
                     </button>
                     <button
                       onClick={() => handleSaveVideoToDisk(renderedVideoUrl)}
                       className="w-full bg-emerald-600 hover:bg-emerald-500 text-white font-semibold py-2 rounded-lg text-xs flex items-center justify-center gap-2 transition-all shadow-[0_0_15px_rgba(16,185,129,0.4)]"
                     >
                       <Download size={14} /> Descargar .MP4 a mi PC
                     </button>
                  </div>
               )}
            </div>

            <div className="pt-6 border-t border-white/5 flex-1 flex flex-col justify-end">
              <h3 className="text-xs font-semibold text-zinc-500 uppercase tracking-wider mb-4">Registro Backend NEXUS</h3>
              <div className="bg-black border border-white/5 rounded-lg p-3 h-44 overflow-y-auto font-mono text-[10px] text-zinc-400 space-y-2">
                 {logs.length === 0 ? (
                   <span className="opacity-50">Esperando orden...</span>
                 ) : (
                   logs.map((log, i) => (
                     <div key={i} className={log.includes('SUCCESS') || log.includes('DONE') ? 'text-emerald-400 font-bold' : log.includes('ERROR') ? 'text-red-400' : ''}>
                       &gt; {log}
                     </div>
                   ))
                 )}
              </div>
            </div>
          </div>
        </aside>
        
      </main>
    </div>
  );
}

export default App;
