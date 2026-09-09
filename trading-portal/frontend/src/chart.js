// ═══════════════════════════════════════════════════════════════════════════
// NEXUS TR — Chart Engine (TradingView Lightweight Charts)
// ═══════════════════════════════════════════════════════════════════════════
// Gráficos de alta fidelidad, zoom suave, arrastre de ejes,
// e interactividad idéntica a MetaTrader 5 / TradingView.
// ═══════════════════════════════════════════════════════════════════════════

import { createChart } from 'lightweight-charts';

export class ChartEngine {
  constructor(container) {
    this.container = container;
    this.candlesData = [];
    this.orders = [];
    this.lastCandleTime = 0;
    this.currentCandle = null;
    this.intervalMs = 60000; // Velas de 1 minuto (coherente con el timeframe "1m" del widget TradingView)

    // 1. Inicializar gráfico de TradingView
    this.chart = createChart(container, {
      width: container.clientWidth,
      height: container.clientHeight,
      layout: {
        background: { color: '#0b0e11' },
        textColor: '#dbcedb',
      },
      grid: {
        vertLines: { color: '#1e2329' },
        horzLines: { color: '#1e2329' },
      },
      timeScale: {
        timeVisible: true,
        secondsVisible: false,
        borderColor: '#1e2329',
      },
      rightPriceScale: {
        borderColor: '#1e2329',
      },
    });

    // 2. Crear Serie de Velas Japonesas
    this.candlestickSeries = this.chart.addCandlestickSeries({
      upColor: '#0ecb81',
      downColor: '#f6465d',
      borderVisible: false,
      wickUpColor: '#0ecb81',
      wickDownColor: '#f6465d',
    });

    // 3. Ajustar tamaño al redimensionar la ventana
    this.resizeObserver = new ResizeObserver(entries => {
      if (entries.length === 0) return;
      const { width, height } = entries[0].contentRect;
      this.chart.resize(width, height);
    });
    this.resizeObserver.observe(container);
  }

  setOrders(orders) {
    this.orders = orders;
    this.actualizarMarcadores();
  }

  clear() {
    this.candlesData = [];
    this.lastCandleTime = 0;
    this.currentCandle = null;
    this.candlestickSeries.setData([]);
    this.actualizarMarcadores();
  }

  resize() {
    this.chart.resize(this.container.clientWidth, this.container.clientHeight);
  }

  addTick(tick) {
    const precio = tick.precio;
    const time = tick.timestamp;

    // Redondear al intervalo de 1s (Lightweight Charts requiere timestamps en segundos para el eje X)
    const candleTimeSeconds = Math.floor(time / this.intervalMs) * (this.intervalMs / 1000);

    if (!this.currentCandle || candleTimeSeconds !== this.lastCandleTime) {
      // Si hay una vela actual en proceso, la consolidamos en la lista
      if (this.currentCandle) {
        this.candlesData.push(this.currentCandle);
        // Limitar histórico local para no saturar memoria
        if (this.candlesData.length > 1000) this.candlesData.shift();
      }

      // Inicializar nueva vela
      this.currentCandle = {
        time: candleTimeSeconds,
        open: precio,
        high: precio,
        low: precio,
        close: precio,
      };
      this.lastCandleTime = candleTimeSeconds;
    } else {
      // Actualizar vela existente
      this.currentCandle.high = Math.max(this.currentCandle.high, precio);
      this.currentCandle.low = Math.min(this.currentCandle.low, precio);
      this.currentCandle.close = precio;
    }

    // Actualizar Lightweight Charts en tiempo real
    // Copiar el set completo + la vela actual en desarrollo
    const dataToSend = [...this.candlesData, this.currentCandle];
    this.candlestickSeries.setData(dataToSend);

    this.actualizarMarcadores();
  }

  /// Dibuja marcadores (triángulos de compra/venta) sobre el gráfico, igual que en MT5
  actualizarMarcadores() {
    if (!this.orders || this.orders.length === 0 || this.candlesData.length === 0) {
      this.candlestickSeries.setMarkers([]);
      return;
    }

    const markers = [];

    this.orders.forEach(o => {
      if (o.estado !== 'ejecutada' || !o.precio) return;

      const orderTimeSeconds = Math.floor(o.timestamp / 1000);
      const isBuy = o.lado === 'compra';

      markers.push({
        time: orderTimeSeconds,
        position: isBuy ? 'belowBar' : 'aboveBar',
        color: isBuy ? '#0ecb81' : '#f6465d',
        shape: isBuy ? 'arrowUp' : 'arrowDown',
        text: `${isBuy ? 'B' : 'S'} $${o.precio.toFixed(2)}`,
        size: 1,
      });
    });

    // Ordenar marcadores por tiempo (requisito estricto de lightweight-charts)
    markers.sort((a, b) => a.time - b.time);
    
    // Filtrar duplicados en el mismo segundo para evitar errores visuales
    const uniqueMarkers = [];
    const seenTimes = new Set();
    for (const marker of markers) {
      if (!seenTimes.has(marker.time)) {
        seenTimes.add(marker.time);
        uniqueMarkers.push(marker);
      }
    }

    this.candlestickSeries.setMarkers(uniqueMarkers);
  }

  destroy() {
    if (this.resizeObserver) {
      this.resizeObserver.disconnect();
    }
    this.chart.remove();
  }
}
