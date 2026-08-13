# 🎨 NEXUS PROTOCOL — KIT DE PROMPTS 2D ISOMÉTRICO
> Pipeline de arte para el juego: 2D plano con tiles isométricos (estética 2.5D sin el costo del 3D).
> Herramientas: **Scenario.gg** (principal) · **Pixellab.ai** (si haces pixel art).
> Post-procesado: `scripts/asset_pipeline.sh` (limpiar → spritesheet → tileset).

## 1. PROMPT BASE — EL ESTILO GLOBAL (úsalo en TODO)
> La consistencia nace aquí: este bloque va al inicio de cada prompt, siempre.

```
2D isometric game asset, stylized fantasy, soft cel-shaded lighting, clean
outlines, vibrant but muted palette (deep teal, warm amber, stone gray),
consistent with a dark sci-fantasy world, game-ready, transparent background,
no text, no watermark, centered composition
```

## 2. PERSONAJE PRINCIPAL (héroe)
```
2D isometric game character sprite, male warrior in dark sci-fantasy armor,
teal energy accents, hooded, [BASE], idle pose facing lower-right, full body,
[64px tiles, front 3/4 view]
```
**Flujo de poses (misma referencia):** idle → run → jump → attack_melee → attack_ranged → hurt → death.
Usa la imagen del idle como *reference* en Scenario para que la cara/armadura no cambie.

## 3. ENEMIGOS (las 4 bestias del juego)
| Enemigo | Prompt específico (añade al BASE) |
|---|---|
| 🦇 Bat | `isometric bat monster, leathery wings spread, glowing teal eyes, small 2-tile flying enemy, hovering` |
| 🐗 Boar | `isometric boar beast, tusks, dark bristly hide with stone-like plates, charging stance, ground enemy` |
| 🗿 Golem | `isometric stone golem, moss and crystal growths, massive boulder fists, heavy idle pose, 2x2 tiles` |
| 🕷️ Spider | `isometric spider monster, eight legs, armored carapace, teal venom drip, skittering pose` |

Cada enemigo: genera `idle / attack / hurt / death` con su primera imagen como referencia.

## 4. TILES DEL MUNDO PROCEDURAL (por bioma)
> Tu mundo genera chunks: necesitas tiles seamless de cada bioma. **Grid 32x32.**
| Bioma | Prompt |
|---|---|
| Grass | `isometric terrain tile, grass top with dirt sides, seamless tileable, [BASE]` |
| Dirt | `isometric terrain tile, packed dirt with small stones, seamless, [BASE]` |
| Stone | `isometric terrain tile, cracked stone platform, seamless, [BASE]` |
| Water | `isometric water tile, animated glow surface, seamless, semi-transparent edges` |
| Snow | `isometric terrain tile, snow-covered ground, seamless, [BASE]` |

Genera 2-4 variantes de cada uno (el procedural se ve mejor con variación).
Verifica con: `scripts/asset_pipeline.sh tileset assets/tiles/grass 32`

## 5. OBJETOS (lotes por categoría)
```
isometric game object, [OBJETO], [BASE], small single-tile prop
```
- Consumibles: potion (roja), potion (verde), bandage, cristal de energía
- Armas (suelo): espada, hacha, arco, bastón
- Cofres: cofre cerrado / abierto, cofre jefe
- Decoración: cristales teal, antorchas, huesos, runas
- Pickups: monedas, gemas, orbes de habilidad

## 6. UI (iconos de habilidades — tu SkillManager)
```
game ui icon, [HABILIDAD], flat minimal, dark background circle, teal accent,
[32px, no text]
```
- Golpe cuerpo a cuerpo (melee), disparo a distancia (ranged), escudo, dash,
  y las que defina SkillManager.

## 7. FONDOS PARALLAX (profundidad 2.5D sin 3D)
```
side-scrolling isometric background layer, [LEJOS/MEDIO/CERCA], silhouette
mountains / ruined city / crystals, [BASE], no ground tile, wide 1920px
```
3 capas por zona: lejos (silueta), medio (ruinas), cerca (cristales/antorchas).

## 8. ORDEN DE PRODUCCIÓN (camino crítico)
1. PROMPT BASE + héroe idle → fija el estilo del juego
2. Tiles de biomas (grass/dirt/stone) → el mundo deja de ser gris
3. Enemigos (bat, boar, golem, spider) → combate visible
4. Objetos + cofres → exploración con recompensa
5. UI de habilidades → el SkillManager se siente real
6. Fondos parallax → polish final

## Notas
- Exporta SIEMPRE PNG (transparente). JPG rompe la transparencia.
- Después de descargar: `asset_pipeline.sh limpiar <carpeta>` quita fondos
  residuales y recorta; `spritesheet` arma las hojas para AnimationPlayer.
- Si un enemigo sale mal a la 3ª intento, cambia el prompt antes de insistir.
