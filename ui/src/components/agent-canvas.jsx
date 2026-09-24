import React, { useCallback } from 'react';
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  addEdge,
} from 'reactflow';
import 'reactflow/dist/style.css';

/**
 * AgentCanvas — Lienzo de nodos interactivos (Capas / ERD).
 * Permite arrastrar, mover, conectar y organizar bloques de arquitectura.
 */
export default function AgentCanvas({ nodes: initialNodes = [], edges: initialEdges = [] }) {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

  // Permite conectar nuevos nodos si el usuario arrastra un cable entre ellos
  const onConnect = useCallback(
    (params) => setEdges((eds) => addEdge({ ...params, animated: true }, eds)),
    [setEdges]
  );

  return (
    <div className="relative flex-1 h-full w-full min-h-0 bg-[#030708]">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        fitView
        proOptions={{ hideAttribution: true }}
        nodesDraggable={true}
        nodesConnectable={true}
        elementsSelectable={true}
        snapToGrid={true}
        snapGrid={[15, 15]}
      >
        <Background color="#10b981" gap={20} size={1} opacity={0.15} />
        <Controls showInteractive={true} className="fill-zinc-400 bg-zinc-900 border-zinc-800" />
        <MiniMap 
          nodeColor="#10b981"
          maskColor="rgba(3, 7, 8, 0.8)"
          className="bg-[#060b0f] border border-zinc-800 rounded-lg overflow-hidden"
        />
      </ReactFlow>
    </div>
  );
}
