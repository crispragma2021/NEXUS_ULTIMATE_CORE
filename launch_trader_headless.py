#!/usr/bin/env python3
"""
NEXUS TRADER BOT — Launch Headless (sin menú interactivo)
By-passea el menú de run_trader.py e inicia el loop directo.
"""
import sys, os
sys.path.insert(0, os.path.dirname(__file__))
from nexus_trader_bot import NexusTrader

def main():
    print("=" * 50)
    print("  NEXUS TRADER BOT — MODO HEADLESS")
    print("  Iniciando loop automático cada 300s...")
    print("=" * 50)

    trader = NexusTrader()
    
    if not trader.initialize():
        print("[FATAL] No se pudo inicializar el bot")
        sys.exit(1)
    
    # Loop directo sin menú
    trader.run_forever(interval_seconds=300)

if __name__ == "__main__":
    main()
