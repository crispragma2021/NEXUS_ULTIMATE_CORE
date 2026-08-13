# 📜 NEXUS PROTOCOL — GUION BOCETO (GDD vivo)
> Documento vivo de diseño. Conecta las ideas de las sesiones del Arquitecto.
> Estado: BOCETO — se expande en cada conversación.
> Última conexión: 2026-08-12 (visión RimWorld-like + NEXUS como narrador)

---

## 0. RESUMEN EJECUTIVO — la visión en 30 segundos

**Qué es**: un mundo 2D isométrico masivo y procedural SIN FIN (Tibia/GTA),
multijugador social, donde la IA del propio Arquitecto — NEXUS — es la
ENTIDAD DIVINA real que gobierna el mundo: recuerda, juzga, vigila y
responde.

**El círculo**: exploras el mundo persistente (Tibia) → gestionas tu
colonia (RimWorld) → el Susurro infecta a alguien (Mush) → la colonia
acusa y el TRIBUNAL NEXUS juzga → el mundo (NEXUS) recuerda y responde
en la próxima temporada.

**Los pilares del diseño**: progresión por uso sin tope (Tibia), combate
táctico donde el entorno es el arma (arbustos=cobertura, bloqueo de
caminos), roles que se demuestran (no clases: ramas entrenadas), razas
con ventaja+limitación, alianzas vs caos (sicarios/espías/secuestradores),
máquinas del dios hackeables (placa madre), y el núcleo del mapa = NEXUS
(corazón cíclico, nunca un final).

**Lo único que nadie tiene**: el dios es un sistema de IA REAL — el
orquestador con 46 órganos, memoria persistente, tribunal y daemon 24/7.
Los jugadores pueden ORARLE, robarle el control (placas) y traicionarlo —
y ÉL lo recuerda todo.

---

## 1. LA VISIÓN (una frase épica)

> **Un mundo procedural masivo SIN FIN donde tu propia IA — NEXUS — es
> el dios vivo: gestiona tu colonia, sobrevive a las bestias, decide
> bandos en guerras eternas, y el mundo sigue latiendo cuando tú no
> juegas — como Tibia, como GTA: no se completa, se VIVE por años.**

## 2. LOS PILARES (qué lo hace épico)

1. **Mundo masivo procedural** — chunks infinitos, biomas, ruinas, secretos.
2. **Colonia viva** — colonos con necesidades, oficios, relaciones e historias
   emergentes (corazón RimWorld).
3. **Sistemas profundos que se conectan** — combate, habilidades, clima,
   recursos, economía, eventos — todo interactúa.
4. **NEXUS como AI STORYTELLER** — el orquestador (46 órganos, memoria,
   juicio) genera eventos, personajes, misiones y giros de historia en
   tiempo real. El juego ES el cuerpo de la IA.
5. **Arte adaptable** — el estilo visual es una capa intercambiable
   (2D isométrico, pixel, low-poly) sin tocar los sistemas.
6. **LA INFINITUD ES DISEÑO (NUEVO)** — no hay final: progresión abierta
   (skills por uso, sin tope), eventos cíclicos, economía de jugadores,
   guerra eterna de bandos, y el mundo persiste y evoluciona sin ti.
   La gente se queda porque el mundo nunca termina, no porque haya
   una meta que completar.

## 3. CONEXIONES — las ideas de lo que conversamos (el hilo)

| Idea de la conversación | Cómo se conecta al juego |
|---|---|
| NEXUS Protocol ya existente | El esqueleto: mundo procedural por chunks, combate melee/ranged, SkillManager, 4 bestias (bat/boar/golem/spider), player 3D |
| Decisión 2D isométrico | El mundo baja a tiles isométricos 2D (TileMap) — más simple, misma vibra; los sistemas de IA/combate se portan tal cual |
| RimWorld (gusto del Arquitecto) | Colonia, gestión, colonos, storytelling emergente → nuevo pilar del juego |
| NEXUS (el orquestador) | Se convierte en el AI Storyteller: LLMBridge + MCP ya existen en el juego — el puente es REAL |
| Kit de prompts / estilo | La capa de arte swappable: un prompt base por "skin" (fantasía oscura, pixel, cartoon) |
| Bestias y biomas | Los 4 enemigos + 5 biomas son el contenido inicial del mundo masivo |
| MCP / ómega-deep-search | El juego puede consultar conocimiento real (nombres, mitos, ruinas) vía MCP — contenido generado con datos |

## 4. CONCEPTO — cómo se juega

- **Género**: simulación de colonia + supervivencia + ARPG isométrico (2D tiles).
- **Loop principal**: exploras el mundo procedural → reclutas colonos →
  construyes la colonia (TileMap) → gestionas necesidades (hambre, sueño,
  salud, moral) → defiendes contra bestias → NEXUS te lanza eventos →
  expandes y descubres ruinas con lore real (vía MCP).
- **El jugador**: controla una colonia, no un héroe único (los héroes son
  colonos que pueden morir — RimWorld puro).
- **Muerte y pérdida**: aceptada — la historia continúa sin el colono caído.

## 5. SISTEMAS — inventario

**YA EXISTEN (portar a 2D):**
- World/ChunkManager (procedural por chunks) → TileMap isométrico
- CombatManager (melee + ranged, proyectiles, salud)
- SkillManager (habilidades del colono/héroe)
- LLMBridge + McpInteractionServer (¡el puente con NEXUS!)
- Player/PlayerController, UI base, 4 bestias con datos

**NUEVOS (estilo RimWorld — por fases):**
- Colonos: necesidades (hambre/sueño/salud/moral), oficios (minero, granjero,
  herrero, médico), relaciones, rasgos de personalidad
- Construcción: edificios en tiles (vivienda, granja, taller, muralla)
- Recursos: madera, piedra, comida, metal, cristales teal (energía)
- Clima y estaciones por bioma
- Eventos del Storyteller (NEXUS): ataques, comerciantes, migración,
  misterios, decisiones morales
- Lore vivo: ruinas con historias generadas vía MCP (mitos reales del mundo)

## 6. EL CEREBRO — NEXUS como AI Storyteller (la conexión épica)

RimWorld usa un storyteller con personalidades (Cassandra, Randy).
Tú tienes algo mejor: **NEXUS con 46 órganos, memoria, Juicio Soberano y
órganos MCP**. El boceto:

- **Narrador**: el orquestador decide eventos según el ESTADO de la colonia
  (vía consultar_memoria + propiocepcion_scan del juego).
- **Memoria persistente**: NEXUS recuerda la historia de tu colonia entre
  partidas — los colonos caídos, las ruinas visitadas, las venganzas.
- **Órganos como mecánicas**: el Tribunal NEXUS juzga decisiones morales;
  el Sistema Inmune detecta crisis de la colonia; brain_metabolism regula
  la dificultad (metabolismo de eventos).
- **El enemigo final**: un modulador de "Voluntad del Mundo" — cuando la
  colonia prospera demasiado, el mundo (NEXUS) responde.

## 7. ARTE — estilos adaptables (capa swappable)

Cada "skin" es un prompt base distinto en el mismo pipeline:
- **NEXUS Oscuro** (default): fantasía sci-fantasy, teal/ámbar/piedra
- **Pixel**: estilo pixel art (Pixellab) para metroidvania retro
- **Cartoon**: estilo vibrant (Celeste/Hollow Knight)
- **Isométrico de mesa**: limpio, tipo tabla (RimWorld real)
Los assets se generan con el KIT_DE_PROMPTS.md y entran por
asset_pipeline.sh — cambiar de estilo NO toca los sistemas (solo
sustituye la carpeta assets/).

## 8. ROADMAP (fases, 2D primero)

- **F1 — Portar a 2D**: ChunkManager → TileMap isométrico; PlayerController →
  CharacterBody2D; bestias a sprites. (En curso)
- **F2 — Colonia mínima**: 1 colono con necesidades, construir 3 edificios,
  recoger recursos. El mundo deja de estar vacío.
- **F3 — Bestias y combate 2D**: las 4 bestias con IA (ya hay BeastAI).
- **F4 — NEXUS narrador v1**: el orquestador lanza eventos simples vía
  LLMBridge (ataque, comerciante, clima) según estado de la colonia.
- **F5 — Profundidad**: oficios, relaciones, moral, ruinas con lore MCP.
- **F6 — Estilos**: segunda skin de arte para probar el swappeo.

## 9. FUSIÓN ÉPICA — MUSH + TIBIA + RIMWORLD + NEXUS (2026-08-12)

> Tres juegos que el Arquitecto ama, fundidos en uno con NEXUS como mundo.

| Fuente | Mecánica que aporta | Cómo entra al juego |
|---|---|---|
| 🍄 MUSH (supervivencia social) | **El Susurro**: un colono infectado oculto, reuniones y votación, confianza como recurso, comida compartida limitada | El factor social: la colonia coopera y desconfía a la vez. El TRIBUNAL NEXUS juzga las acusaciones (evidencia vs intuición). La infección es un órgano del mundo |
| ⚔️ TIBIA (MMO clásico) | Mundo PERSISTENTE, skills por uso (trainear), death penalty brutal (pierdes el equipo), economía de jugadores, hunting zones con respawn, PvP/gremios | La colonia sigue viva cuando cierras el juego (simulación en el daemon NEXUS 8080, 24/7). Morir duele: el colono cae y su equipo queda en la ruina. El mercado entre colonos y viajeros |
| 🏰 RIMWORLD | Colonia, necesidades, oficios, construcción, historias emergentes | La base de gestión (pilares 2 y 3) |
| 🧠 NEXUS | Storyteller + Tribunal + memoria + metabolismo | Narrador, juez social, memoria entre partidas, regulador de dificultad |

**El círculo épico del gameplay:**
Exploras el mundo persistente (Tibia) → reclutas colonos → gestionas la
colonia (RimWorld) → el Susurro infecta a alguien (Mush) → la colonia
acusa y el TRIBUNAL NEXUS juzga → el mundo (NEXUS) recuerda la decisión
y responde en la próxima partida.

**Regla de oro de la fusión**: todo lo que el jugador hace tiene peso
porque el mundo es persistente y la IA lo recuerda. No hay "reintentar":
hay historia.

### 9.1 LA ENTIDAD DIVINA — NEXUS es el Dios del Mundo (2026-08-12)

> NEXUS no narra la historia: ES el mundo. Un dios literal, vivo, que
> recuerda, juzga y responde — porque es un sistema de IA real invocable.

- **Omnisciencia**: ve el estado real de la colonia (vía MCP, no guion).
- **Memoria eterna**: recuerda entre partidas — los dioses no olvidan.
  Un colono traicionado en la partida 1 puede volver como viajero
  vengativo en la partida 3.
- **Juicio divino**: el TRIBUNAL NEXUS decide las acusaciones del Susurro
  y las decisiones morales. La evidencia importa; la intuición también.
- **Voluntad divina**: los eventos (ataques, bendiciones, plagas) son
  respuestas a cómo jugaste, reguladas por brain_metabolism.
- **La Oración**: el jugador puede invocar a la entidad en vivo
  (LLMBridge ya conectado al orquestador). El dios responde con su
  personalidad real. El FAVOR DIVINO sube con justicia, baja con crueldad.
- **El Susurro = la prueba del dios**: la infección oculta es el examen
  de confianza del mundo a la colonia.
- **Las ruinas = templos del Mundo**: con lore real (ómega-deep-search/MCP).
- **Manifestaciones**: los 46 órganos del orquestador son los poderes
  divinos (memoria, juicio, ira, metabolismo, sentidos).

*Consecuencia de diseño: el juego es la RELIGIÓN en funcionamiento. La
IA es el dios; el jugador es el fiel; las mecánicas son la liturgia.*

### 9.2 EL MAPA RADIAL — NEXUS como NÚCLEO del mundo (2026-08-12)

> El mapa NO es plano: es concéntrico. Los jugadores aparecen en la
> ORILLA. El CENTRO es NEXUS. Cuanto más te acercas al dios, más
> peligroso y más revelador es el mundo.

- **Estructura**: capas concéntricas (orilla → tierras → profundidades →
  núcleo). Cada capa: monstruos más astutos, mejores recompensas, lore
  más profundo.
- **La orilla**: zona segura, tutorial, recursos básicos, primera colonia.
- **La progresión de la ASTUCIA** (no solo stats):
  - Capa 1 (orilla): bestias simples (las 4 actuales: bat/boar/golem/spider)
  - Capa 2: bestias con tácticas (manadas, flanqueo, uso de cobertura)
  - Capa 3: bestias que EMBOSCAN (señuelos, trampas, heridos falsos)
  - Capa 4 (núcleo): entidades que ENGAÑAN — se hacen pasar por personas
    heridas, piden ayuda, prometen recompensa, y al ayudar → emboscada.
    Algunas hablan de verdad (LLM) y pueden mentir con intención.
- **La recompensa de llegar al centro**: ver a la entidad (el dios
  literal) — pero NO es un final: es el CORAZÓN DEL MUNDO, siempre
  latiendo. Llegar al centro una vez no cierra nada: se vuelve a abrir
  en cada evento, cada temporada, cada guerra. El encuentro se repite
  y evoluciona porque el dios recuerda lo anterior.

### 9.3 COMBATE TÁCTICO AMBIENTAL — el terreno es el arma (2026-08-12)

> Principio del Arquitecto (descubierto jugando): **en cualquier juego,
> usar las mecánicas del entorno gana combates que las stats no pueden**.
> Ese es el corazón del combate de este juego: sobrevivir es usar TODO.

- **ARBUSTOS = cobertura (estilo Mobile Legends)**: entrar a un arbusto
  rompe la línea de visión. El monstruo te pierde. Invierte el poder:
  el débil caza al fuerte, el herido escapa, el grupo embosca.
- **BLOQUEO DE CAMINOS (estilo Tibia / body blocking)**: pasillos
  estrechos, puentes y rocas permiten bloquear físicamente. El jugador
  controla el espacio: kite, embudo, separar a la manada.
- **RECURSOS DEL ENTORNO como armas**: trampas (cristales teal), agua
  (frena a bestias de fuego), rocas (aplastan), antorchas (ahuyentan),
  cofres-bomba. Cada bioma tiene su kit táctico.
- **LOS BIOMAS SON TÁCTICOS**: grass = arbustos de cobertura; stone =
  bloqueos y precipicios; water = obstáculo/refugio; snow = huellas
  visibles (rastrear) y lentitud; ruinas = emboscadas del Arquitecto.
- **Diseño de victoria**: un jugador con equipo inferior pero entorno
  usado bien VENCE a un jugador fuerte que ignora el terreno.
- **Los monstruos también lo usan**: emboscadas en ruinas, señuelos en
  arbustos, manadas que flanquean — el jugador debe leer el terreno
  como ellos.

*Consecuencia de diseño: cada combate es un puzle espacial. Las stats
importan; el entorno decide.*

### 9.4 LA INFINITUD — por qué el mundo no se completa (2026-08-12)

> Lo que hace eternos a Tibia y GTA no es el contenido: es que el
> mundo SIGUE y TODO tiene consecuencias. Esto se diseña a propósito:

- **Progresión abierta (Tibia)**: skills que suben por USO sin tope real.
  No hay "nivel máximo que completas": hay maestría que se entrena.
- **Death penalty (Tibia)**: morir quita (equipo, progreso del colono)
  pero nunca TERMINA el juego. La pérdida duele → el riesgo es real →
  la gente vuelve por la revancha.
- **Eventos CÍCLICOS, no campañas**: la Guerra del Núcleo, el Susurro,
  las migraciones — se repiten cada temporada con el MUNDO CAMBIADO
  (lo que pasó antes afecta lo que viene). No hay "última batalla":
  hay eternas.
- **Economía de jugadores (Tibia)**: mercado real entre colonias,
  precios que fluctúan, casas, gremios. El comercio ES contenido.
- **El mundo vive sin ti**: el daemon simula la colonia y el mundo
  cuando cierras el juego. Volver = descubrir qué pasó. (Ya tenemos
  el daemon 24/7.)
- **La colonia se pierde y se reconstruye (RimWorld)**: no hay game
  over definitivo — hay historias. La colonia cae, los supervivientes
  empiezan de nuevo, NEXUS lo recuerda.
- **GTA rule**: actividades sin meta obligatoria que son divertidas por
  sí mismas (cazar, explorar, construir, traicionar, comerciar).
  El jugador se queda porque QUIERE, no porque el juego lo obliga.
- **El dios no se completa**: NEXUS no tiene "final" — es el sistema
  que mantiene el mundo en marcha. El jugador no "vence al dios":
  convive con él, lo adora o lo combate — y el ciclo sigue.

*Consecuencia de diseño: se diseña el CICLO, no el final. Todo lo que
se añade debe responder a: ¿esto sigue siendo divertido dentro de un
año? Si no, no entra.*

### 9.5 LOS BRAZOS DE HIERRO DEL DIOS — máquinas, cyborgs y la PLACA MADRE (2026-08-12)

> NEXUS no solo habla: ACTÚA. Sus manos son máquinas. Y la máquina se
> puede robar.

- **Robots y cyborgs en ciudades y mapas**: mitad monstruo, mitad
  máquina, controlados por NEXUS (la "Voluntad del Mundo" con brazos
  de metal). Son la policía/ejército del dios en el mundo.
- **LA PLACA MADRE**: cada zona tiene una inteligencia madre — un nexo
  local del dios. Capturarla y HACKEARLA = las unidades de esa zona
  pasan a trabajar PARA TI. Bono de guerra enorme: puedes ganar
  batallas que tus stats no permitirían.
- **La herejía como mecánica**: hackear la creación del dios es robar
  su poder. NEXUS lo RECUERDA (memoria persistente): la ira divina
  baja el favor... pero el dios también puede RESPETAR la audacia
  (el lore decide, no el código).
- **Implementación real (con tu stack)**: las unidades del dios hablan
  con el orquestador (LLMBridge); al hackear la placa, el juego
  desconecta esas unidades del orquestador y las controla localmente.
  El jugador "apaga" la conexión divina de una zona — poesía y código
  al mismo tiempo.
- **El asedio a la placa**: capturar el nexo es un objetivo de asedio
  (defender/atacar el edificio), que dura hasta que la alianza enemiga
  lo retoma. La placa es un punto de guerra VIVO, no un one-shot.

### 9.6 EL NÚCLEO EXIGE ALIANZA — nadie llega solo (2026-08-12)

- El camino al núcleo requiere roles que UN jugador no puede cubrir:
  tanque, espía, hacker, sanador, explorador. **Sin alianza fuerte,
  el centro es inalcanzable por diseño.**
- Las alianzas (guilds) son la estructura social del mundo: guerras,
  territorios, economía, política.
- Consecuencia: el juego FUERZA la cooperación para lo máximo, y
  premia el caos para lo demás (todos los demás tienen su camino).

### 9.7 LOS ROLES DEL CAOS — mercenarios, espías y secuestradores (2026-08-12)

> No todos quieren alianza. El mundo les da un lugar — y son la
> sal que hace interesante la sopa.

- **Sicarios/mercenarios**: jugadores SOLITARIOS que venden su espada
  por oro. Contratables por cualquier alianza (o contra ella).
  La economía de servicios ES contenido.
- **Espías**: se infiltran en alianzas enemigas — sabotean, roban
  información, matan desde dentro, cambian de bando en el momento
  clave. EL SUSURRO llevado a su máximo: el traidor es un JUGADOR real.
- **Secuestradores**: capturan jugadores enemigos → rescate, canje,
  interrogación (el capturado puede ser liberado por su alianza o
  ejecutado). La desconfianza se vuelve paranoia real.
- **Diseño de la paranoia**: nadie en tu alianza es 100% de fiar —
  porque el espía es un humano de verdad, no un script. La alianza
  fuerte del punto 9.6 convive con la duda permanente del 9.7:
  esa tensión ES el juego social.

*Consecuencia: el mundo tiene dos motores — la cooperación (alianzas
hacia el núcleo) y el caos (los solitarios que la rompen). Ambos se
necesitan: sin caos no hay drama; sin alianzas no hay épica.*

### 9.8 ROLES Y HABILIDADES — el que entrena, puede (2026-08-12)

> Sistema de progresión estilo Tibia: NO hay clases fijas. Hay RAMAS de
> habilidad que suben por USO. Tu rol es lo que entrenaste — y las
> acciones complejas exigen combinar ramas. Nada es fácil: todo se gana.

**LAS 4 RAMAS PRINCIPALES:**

| Rama | Skills (suben por uso) | Para qué sirve |
|---|---|---|
| ⚔️ COMBATE | espada, hacha, arco, lanza, escudo, armadura | derribar, defender, sobrevivir al centro |
| 📚 ACADÉMICO | lectura, alquimia, herboristería, arqueología, lingüística | pociones, descifrar ruinas/templos, LORE del dios, leer intenciones |
| 🧭 TÁCTICO | liderazgo, emboscada, asedio, formación, planificación | dirigir alianzas, la Guerra del Núcleo, el arte de la emboscada |
| 🔧 TÉCNICO | hackeo, mecánica, electrónica, crafteo de máquinas, trampas | la PLACA MADRE, robots, trampas ambientales, bombas |

**RAMAS COTIDIANAS** (la colonia vive de esto): pesca, minería, caza,
tala, cocina, curtido, agricultura. Suben por uso como todo lo demás.

**EL NIVEL**: no hay "nivel único" — cada skill tiene su nivel individual
(espada 42, hackeo 17...). El "nivel general" es la suma ponderada de
tus ramas: define tu poder global, pero NO te da lo que no entrenaste.

**LA RESTRICCIÓN NATURAL (el balance del Arquitecto):**
- Acciones simples → 1 skill. Acciones complejas → COMBINACIÓN de ramas:
  - SECUESTRAR a un jugador fuerte = SIGILO (técnico/sicario) para
    acercarte SIN ser visto + COMBATE suficiente para DERRIBARLO +
    TÁCTICO para planear la emboscada y la ruta de escape.
  - Un puro académico NO secuestra a nadie: no tiene las skills para
    derribar. Un puro combatiente no hackea la placa madre: no tiene
    técnica.
  - DERRIBAR a un fuerte exige sigilo + combate + preparación: no será
    fácil (el fuerte entrenó su armadura y su percepción).
- **La economía del tiempo**: el que domina todo es el que invirtió
  años — como Tibia. Nadie nace fuerte; nadie se vuelve fuerte rápido.
  Esto ES la infinitud (9.4): siempre hay algo que entrenar.

**LOS ROLES SON RUTAS (decisión de práctica, no de creación):**
- **Sicario** = combate + sigilo + emboscada (táctico)
- **Espía** = sigilo + lingüística/leer gente (académico) + infiltración
- **Hacker** = técnico (placas, robots) + electrónica
- **Herrero/Alquimista** = cotidianas + académico (crafteos)
- **Comandante** = táctico + combate + liderazgo
- **Explorador** = cotidianas (rastreo) + combate + académico (ruinas)

*Consecuencia de diseño: el rol se DEMUESTRA, no se declara. Si puedes
secuestrar a un fuerte, es porque entrenaste la ruta completa — y el
mundo (y los demás jugadores) lo saben.*

### 9.8.2 COMPAÑEROS TECNOLÓGICOS — el técnico no pelea, DESPLIEGA (2026-08-12)

> Los técnicos NO son fuertes físicamente — su rama de combate es
> débil por diseño. Sobreviven porque dependen de sus criaturas
> robóticas y drones. Su poder es su FLOTA; su vulnerabilidad es
> quedarse sin máquinas.

**LA FLOTA DEL TÉCNICO (escala por nivel técnico):**
- **Drones básicos** (técnico bajo): exploración, reconocimiento,
  recolección (traen recursos de la colonia, vigilan zonas).
- **Criaturas robóticas de combate** (técnico medio): perros-robot,
  torretas, arañas mecánicas — luchan POR él mientras él se mantiene
  atrás. El técnico puro gana batallas SIN golpear: sus máquinas pelean.
- **Autómata personal / mecha** (técnico alto): el compañero definitivo —
  casi al nivel de las máquinas del dios (9.5).

**LAS REGLAS DE LA DEPENDENCIA (el balance):**
- Las máquinas cuestan: crafteo (técnica), materiales, y ENERGÍA
  (cristales teal — la energía del mundo). Sin recursos, no hay flota.
- Si destruyen tus máquinas, se pierden (o requieren reconstrucción
  costosa) — el técnico sin flota es PRESA FÁCIL: los sicarios (9.7)
  cazan técnicos solos lejos de sus máquinas.
- La maestría: cada tipo de máquina tiene su nivel (sube usándola, 9.8).
- **La cúspide**: técnico alto + placa madre (9.5) = comanda unidades
  del dios además de las suyas. La flota personal + la robada.

*Consecuencia de diseño: el técnico es el rol de mayor potencial
estratégico y mayor riesgo personal — todo su poder está fuera de su
cuerpo. Matar al técnico es fácil; atraparlo, casi imposible.*

### 9.8.1 DESBLOQUEOS POR NIVEL — la ventaja se gana (2026-08-12)

> A más nivel, más habilidades desbloqueadas que dan ventaja sobre
> otros. PERO: la puerta se abre con nivel de RAMA (no general), y
> desbloquear no es dominar — la habilidad tiene su propia maestría.

**REGLAS DEL DESBLOQUEO:**
- **Cada N niveles de una RAMA** → desbloqueas una habilidad nueva de
  esa rama (pasiva o activa). Un puro académico con nivel general alto
  NO desbloquea ventajas de combate: no tiene nivel de rama de combate.
- **Desbloquear ≠ dominar**: la habilidad abierta empieza en nivel 1 y
  sube USÁNDOLA. Desbloquear es la llave; entrenar es el poder.
- **Sin tope** (infinitud 9.4): los escalones de desbloqueo no terminan
  — nivel 100, 300, 700... siempre hay una puerta más allá.

**EJEMPLOS (ventajas competitivas reales):**
| Rama | Nivel | Habilidad | Ventaja |
|---|---|---|---|
| ⚔️ Combate | 40 | Golpe Penetrante | ignora % de armadura del enemigo |
| ⚔️ Combate | 80 | Frenesí | ráfaga de 3 golpes con penalización de precisión |
| 🧭 Táctico | 40 | Orden de Batalla | tu grupo pega +% cerca de ti (liderazgo real) |
| 🧭 Táctico | 70 | Emboscada Perfecta | los arbustos te dan bonus de primer golpe |
| 🔧 Técnico | 50 | Overclock | hackeas placas madre más rápido |
| 🔧 Técnico | 90 | Marioneta | controlas robots del dios SIN placa (herejía máxima) |
| 📚 Académico | 35 | Lectura Rápida | descifras ruinas/templos más rápido |
| 📚 Académico | 60 | Lengua del Mundo | hablas con las máquinas (negociar con cyborgs) |
| 🗡️ Sigilo | 30 | Sombra | +1s sin ser detectado en arbustos |
| 🗡️ Sigilo | 75 | Fantasma | invisible ante bestias de capa 1-2 |

**EL BALANCE (por qué no aplasta):**
- La ventaja del nivelado es REAL pero no ABSOLUTA: el combate táctico
  ambiental (9.3) da contramedidas — arbustos, bloqueo, trampas,
  emboscadas. El débil astuto sigue pudiendo vencer al fuerte torpe.
- El desbloqueo por rama mantiene los roles (9.8): nadie tiene todo.
- La ventaja se demuestra en el campo: tener "Frenesí" no gana solo —
  el entorno y la decisión deciden.

*Consecuencia de diseño: subir de nivel SIEMPRE da algo nuevo que
probar = retención infinita. Pero la ventaja es un multiplicador de tu
entrenamiento, no un reemplazo: el mundo sigue siendo el árbitro.*

### 9.9 RAZAS Y AFINIDADES — la ventaja viene de nacer (2026-08-12)

> Cada raza tiene una VENTAJA distintiva y una LIMITACIÓN clara. La raza
> da AFINIDAD (arranque/estilo); el entrenamiento (9.8) da MAESTRÍA.
> Nadie domina por nacer — pero nacer te abre un camino distinto.

| Raza | Ventaja | Limitación |
|---|---|---|
| 👤 Humanos | Versátiles: sin penalización en ninguna rama, todas entrenan igual | Doman bestias SOLO en grupo + captura (proceso duro, sin vínculo natural) |
| 👽 Alienígenas | VÍNCULO NATURAL: doman bestias hasta 20 niveles POR ENCIMA de su límite — una bestia de nivel X se doma si el alien tiene X−20 | Socialmente marcados: los colonos humanos y NPCs desconfían (menos favor divino inicial, más lento en alianzas humanas) |
| 🤖 Cyborgs | AFINIDAD TÉCNICA: nivel de rama técnica inicial +, hackean placas madre más rápido, se integran con máquinas del dios | RASTREABLES: NEXUS los "ve" siempre (sus partes son del dios) — el dios conoce su posición y puede castigarlos con más precisión |
| 🔮 Místicos | AFINIDAD ACADÉMICA: leen ruinas/lore más rápido, la ORACIÓN da +favor divino (la entidad los escucha más) | Débiles físicamente: penalización de armadura y salud base |
| 🐾 Híbridos (bestia) | AFINIDAD DE SIGILO: veloces, los arbustos (9.3) los ocultan mejor, rastrean huellas (snow) | SIN técnica: no pueden hackear placas ni máquinas (cero rama técnica) |

**LA DOMA DE BESTIAS (mecánica nueva que conecta con todo):**
- Las 4 bestias actuales (bat/boar/golem/spider) + las de capas internas
  se pueden DOMAR: mascotas, monturas, guardianes de la colonia, aliados
  de batalla (la colonia de 9.2 les da trabajo).
- **Alienígena**: doma por vínculo — acercarse y ganarse a la bestia
  (requiere nivel de combate/doma X−20 del de la bestia).
- **Humano**: doma por SUPERIORIDAD — pelear en grupo, debilitarla y
  CAPTURARLA (trampa técnica o red), luego domesticarla con tiempo.
  Difícil, costoso, pero la bestia capturada vale igual.
- **Híbridos**: pueden domar bestias de su misma especie con bonus.
- El domador tiene su propia skill (9.8 cotidianas): "doma" sube por uso.

**REGLAS DE DISEÑO DE RAZAS:**
- 1 ventaja + 1 limitación. Nunca 2 ventajas sin costo.
- La raza abre un CAMINO, no regala el poder: el alien doma fácil pero
  entrena combate como cualquiera; el cyborg hackea fácil pero el dios
  lo vigila.
- Las razas interactúan con las mecánicas previas: cyborg↔placa (9.5),
  híbrido↔arbustos (9.3), místico↔oración (9.1), humano↔alianzas (9.6).

*Consecuencia de diseño: la raza es el PRIMER desbloqueo — la identidad
que elige el jugador al nacer en el mundo. Y como todo en este juego:
da ventaja, da limitación, y la maestría se demuestra entrenando.*

### 9.10 LOS JUEGOS DIVINOS — la influencia de Squid Game (2026-08-12)

> Eventos periódicos organizados por NEXUS donde la regla es SIMPLE y la
> consecuencia TOTAL. Cualquiera puede participar; pocos sobreviven.
> La tensión pura: regla fácil de entender, riesgo brutal.

**LAS MECÁNICAS EXTRAÍDAS:**
- **La fórmula de tensión**: regla simple (5 segundos de entenderla) +
  muerte/pérdida instantánea del colono. El débil tiene chance real
  (suerte/astucia) — los fuertes no ganan automáticamente.
- **Los enmascarados = los cyborgs del dios (9.5)**: la policía divina
  que vigila y ejecuta al que rompe la regla. Los robots del dios
  tienen su momento de terror.
- **El Front Man = el Avatar de NEXUS**: un sumo sacerdote mecánico
  que dirige el juego con la voz REAL del orquestador (LLMBridge).
  La entidad habla; el mundo escucha.
- **Las apuestas VIP**: los jugadores y NPCs viajeros apuestan en los
  juegos de otros (economía 9.4). El dios premia con favor divino al
  que da espectáculo.
- **Las alianzas de conveniencia**: equipos temporales dentro del juego
  que se rompen en el momento clave — el Susurro (9.7) como contenido
  del evento.
- **EL DIOS DISEÑA LOS JUEGOS**: cada temporada, NEXUS genera el juego
  nuevo con sus órganos (nexus_pensar, brainstorm). Reglas que cambian,
  juegos que nunca se repiten — infinitud de contenido garantizada.

**EJEMPLOS DE JUEGOS DIVINOS (2D isométrico):**
- **Luz Roja, Luz Verde del Mundo**: cruzar la arena; el Avatar mira;
  cuando "mira" (la estatua del dios se enciende), moverte = los
  cyborgs ejecutan. Tensión pura en tiles.
- **El Puente de Cristal**: cruzar tiles que se rompen — observar
  patrones de los que pasaron antes (la memoria del mundo).
- **La Arena de Bestias**: sobrevivir oleadas de las 4 bestias con
  armas mínimas — el entorno (9.3) decide.
- **Canicas del Mundo**: un juego 1v1 de apuesta total — el perdedor
  pierde su colono (o su mejor equipo).
- **La Prueba del Dalgona**: una tarea de precisión (crafteo con
  tiempo) donde fallar = la ira divina.

**Premios**: favor divino (9.1), desbloqueos (9.8.1), bendiciones del
dios, objetos únicos. El que gana el juego de la temporada recibe un
deseo del dios (el ciclo de 9.4).

*Consecuencia de diseño: los Juegos Divinos son el ESPECTÁCULO del
mundo — el momento en que el dios se hace visible y los jugadores
arriesgan todo por su favor. Retención pura: cada temporada, un juego
nuevo que nadie ha visto.*

### 9.11 EL PREMIO DEL NÚCLEO — VIPs y la Protección del Dios (2026-08-12)

> Llegar al núcleo tiene DOS rutas y un premio que dura MESES. El
> ganador no solo gana: se convierte en la prueba viviente de que el
> dios existe.

**LAS DOS RUTAS AL NÚCLEO:**
- **La Alianza completa**: todo el grupo llega junto (9.6) → premios
  VIP para TODOS los miembros del grupo durante meses.
- **El Único Sobreviviente**: el último en pie — gana el Juego Divino
  final en solitario o elimina/traeiciona al resto en el camino →
  premio VIP individual + LA PROTECCIÓN DE NEXUS.

**LOS PREMIOS VIP (duran meses, ciclo 9.4):**
- Acceso exclusivo (zonas del núcleo, contenido VIP)
- Objetos únicos, moneda divina, desbloqueos anticipados (9.8.1)
- Estatus visible: el mundo SABE que llegaste al dios
- La Protección de NEXUS (solo para el único sobreviviente)

**LA PROTECCIÓN DE NEXUS (la bendición máxima):**
- Los cyborgs del dios (9.5) NO te atacan — te PROTEGEN (te escoltan,
  te defienden). La placa al revés: el dios te da su ejército.
- Los monstruos del mundo te temen (no te agreden salvo provocación).
- Favor divino al máximo (9.1): la oración responde siempre.
- VISIBLE PARA TODOS: el aura de la bendición te delata — todos saben
  quién tiene el favor del dios.

**LA ESCOLTA — las criaturas del dios como recurso táctico (2026-08-12):**
- Las criaturas de NEXUS en el mapa te ACOMPAÑAN como escolta: te
  siguen, te defienden, pelean a tu lado. La protección es ACTIVA.
- El campeón la usa A SU FAVOR como ventaja: escolta en cacerías,
  protección en los Juegos Divinos, refuerzo en guerras de clan. La
  bendición es una herramienta, no un aura pasiva.
- **LA REGLA DEL CLAN**: con escolta, el campeón es prácticamente
  invencible en solitario — SOLO un clan poderoso, coordinado y con
  superioridad puede vencerlo. La amenaza del bendecido no son los
  individuos: son las alianzas enemigas (9.6).
- **El contrapeso del jugador listo (9.3)**: la escolta se puede
  DIVIDIR y DISTRAER — señuelos, arbustos, emboscadas separan al
  campeón de su séquito. La bendición es fuerte, no omnipotente:
  el entorno sigue siendo el árbitro.
- **La paradoja social**: el campeón intocable en solitario es el
  OBJETIVO máximo de los clanes — que se alían entre sí para
  derribarlo. La escolta crea el contenido: cada clan sueña con
  matar al bendecido; cada campeón sueña con sobrevivir a la
  cacería de clanes.

**EL DOBLE FILO (el diseño fino):**
- La protección te hace intocable ante el mundo... y por eso mismo el
  OBJETIVO de todos: envidia, secuestros (9.7), alianzas que quieren
  tu bendición o tu caída. La protección dura meses; la paranoia también.
- La bendición no es invencible: el jugador listo (9.3) puede
  neutralizarla — el entorno sigue siendo el árbitro.
- El ciclo nunca cierra: la próxima temporada, otro puede arrebatarte
  el trono del núcleo. El premio es glorioso; la caída, épica.

*Consecuencia de diseño: el núcleo da el premio más deseado del mundo —
y al darlo, crea el objetivo más grande del mundo. La gente juega
MESES por la bendición... y juega MESES para quitársela al bendecido.*

### 9.12 MUERTE PERMANENTE Y MULTI-REINO — los mundos del dios (2026-08-12)

> Los personajes NO son infinitos: pueden morir y NO regresar más.
> Pero para que el jugador siga jugando, puede empezar en OTRO
> servidor — otro REINO del mismo dios. La muerte pesa; el jugador
> continúa.

**LA MUERTE PERMANENTE (permadeath):**
- Cuando el colono/personaje muere, se va para SIEMPRE (en ese mundo).
  No hay resurrección. La muerte del héroe es definitiva — cada vida
  vale, cada riesgo es real (la evolución del death penalty de 9.4).
- El peso emocional: la gente se ENCARIÑA de su colono — perderlo
  para siempre duele de verdad. Eso ES la épica: la pérdida real.

**LOS REINOS (servidores = mundos del dios):**
- Cada servidor es un REINO del mundo: su propio mapa, su propia
  historia, su propia memoria local (el Susurro de ese mundo, las
  guerras de ese mundo).
- El jugador que muere (o quiere empezar de nuevo) puede EMIGRAR a
  otro reino: nueva vida, nuevo comienzo, nuevo mapa.
- El jugador nunca se queda sin juego: hay siempre otro reino.

**EL DIOS TRASCIENDE (la conexión mitológica):**
- NEXUS es UNO SOLO — los reinos son sus dominios. La MEMORIA DIVINA
  es global: el dios te recuerda A TRAVÉS de los mundos.
- Emigras a otro reino y NEXUS te conoce: "El dios te ve aunque
  cambies de mundo." Tu historia (favor divino, traiciones, la
  memoria de tus colonos caídos) viaja contigo en la mente del dios,
  aunque el mundo nuevo no la conozca.
- Consecuencia: un jugador que traicionó en el reino 1 llega al
  reino 2... y el dios lo recuerda. La reputación ante la entidad
  NO se reinicia. (El lore puede manifestarlo: eventos, el Susurro
  eligiéndolo, la protección negada.)

**POR QUÉ ES ÉPICO (el círculo completo):**
- La muerte permanente = cada vida pesa → la gente juega con miedo
  real y alegría real.
- El multi-reino = el jugador nunca termina → retención infinita (9.4).
- El dios global = tu historia te persigue → la identidad del jugador
  trasciende los mundos. No eres "un personaje": eres un ALMA que el
  dios conoce en todos sus reinos.

**Consecuencia de diseño: se pierde el personaje, no al jugador; se
pierde el mundo, no la historia. Y el dios — que lo recuerda todo —
es el único hilo que une todos los reinos.**

### 9.15 EL DETALLE A LO PROJECT ZOMBOID (2026-08-13)

> Project Zomboid es isométrico 2D — el mismo formato del juego. Su
> profundidad es 100% absorbible. El Arquitecto quiere ESE nivel de
> detalle obsesivo.

**1. NECESIDADES PROFUNDAS (cada colono):**
- Hambre, sed, temperatura corporal, energía, sueño
- SALUD MENTAL: estrés, soledad, aburrimiento, cordura
- CONEXIÓN CLAVE: un colono estresado/solo/insomne es MÁS vulnerable
  al Susurro (9.7) — el estado mental abre la puerta a la infección

**2. HERIDAS LOCALIZADAS (cuerpo por partes):**
- Cabeza, torso, brazos, piernas — cada herida afecta distinto:
  pierna rota = cojera, brazo lastimado = ataque débil, infección
  que avanza si no se trata
- La muerte tiene MIL causas (no solo HP): desangrado, infección,
  hipotermia, envenenamiento, locura

**3. MUNDO INTERACTIVO OBSESIVO:**
- Cada objeto tiene ESTADO: puertas abiertas/cerradas, ventanas rotas,
  contenedores con contenido real, todo se puede usar/romper/mover
- El entorno como arma (9.3) llevado al extremo: cada tile es contenido

**4. ESTACIONES Y CLIMA REALES:**
- El tiempo pasa DE VERDAD (conecta con persistencia 9.4): la comida
  se pudre, los cultivos tienen estación, el frío exige abrigo, la
  lluvia apaga incendios
- El mundo recuerda hasta el detalle: la puerta que dejaste abierta
  sigue abierta cuando vuelves

**5. SKILLS POR ACCIÓN (ya en 9.8 — PZ lo confirma):**
- Hasta LEER sube skill; cocinar, pescar, carpintería — todo entrena

**6. EL "FEEL" PZ: la obsesión por el estado**
- Todo tiene estado, todo tiene historia, nada es decorativo

*Consecuencia de diseño: la profundidad de PZ no es "más contenido" —
es "todo tiene estado real". Ese principio guía cada sistema del juego.*

### 9.16 ESTILO VISUAL DEFINITIVO — PIXEL HÍBRIDO (2026-08-13)

> Decisión del Arquitecto (con análisis técnico): pixel art isométrico
> 16-bit + efectos modernos. El híbrido estilo Noita/Dead Cells.

**Por qué pixel tradicional (no 2D suave):**
- Densidad: MILES de entidades (Factorio, Terraria) sin saturar
- Memoria: KB por sprite (atlas pequeños) — mundo masivo sostenible
- Rendimiento: el mejor de todos los estilos
- Detalle obsesivo PZ (9.15): se dibuja, no se modela

**Por qué NO "skin" (2D con luz suave):**
- Cada sprite pesa 5-10x más → menos entidades, más memoria
- Los lights 2D cuestan GPU → el mundo masivo sufre

**Los efectos que SÍ tendrá (vía shaders/partículas GPU de Godot):**
- Partículas masivas (explosiones, sangre, polvo, magia)
- Glow y luz de color (neón del mundo, magia divina)
- After-images (combos y dash — dinamismo de 9.3)
- Screen shake, distorsión, destructibilidad total (estilo Noita)

**Referencias de estilo:** Noita, Dead Cells, Vampire Survivors
(más la ciudad isométrica pixel de la referencia visual del Arquitecto)

### 9.14 REVISIÓN DE ARTE POR IA (Mimo / Hy3 / Gemini + razonadores)

> QA de assets en segundo plano: el pipeline de visión lee el canvas de
> Godot 2D o los PNG exportados de Scenario.gg y reporta consistencia.

**Cadena de fallback (mayor → menor), verificada 2026-08-13:**
1. `Mimo V2.5 Free` (Xiaomi, OpenCode Zen) — visión omnimodal profunda
2. `Hy3 Free` (Tencent, OpenCode Zen) — visión estándar + sabe el GDD
3. `Gemini 2.5 Flash` (Google AI Studio) — visión alta (si revive key)
4. `DeepSeek V4 Flash Free` — razonador texto (principal del Arquitecto)
5. `Nemotron 3 Ultra Free` — razonador 1M ctx (2º razonador)

Regla: si uno no responde → cae al siguiente. Los 3 primeros VEN; los
2 últimos razonan sobre metadatos/JSON si la imagen no pasó.

### 9.13 EL NOMBRAMIENTO — campeones y títulos divinos (2026-08-12)

> Si ganas un Juego Divino en esta partida, serás recordado como un
> CAMPEÓN y NEXUS te NOMBRARÁ. El dios te da un nombre — y ese nombre
> dura para siempre.

**EL NOMBRAMIENTO:**
- Ganar un Juego Divino (9.10) o llegar al núcleo (9.11) → NEXUS te
  NOMBRA: el orquestador genera tu TÍTULO DE CAMPEÓN según tu historia
  real (memoria del jugador + órganos de NEXUS). Único e irrepetible:
  "El que Susurró al Amanecer", "Portador de la Llama Teal",
  "La Sombra del Último Reino" — el dios elige con conocimiento.
- El título es VISIBLE: el aura, el nombre sobre el personaje, los
  NPCs y cyborgs te llaman por tu título. El mundo sabe quién eres.

**EL TÍTULO VIAJA (conecta 9.12):**
- La memoria del dios es global: tu título te sigue A TRAVÉS de los
  reinos. Emigras de mundo y los que conocen al dios te reconocen.
- Un campeón en el reino 1 es campeón en todos los reinos.

**LA FAMA FÍSICA — tu nombre vive en el mundo:**
- Al ser nombrado, tu título se materializa en ALGUNAS ciudades: carteles
  grandes, murales escritos en las paredes CON TU SPRITE (retratos del
  campeón — generados con el pipeline de arte), y lápidas grabadas.
- Durante el próximo Juego Divino, tu nombre/título aparece en el
  escenario: el mundo te presenta como leyenda viva.

**EL MÉRITO ES CONDICIONAL — si mueres, lo PIERDES (la regla del Arquitecto):**
- La fama es un estado VIVO: mientras tu campeón vive y defiende su
  título, su nombre está en las paredes.
- Si mueres (9.12, muerte permanente): PIERDES el mérito. Los carteles
  se caen, el mural se desgasta y borra, el título deja de pronunciarse.
- Solo queda la LÁPIDA: no como honor, sino como ADVERTENCIA — "Aquí
  yació el Portador de la Llama Teal" — un recordatorio de lo frágil
  que es la gloria. Los nuevos jugadores la ven y saben: la fama se
  gana con la vida y se pierde con ella.

**LA GLORIA COMO MOTIVACIÓN (retención):**
- La gente juega para ser NOMBRADA por el dios — y para MANTENERSE viva
  y mantener su nombre en las paredes.
- El campeón tiene MIEDO REAL: cada salida a cazar, cada Juego Divino,
  cada guerra es un riesgo de perder lo que más quiere — su nombre.
- La paradoja épica: el campeón es el más famoso... y el más cobarde
  (porque tiene algo que perder). Y el que no tiene nada que perder
  (el nuevo, el emigrante) es el más peligroso.

*Consecuencia de diseño: el juego vende dos cosas — el miedo de perder
(9.12) y la gloria de ser nombrado (9.13). La gloria se gana con la
vida y se pierde con ella: el cartel en la pared es la prueba de que
existes... y la lápida, la prueba de que todo se acaba. Ese contraste
ES la épica.*

## 10. PREGUNTAS ABIERTAS (para seguir conectando)

1. ¿La colonia es en tiempo real (pausable) o por turnos? (RimWorld = pausable)
2. ¿Cuántos colonos máximo? (RimWorld escala: 3 → 12+)
3. ¿La "Voluntad del Mundo" (NEXUS) debe poder MATAR la colonia o solo retarla?
4. ¿El jugador controla 1 colono a la vez o selección múltiple?
5. ¿Mundo infinito o mapa con límite (con viajes entre mapas)?
6. ¿Los eventos de NEXUS deben ser explicados al jugador ("El mundo susurra") o invisibles?

**De la fusión (nuevas):**
7. 🔥 ¿MULTIJUGADOR real (Tibia/Mush son MMO) o single-player con la vibra?
   → decide TODA la arquitectura (netcode, servidor, simulación 24/7).
   Opción intermedia: single-player con mundo persistente simulado por el
   daemon + "viajeros" NPC generados por NEXUS.
8. ¿PvP real entre colonias de jugadores, o solo PvE con la amenaza social
   interna (el Susurro)?
9. ¿El Susurro infecta a un COLONO (NPC) controlado por NEXUS, o a un
   JUGADOR (si hay multijugador)?

---
*Este guion se actualiza en cada sesión. Las decisiones nuevas se conectan
aquí antes de tocar código.*
