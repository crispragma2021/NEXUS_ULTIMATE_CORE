import React from 'react'
import ReactFlow, { Background, Controls } from 'reactflow'
import 'reactflow/dist/style.css'

/**
 * AgentCanvas — Lienzo/editor de nodos (Regla 4).
 * Dibuja el flujo visual del orquestador y sus agentes con React Flow.
 * Dark mode nativo.
 */
export default function AgentCanvas({ nodes, edges }) {
  return (
    <div className="h-full w-full">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        fitView
        proOptions={{ hideAttribution: true }}
        nodesDraggable
        nodesConnectable={false}
      >
        <Background color="#1e1e2e" gap={16} />
        <Controls showInteractive={false} />
      </ReactFlow>
    </div>
  )
}
