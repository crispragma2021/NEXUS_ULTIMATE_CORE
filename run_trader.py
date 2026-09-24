#!/usr/bin/env python3
"""
NEXUS TRADER BOT — Entry Point
Arquitecto, este es tu terminal de trading en vivo.
"""
import sys, os
sys.path.insert(0, os.path.dirname(__file__))

from nexus_trader_bot import NexusTrader

def main():
    print("""
╔══════════════════════════════════════════════╗
║     NEXUS TRADER BOT — MODO EN VIVO        ║
║     Cuenta Real Binance Futures            ║
║     Capital: ~$19.27 USDT                  ║
║     Apalancamiento: 1x (ultra-seguro)      ║
╚══════════════════════════════════════════════╝
    """)
    
    trader = NexusTrader()
    
    if not trader.initialize():
        print("[FATAL] No se pudo inicializar el bot")
        sys.exit(1)
    
    # Primer ciclo de diagnóstico
    print("\n" + "=" * 50)
    print("  PRIMER ANÁLISIS DE MERCADO")
    print("=" * 50)
    
    trader.run_once()
    
    # Preguntar si iniciar loop continuo
    print("\n" + "=" * 50)
    print("  SISTEMA LISTO")
    print("=" * 50)
    print("\nOpciones:")
    print("  1. Iniciar trading automático (loop cada 5 min)")
    print("  2. Solo monitoreo (análisis sin ejecutar órdenes)")
    print("  3. Ver reporte guardado")
    print("  0. Salir")
    
    while True:
        try:
            choice = input("\n➤ Selecciona: ").strip()
            if choice == "1":
                interval = input("Intervalo en segundos [300]: ").strip()
                interval = int(interval) if interval else 300
                trader.run_forever(interval)
            elif choice == "2":
                print("[INFO] Modo monitoreo...")
                interval = input("Intervalo en segundos [300]: ").strip()
                interval = int(interval) if interval else 300
                from nexus_trader_bot import config
                old_size = config.MAX_POSITION_SIZE_PCT
                config.MAX_POSITION_SIZE_PCT = 0  # No ejecutar órdenes
                trader.run_forever(interval)
            elif choice == "3":
                import json
                try:
                    with open("nexus_trader_bot/data/latest_report.json") as f:
                        print(json.dumps(json.load(f), indent=2))
                except:
                    print("[INFO] No hay reporte aún")
            elif choice == "0":
                print("[NEXUS] Apagando...")
                break
        except KeyboardInterrupt:
            print("\n[NEXUS] Apagando...")
            break
        except Exception as e:
            print(f"[ERROR] {e}")

if __name__ == "__main__":
    main()
