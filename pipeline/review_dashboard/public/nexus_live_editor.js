(function () {
  console.log("⚡ [NEXUS LIVE EDITOR] Motor de Edición Táctil & Drag-and-Drop activado.");

  const API_BASE = window.location.origin;

  window.NexusLiveEditor = {
    activePointerId: null,
    draggedElement: null,
    startPos: { x: 0, y: 0 },
    initialOffset: { x: 0, y: 0 },

    async updateElement(componentId, field, newValue) {
      console.log(`✏️ [NEXUS LIVE EDITOR] Sincronizando: ${componentId}.${field} -> "${newValue}"`);
      try {
        const res = await fetch(`${API_BASE}/api/refine`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            prompt: `Actualizar ${field} en ${componentId} a "${newValue}"`,
            component_id: componentId,
            field: field,
            new_value: newValue
          })
        });
        const data = await res.json();
        console.log(`✅ [NEXUS LIVE EDITOR] Sincronizado:`, data.message);
        window.dispatchEvent(new CustomEvent('nexus-ui-updated', { detail: data }));
      } catch (err) {
        console.error(`❌ Error de conexión con ${API_BASE}:`, err);
      }
    },

    enableInlineEditing() {
      document.querySelectorAll('.nexus-editable, [contenteditable="true"]').forEach(el => {
        el.addEventListener('blur', function () {
          const compId = el.getAttribute('data-comp-id') || el.closest('[id]')?.id || 'element_gen';
          const field = el.getAttribute('data-field') || 'text';
          window.NexusLiveEditor.updateElement(compId, field, el.textContent.trim());
        });
        el.addEventListener('keydown', function (e) {
          if (e.key === 'Enter') {
            e.preventDefault();
            el.blur();
          }
        });
      });
    },

    enableTouchAndMouseDraggable() {
      const draggables = document.querySelectorAll('.v0-card, .shadcn-card, .draggable-item');

      draggables.forEach(item => {
        item.style.touchAction = 'none';

        // Restaurar posición guardada previo drag si existe
        let curX = parseFloat(item.getAttribute('data-x') || '0');
        let curY = parseFloat(item.getAttribute('data-y') || '0');

        if (curX || curY) {
          item.style.transform = `translate3d(${curX}px, ${curY}px, 0)`;
        }

        item.addEventListener('pointerdown', (e) => {
          if (e.target.isContentEditable || e.target.closest('[contenteditable="true"]')) return;

          this.draggedElement = item;
          this.activePointerId = e.pointerId;
          item.setPointerCapture(e.pointerId);

          this.startPos = { x: e.clientX, y: e.clientY };
          this.initialOffset = {
            x: parseFloat(item.getAttribute('data-x') || '0'),
            y: parseFloat(item.getAttribute('data-y') || '0')
          };

          item.style.transition = 'none';
          item.style.zIndex = '100';
          item.classList.add('shadow-2xl', 'ring-2', 'ring-emerald-500');
        });

        item.addEventListener('pointermove', (e) => {
          if (!this.draggedElement || this.draggedElement !== item || this.activePointerId !== e.pointerId) return;

          const dx = e.clientX - this.startPos.x;
          const dy = e.clientY - this.startPos.y;

          const newX = this.initialOffset.x + dx;
          const newY = this.initialOffset.y + dy;

          item.setAttribute('data-x', newX);
          item.setAttribute('data-y', newY);
          item.style.transform = `translate3d(${newX}px, ${newY}px, 0) scale(1.02)`;
        });

        const handlePointerUp = (e) => {
          if (!this.draggedElement || this.draggedElement !== item || this.activePointerId !== e.pointerId) return;

          item.releasePointerCapture(e.pointerId);

          const finalX = parseFloat(item.getAttribute('data-x') || '0');
          const finalY = parseFloat(item.getAttribute('data-y') || '0');

          item.style.transition = 'transform 0.15s ease-out, box-shadow 0.15s ease-out';
          item.style.transform = `translate3d(${finalX}px, ${finalY}px, 0)`;
          item.style.zIndex = '1';
          item.classList.remove('shadow-2xl', 'ring-2', 'ring-emerald-500');

          console.log(`📍 [NEXUS DRAG] Elemento [${item.id || 'card'}] fijado permanentemente en: (${finalX}px, ${finalY}px)`);

          this.draggedElement = null;
          this.activePointerId = null;
        };

        item.addEventListener('pointerup', handlePointerUp);
        item.addEventListener('pointercancel', handlePointerUp);
      });
    }
  };

  document.addEventListener('DOMContentLoaded', () => {
    window.NexusLiveEditor.enableInlineEditing();
    window.NexusLiveEditor.enableTouchAndMouseDraggable();
  });
})();
