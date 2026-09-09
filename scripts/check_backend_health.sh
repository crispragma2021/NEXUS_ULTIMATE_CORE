#!/bin/bash
for i in $(seq 1 15); do
  sleep 2
  if curl -s -o /dev/null -w "%{http_code}" http://localhost:43210/api/health 2>/dev/null | grep -q '200\|404'; then
    echo "✅ Backend respondiendo en iteración $i"
    exit 0
  fi
  echo "Esperando... ($i)"
done
echo "❌ Backend no respondió a tiempo."
exit 1
